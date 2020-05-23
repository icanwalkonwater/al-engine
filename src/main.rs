use al_engine::fps_limiter::FpsLimiter;
use al_engine::renderer::vulkan_app::VulkanApp;
use log::{warn, LevelFilter};
use simplelog::{Config, SimpleLogger, TermLogger, TerminalMode};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use al_engine::FPS_LIMIT;

fn main() {
    setup_logger();

    let event_loop = EventLoop::new();
    let vulkan_app = VulkanApp::new(&event_loop);

    main_loop(event_loop, vulkan_app);
}

fn setup_logger() {
    TermLogger::init(LevelFilter::max(), Config::default(), TerminalMode::Mixed).unwrap_or_else(
        |_| {
            SimpleLogger::init(LevelFilter::max(), Config::default())
                .expect("Failed to setup a logger");
            warn!("Failed to setup TermLogger, falled back to SimpleLogger.");
        },
    );
}

fn main_loop(event_loop: EventLoop<()>, mut vulkan_app: VulkanApp) {
    let mut limiter = FpsLimiter::new(FPS_LIMIT);
    limiter.tick();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    virtual_keycode,
                    state,
                    ..
                } => match (virtual_keycode, state) {
                    (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                        dbg!("Exit");
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                },
            },
            _ => {}
        },
        Event::MainEventsCleared => {
            vulkan_app.window().request_redraw();
        }
        Event::RedrawRequested(_) => {
            limiter.tick();
            let delta_time = limiter.delta_time();
            vulkan_app.draw_frame(delta_time);

            #[cfg(debug_assertions)]
            print!("\rFPS: {}   ", (1. / delta_time) as u32);
        }
        _ => {}
    });
}
