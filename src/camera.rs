use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, Device};
use winit::{
    dpi::PhysicalSize,
    event::{MouseScrollDelta, WindowEvent},
};

#[derive(Default)]
pub(crate) struct Camera {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub(crate) struct CameraUniform {
    pub(crate) offset: [f32; 2],
    pub(crate) size: [f32; 2],
}

impl From<&Camera> for CameraUniform {
    fn from(value: &Camera) -> Self {
        CameraUniform {
            offset: [value.x, value.y],
            size: [value.width, value.height],
        }
    }
}

impl CameraUniform {
    pub(crate) fn to_buffer(self, device: &Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&[self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub(crate) fn bind_group_layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct CameraController {
    pub(crate) speed: f32,
    pub(crate) delta_x: f32,
    pub(crate) delta_y: f32,
}

impl CameraController {
    pub(crate) fn new(speed: f32) -> Self {
        Self {
            speed,
            ..Default::default()
        }
    }

    pub(crate) fn process_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(x, y),
                ..
            } => {
                self.delta_x += *x;
                self.delta_y += *y;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn update_camera(&mut self, camera: &mut Camera, size: PhysicalSize<u32>) {
        camera.x += self.delta_x.powi(2) * self.speed * self.delta_x.signum();
        camera.y += self.delta_y.powi(2) * self.speed * self.delta_y.signum();
        camera.width = size.width as f32;
        camera.height = size.height as f32;
        self.delta_x = 0.0;
        self.delta_y = 0.0;
    }
}
