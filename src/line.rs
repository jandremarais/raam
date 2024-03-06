use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct Vertex {
    pos: [f32; 2],
}

impl Vertex {
    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

pub(crate) const VERTICES: &[Vertex] = &[
    Vertex { pos: [0., 0.] },
    Vertex { pos: [1., 0.] },
    Vertex { pos: [0., 1.] },
    Vertex { pos: [1., 1.] },
];

pub(crate) const INDICES: &[u16] = &[0, 1, 2, 1, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub(crate) struct Instance {
    offset: [f32; 2],
    scale: [f32; 2],
}

impl Instance {
    pub(crate) fn new(offset: (f32, f32), scale: (f32, f32)) -> Self {
        Self {
            offset: [offset.0, offset.1],
            scale: [scale.0, scale.1],
        }
    }

    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
