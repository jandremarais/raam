use std::time::Instant;

use cosmic_text::{Attrs, FontSystem, Metrics, SwashCache};
use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

use crate::{
    camera::{Camera, CameraController, CameraUniform},
    grid::Grid,
    line::{self},
};

pub(crate) struct State<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    pub(crate) num_indices: u32,
    pub(crate) line_instances: Vec<line::Instance>,
    grid: Grid,
    grid_buffer_size: u64,
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_buffer: cosmic_text::Buffer,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) instance_buffer: wgpu::Buffer,
    pub(crate) render_pipeline: wgpu::RenderPipeline,
    last_reload: f32,
}

impl<'a> State<'a> {
    pub(crate) async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

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

        let camera = Camera::default();
        let camera_uniform = CameraUniform::from(&camera);
        let camera_buffer = camera_uniform.to_buffer(&device);
        let camera_bind_group_layout =
            device.create_bind_group_layout(&CameraUniform::bind_group_layout_desc());
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        let camera_controller = CameraController::new(10.);

        let line_vertices = line::VERTICES;
        let line_indices = line::INDICES;
        let num_indices = line_indices.len() as u32;
        let grid = Grid::new(1000, 10, 20., 100., 2.);
        let mut line_instances = grid.line_instances();

        let grid_buffer_size = 4 * line_instances.len() as u64;

        // text stuff
        let mut font_system = FontSystem::new();
        let mut swash_cache = SwashCache::new();
        let metrics = Metrics::new(12.0, 20.);
        let mut text_buffer = cosmic_text::Buffer::new(&mut font_system, metrics);
        let mut text_buffer_b = text_buffer.borrow_with(&mut font_system);
        text_buffer_b.set_size(grid.col_width, grid.row_height);
        let attrs = Attrs::new();
        let text_color = cosmic_text::Color::rgb(0xFF, 0xFF, 0xFF);

        for i in 0..grid.ncols {
            // for j in 0..grid.nrows {
            for j in 0..50 {
                text_buffer_b.set_text(&format!("data {i}"), attrs, cosmic_text::Shaping::Advanced);
                text_buffer_b.shape_until_scroll(true);

                text_buffer_b.draw(&mut swash_cache, text_color, |x, y, w, h, color| {
                    line_instances.push(line::Instance::new(
                        (
                            x as f32 + grid.line_width + 1. + i as f32 * grid.col_width,
                            y as f32 + grid.line_width + j as f32 * grid.row_height,
                        ),
                        (w as f32, h as f32),
                        color.a() as f32,
                    ));
                });
            }
        }
        // <-- end

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(line_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(line_indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&line_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[line::Vertex::desc(), line::Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            num_indices,
            line_instances,
            grid,
            grid_buffer_size,
            font_system,
            swash_cache,
            text_buffer,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            render_pipeline,
            last_reload: 0.,
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
        let output = self.surface.get_current_texture()?;

        let view = output
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
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.line_instances.len() as _);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub(crate) fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_event(event)
    }

    pub(crate) fn update(&mut self) {
        let start = Instant::now();
        self.camera_controller
            .update_camera(&mut self.camera, self.size);
        self.camera_uniform = CameraUniform::from(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let total_height = self.grid.row_height;
        if (self.last_reload - self.camera.y).abs() > (total_height * 10.) {
            let mut text_buffer_b = self.text_buffer.borrow_with(&mut self.font_system);
            let attrs = Attrs::new();
            let text_color = cosmic_text::Color::rgb(0xFF, 0xFF, 0xFF);
            let mut line_instances = vec![];
            let start_row = (self.camera.y.abs() / total_height) as usize;
            // dbg!(start_row, self.camera.y, total_height);
            for i in 0..10 {
                // let i = 0;
                for j in start_row..(start_row + 50) {
                    // let j = 0;
                    text_buffer_b.set_text(
                        &format!("data {i}"),
                        attrs,
                        cosmic_text::Shaping::Advanced,
                    );
                    text_buffer_b.shape_until_scroll(true);

                    text_buffer_b.draw(&mut self.swash_cache, text_color, |x, y, w, h, color| {
                        line_instances.push(line::Instance::new(
                            (
                                x as f32
                                    + 1.
                                    + i as f32 * self.grid.col_width
                                    + self.grid.line_width,
                                y as f32 + j as f32 * self.grid.row_height + self.grid.line_width,
                            ),
                            (w as f32, h as f32),
                            color.a() as f32,
                        ));
                    });
                }
            }
            let grid_offset = 20 * self.grid.nlines() as u64;
            self.queue.write_buffer(
                &self.instance_buffer,
                grid_offset,
                bytemuck::cast_slice(&line_instances),
            );
            self.last_reload = -1. * (start_row as f32 * total_height);
            println!(
                "reloading at {} for rows in {} {}",
                self.camera.y,
                start_row,
                start_row + 50
            );
        }
        // println!("update took: {:?}", start.elapsed());
        // let tmp: wgpu::Buffer = self.instance_buffer.slice(0..12).into();
        // tmp.into()
    }
}
