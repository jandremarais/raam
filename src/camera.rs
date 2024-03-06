use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, Device};
use winit::event::{MouseScrollDelta, WindowEvent};

#[derive(Default)]
pub(crate) struct Camera {
    x: f32,
    y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct CameraUniform {
    pub(crate) offset: [f32; 2],
}

impl From<&Camera> for CameraUniform {
    fn from(value: &Camera) -> Self {
        CameraUniform {
            offset: [value.x, value.y],
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

#[derive(Default)]
pub(crate) struct CameraController {
    speed: f32,
    delta_x: f32,
    delta_y: f32,
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
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    self.delta_x = *x;
                    self.delta_y = *y;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub(crate) fn update_camera(&self, camera: &mut Camera) {
        camera.x += self.delta_x * self.speed;
        camera.y += self.delta_y * self.speed;
    }
}
