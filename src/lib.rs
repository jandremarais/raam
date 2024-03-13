use std::time::Instant;

use state::State;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
mod camera;
mod grid;
mod line;
mod state;

pub fn run() {
    env_logger::init();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let event_loop = EventLoop::new().expect("EventLoop failed");
    let window = WindowBuilder::new()
        .with_title("raam")
        .with_inner_size(winit::dpi::PhysicalSize::new(512, 512))
        .build(&event_loop)
        .expect("WindowBuilder failed");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut state = pollster::block_on(State::new(&window, instance));
    event_loop
        .run(|event, elwt| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if state.input(event) {
                    window.request_redraw();
                } else {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                        WindowEvent::ScaleFactorChanged { .. } => state.resize(window.inner_size()),
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        })
        .expect("event loop failed");
}
