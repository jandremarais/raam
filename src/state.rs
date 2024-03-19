use std::time::Instant;

use datafusion::{
    arrow::{
        error::ArrowError,
        record_batch::RecordBatch,
        util::display::{ArrayFormatter, FormatOptions},
    },
    prelude::*,
};
use glyphon::{
    Attrs, FontSystem, Metrics, Resolution, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer,
};
use tokio::{runtime::Builder, sync::oneshot};
use wgpu::{MultisampleState, TextureFormat};
use winit::{
    event::{MouseScrollDelta, WindowEvent},
    window::Window,
};

pub(crate) struct State<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    // ctx: SessionContext,
    text_system: TextSystem,
    offsets: Offsets,
    query_tx: tokio::sync::mpsc::Sender<usize>,
    results_rx: tokio::sync::mpsc::Receiver<Vec<RecordBatch>>,
    last_update: usize,
}

struct Cell {
    col: usize,
    row: usize,
    buffer: glyphon::Buffer,
}

impl Cell {
    fn new(col: usize, row: usize, buffer: glyphon::Buffer) -> Self {
        Self { col, row, buffer }
    }
}

impl<'a> State<'a> {
    pub(crate) async fn new(window: &'a Window, instance: wgpu::Instance) -> Self {
        let (query_tx, mut query_rx) = tokio::sync::mpsc::channel::<usize>(2);
        let (results_tx, results_rx) = tokio::sync::mpsc::channel::<Vec<RecordBatch>>(2);
        let (field_tx, field_rx) = oneshot::channel();

        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        std::thread::spawn(move || {
            rt.block_on(async move {
                let ctx = SessionContext::new();
                ctx.register_csv("example", "measurements.csv", CsvReadOptions::new())
                    .await
                    .unwrap();
                let df = ctx
                    .sql("SELECT inlet_temperature, outlet_temperature, energy_usage FROM example")
                    .await
                    .unwrap();
                let field_names = df
                    .schema()
                    .fields()
                    .iter()
                    .map(|f| f.name().clone())
                    .collect();
                field_tx.send(field_names).unwrap();

                while let Some(skip) = query_rx.recv().await {
                    let now = Instant::now();
                    let batches = df
                        .clone()
                        .limit(skip, Some(100))
                        .unwrap()
                        .collect()
                        .await
                        .unwrap();
                    results_tx.send(batches).await.unwrap();
                    println!("query done in: {:?}", now.elapsed());
                }
            })
        });
        println!("outside thread");
        let field_names = field_rx.await.unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let text_system = TextSystem::new(&device, &queue, surface_format, field_names);
        query_tx.send(0).await.unwrap();
        let last_update = 0;

        let offsets = Offsets::default();

        Self {
            surface,
            device,
            queue,
            config,
            size,
            text_system,
            offsets,
            query_tx,
            results_rx,
            last_update,
        }
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub(crate) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let bg_color = wgpu::Color {
                r: 0.,
                g: 0.005,
                b: 0.06,
                a: 1.0,
            };
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(bg_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            self.text_system.render(&mut render_pass);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        self.text_system.atlas.trim(); // TODO: is this where the trim should go?
        Ok(())
    }

    pub(crate) fn process_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(x, y),
                ..
            } => {
                self.offsets.update(*x, *y, 10.);
                true
            }

            _ => false,
        }
    }

    pub(crate) fn prepare(&mut self) {
        if let Ok(batches) = self.results_rx.try_recv() {
            self.text_system.update_buffers(&batches, self.last_update);
        }
        let current_line = self.offsets.y / -14.;
        if (current_line - self.last_update as f32).abs() > 50. {
            dbg!(self.offsets.y, current_line, self.last_update);
            self.query_tx.blocking_send(current_line as usize).unwrap();
            self.last_update = current_line as usize;
        }
        self.text_system.prepare(
            &self.device,
            &self.queue,
            self.config.width,
            self.config.height,
            self.offsets,
        );
    }
}

#[derive(Default, Clone, Copy)]
struct Offsets {
    x: f32,
    y: f32,
}

impl Offsets {
    fn update(&mut self, x_delta: f32, y_delta: f32, speed: f32) {
        self.y += speed * y_delta * y_delta * y_delta.signum();
        self.x += speed * x_delta * x_delta * x_delta.signum();
        // dbg!(self.y);
        self.y = self.y.min(0.);
        self.x = self.x.min(0.);
    }
}

struct TextSystem {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    metrics: Metrics,
    renderer: TextRenderer,
    field_buffers: Vec<Cell>,
    buffers: Vec<Cell>,
}

impl TextSystem {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: TextureFormat,
        field_names: Vec<String>,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let mut atlas = TextAtlas::new(device, queue, format);
        let metrics = Metrics::new(12.0, 14.);
        let renderer = TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        let field_buffers = field_names
            .into_iter()
            .enumerate()
            .map(|(j, f)| {
                let mut buffer = glyphon::Buffer::new(&mut font_system, metrics);
                let mut buffer_bor = buffer.borrow_with(&mut font_system);
                buffer_bor.set_size(100., 14.);
                buffer_bor.set_wrap(glyphon::Wrap::Glyph);
                buffer_bor.set_text(&f, Attrs::new(), glyphon::Shaping::Advanced);
                Cell::new(j, 0, buffer)
            })
            .collect();

        Self {
            font_system,
            swash_cache,
            atlas,
            metrics,
            renderer,
            field_buffers,
            buffers: vec![],
        }
    }

    fn update_buffers(&mut self, batches: &[RecordBatch], skip: usize) {
        let now = Instant::now();
        let format_options = FormatOptions::default();
        let mut cells = Vec::new();
        // for (j, field) in batches[0].schema().fields().iter().enumerate() {
        //     let mut buffer = glyphon::Buffer::new(&mut self.font_system, self.metrics);
        //     let mut buffer_bor = buffer.borrow_with(&mut self.font_system);
        //     buffer_bor.set_size(100., 14.);
        //     buffer_bor.set_wrap(glyphon::Wrap::Glyph);
        //     buffer_bor.set_text(field.name(), Attrs::new(), glyphon::Shaping::Advanced);
        //     cells.push(Cell::new(j, 0, buffer));
        // }
        for batch in batches.iter() {
            let formatters = batch
                .columns()
                .iter()
                .map(|c| ArrayFormatter::try_new(c.as_ref(), &format_options))
                .collect::<Result<Vec<_>, ArrowError>>()
                .unwrap();

            for row in 0..batch.num_rows() {
                for (j, formatter) in formatters.iter().enumerate() {
                    let val = formatter.value(row);
                    let mut buffer = glyphon::Buffer::new(&mut self.font_system, self.metrics);
                    let mut buffer_bor = buffer.borrow_with(&mut self.font_system);
                    buffer_bor.set_size(100., 14.);
                    buffer_bor.set_wrap(glyphon::Wrap::Glyph);
                    buffer_bor.set_text(&val.to_string(), Attrs::new(), glyphon::Shaping::Advanced);
                    cells.push(Cell::new(j, row + 1 + skip, buffer));
                }
            }
        }
        println!("buffers took: {:?}", now.elapsed());

        self.buffers = cells;
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        offsets: Offsets,
    ) {
        let mut areas: Vec<_> = self
            .field_buffers
            .iter()
            .map(|c| TextArea {
                buffer: &c.buffer,
                left: offsets.x + (c.col as f32 * 110.),
                top: 0.,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: i32::MAX,
                    bottom: i32::MAX,
                },
                default_color: glyphon::Color::rgb(180, 180, 180),
            })
            .collect();

        let cell_areas: Vec<_> = self
            .buffers
            .iter()
            .map(|c| TextArea {
                buffer: &c.buffer,
                left: offsets.x + (c.col as f32 * 110.),
                top: offsets.y + (c.row as f32 * 14.),
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 14,
                    right: i32::MAX,
                    bottom: i32::MAX,
                },
                default_color: glyphon::Color::rgb(240, 240, 255),
            })
            .collect();
        areas.extend(cell_areas);
        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                Resolution { width, height },
                areas,
                &mut self.swash_cache,
            )
            .unwrap();
    }

    fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        self.renderer.render(&self.atlas, pass).unwrap();
    }
}
