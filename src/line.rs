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

pub(crate) enum Line {
    Horizontal(f32),
    Vertical(f32),
}

impl Line {
    pub(crate) fn vertices(&self, width: f32, limits: (f32, f32)) -> Vec<Vertex> {
        match self {
            Self::Horizontal(y) => vec![
                Vertex {
                    pos: [limits.0, *y],
                },
                Vertex {
                    pos: [limits.1, *y],
                },
                Vertex {
                    pos: [limits.0, *y - width],
                },
                // Vertex {
                //     pos: [limits.1, *y],
                // },
                // Vertex {
                //     pos: [limits.0, *y - width],
                // },
                Vertex {
                    pos: [limits.1, *y - width],
                },
            ],
            Self::Vertical(x) => vec![],
        }
    }

    pub(crate) fn indices() -> Vec<u16> {
        vec![0, 1, 2, 1, 2, 3]
    }
}
