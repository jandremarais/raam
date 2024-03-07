use glyphon::{
    Attrs, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas,
    TextBounds, TextRenderer,
};
use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

use crate::{
    camera::{Camera, CameraController, CameraUniform},
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
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) instance_buffer: wgpu::Buffer,
    pub(crate) render_pipeline: wgpu::RenderPipeline,
    cache: SwashCache,
    text_renderer: TextRenderer,
    buffer: glyphon::Buffer,
    buffer2: glyphon::Buffer,
    font_system: FontSystem,
    atlas: TextAtlas,
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

        // TODO: put these params elsewhere
        let row_height = 20.0;
        let col_width = 100.0;
        let line_width = 2.;
        let ncols = 50;
        let nrows = 1_000_000;
        let xlim = ncols as f32 * col_width + line_width;
        let ylim = nrows as f32 * row_height + line_width;
        let hlines: Vec<_> = (0..nrows + 1)
            .map(|i| line::Instance::new((0., i as f32 * row_height), (xlim, line_width)))
            .collect();
        let vlines: Vec<_> = (0..ncols + 1)
            .map(|i| line::Instance::new((i as f32 * col_width, 0.), (line_width, ylim)))
            .collect();
        let mut line_instances = Vec::new();
        line_instances.extend_from_slice(&hlines);
        line_instances.extend_from_slice(&vlines);

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
            usage: wgpu::BufferUsages::VERTEX,
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
                    blend: Some(wgpu::BlendState::REPLACE),
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

        // text stuff
        let mut font_system = FontSystem::new();
        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(&device, &queue, surface_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        let mut buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(12.0, 18.0));

        let scale_factor = window.scale_factor();
        let physical_width = (size.width as f64 * scale_factor) as f32;
        let physical_height = (size.height as f64 * scale_factor) as f32;

        buffer.set_size(&mut font_system, physical_width, physical_height);
        buffer.set_text(&mut font_system, "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system);

        let mut buffer2 = glyphon::Buffer::new(&mut font_system, Metrics::new(12.0, 18.0));
        buffer2.set_size(&mut font_system, physical_width, physical_height);
        buffer2.set_text(
            &mut font_system,
            "Another 1",
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        buffer2.shape_until_scroll(&mut font_system);

        // end

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
            vertex_buffer,
            index_buffer,
            instance_buffer,
            render_pipeline,
            cache,
            text_renderer,
            buffer,
            buffer2,
            font_system,
            atlas,
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
        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.atlas,
                Resolution {
                    width: self.size.width,
                    height: self.size.height,
                },
                [
                    TextArea {
                        buffer: &self.buffer,
                        left: self.camera_uniform.offset[0] + 3.0,
                        top: self.camera_uniform.offset[1] + 20.0,
                        scale: 1.0,
                        bounds: TextBounds::default(),
                        default_color: glyphon::Color::rgb(255, 255, 255),
                    },
                    TextArea {
                        buffer: &self.buffer2,
                        left: 50.0,
                        top: 200.0,
                        scale: 1.0,
                        bounds: TextBounds::default(),
                        default_color: glyphon::Color::rgb(255, 255, 255),
                    },
                ],
                &mut self.cache,
            )
            .unwrap();

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
            self.text_renderer
                .render(&self.atlas, &mut render_pass)
                .unwrap();
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.atlas.trim();
        Ok(())
    }

    pub(crate) fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_event(event)
    }

    pub(crate) fn update(&mut self) {
        self.camera_controller
            .update_camera(&mut self.camera, self.size);
        self.camera_uniform = CameraUniform::from(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}
