use crate::renderer::VulkanApplication;
use log::info;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

pub struct Application {
    vulkan_app: VulkanApplication,
    event_loop: EventLoop<()>,
}

impl Application {
    pub fn new() -> Self {
        let (vulkan_app, event_loop) = VulkanApplication::new_with_event_loop();

        Self {
            vulkan_app,
            event_loop,
        }
    }

    /// Run until the application closes.
    /// The application is thus consumed.
    pub fn main_loop(self) {
        let mut vulkan_app = self.vulkan_app;

        self.event_loop.run(move |event, _, control_flow| {
            // Continuously run the loop without waiting for an event
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    info!("Close requested, stopping");
                    *control_flow = ControlFlow::Exit;
                }
                Event::MainEventsCleared => {
                    // TODO: Update scene and stuff

                    // And request a draw
                    vulkan_app.window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    // Redraw
                    vulkan_app.draw_frame();
                }
                _ => (),
            }
        });
    }
}
