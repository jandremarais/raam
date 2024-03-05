use winit::{
    event::{Event, WindowEvent},
    window::Window,
};

pub(crate) struct State<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
}

impl<'a> State<'a> {
    pub(crate) async fn new(window: &'a Window) -> Self {
        todo!()
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        todo!()
    }

    pub(crate) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        todo!()
    }

    pub(crate) fn input(&mut self, event: &WindowEvent) -> bool {
        todo!()
    }

    pub(crate) fn update(&mut self) {
        todo!()
    }
}
