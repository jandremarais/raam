use glyphon::{
    Attrs, FontSystem, Metrics, Resolution, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer,
};
use wgpu::{MultisampleState, TextureFormat};
use winit::{event::WindowEvent, window::Window};

pub(crate) struct State<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    text_system: TextSystem,
}

impl<'a> State<'a> {
    pub(crate) async fn new(window: &'a Window, instance: wgpu::Instance) -> Self {
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

        let text_system = TextSystem::new(&device, &queue, surface_format);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            text_system,
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
        Ok(())
    }

    pub(crate) fn process_input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub(crate) fn prepare(&mut self) {
        self.text_system.prepare(
            &self.device,
            &self.queue,
            self.config.width,
            self.config.height,
        );
    }
}

struct TextSystem {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    renderer: TextRenderer,
    buffer: glyphon::Buffer,
}

impl TextSystem {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let mut atlas = TextAtlas::new(device, queue, format);
        let metrics = Metrics::new(12.0, 14.);
        let renderer = TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        let mut buffer = glyphon::Buffer::new(&mut font_system, metrics);
        buffer.set_size(&mut font_system, 100., 100.);
        buffer.set_text(
            &mut font_system,
            "hello world",
            Attrs::new(),
            glyphon::Shaping::Advanced,
        );

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            buffer,
        }
    }

    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        let areas = [TextArea {
            buffer: &self.buffer,
            left: 100.0,
            top: 15.0,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: i32::MAX,
                bottom: i32::MAX,
            },
            default_color: glyphon::Color::rgb(255, 255, 255),
        }];
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
