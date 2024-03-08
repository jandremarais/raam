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

    let event_loop = EventLoop::new().expect("EventLoop failed");
    let window = WindowBuilder::new()
        .with_title("raam")
        .with_inner_size(winit::dpi::PhysicalSize::new(512, 512))
        .build(&event_loop)
        .expect("WindowBuilder failed");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut state = pollster::block_on(State::new(&window));
    event_loop
        .run(|event, elwt| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                        WindowEvent::ScaleFactorChanged { .. } => state.resize(window.inner_size()),
                        WindowEvent::RedrawRequested => {
                            state.update();
                            let start = Instant::now();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                            println!("render took: {:?}", start.elapsed());
                        }
                        _ => {}
                    }
                } else {
                    // TODO!: is this the right place to put it. Will an input event always need to trigger it?
                    window.request_redraw();
                }
            }
            _ => {}
        })
        .expect("event loop failed");
}
