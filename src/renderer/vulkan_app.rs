use crate::renderer::device_selection::pick_physical_device;
use crate::renderer::{APPLICATION_NAME, DIMENSIONS, ENABLE_VALIDATION_LAYERS, VALIDATION_LAYERS};
use std::sync::Arc;
use vulkano::instance::{layers_list, ApplicationInfo, Instance, InstanceExtensions, Version};
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::platform::unix::WindowBuilderExtUnix;
use winit::window::{Window, WindowBuilder};
use vulkano::instance::debug::DebugCallback;

pub struct VulkanApplication {
    instance: Arc<Instance>,
    #[cfg(debug_assertions)]
    debug_callback: DebugCallback,

    surface: Arc<Surface<Window>>,
}

impl VulkanApplication {
    pub fn new_with_event_loop() -> (Self, EventLoop<()>) {
        trace!("Creating vulkan app");

        // Create Vulkan instance, the entry point of vulkan
        let instance = Self::create_instance();

        // If in debug mode, create the handler for the validation layers
        #[cfg(debug_assertions)]
        let debug_callback = Self::setup_debug_callback(&instance);

        // Create the surface
        let (event_loop, surface) = Self::create_surface(&instance);

        // Pick a physical device
        let physical_device = pick_physical_device(&instance, &surface);

        (
            Self {
                instance,
                #[cfg(debug_assertions)]
                debug_callback,
                surface,
            },
            event_loop,
        )
    }

    pub fn window(&self) -> &Window {
        return self.surface.window();
    }

    pub fn draw_frame(&mut self) {}

    fn create_instance() -> Arc<Instance> {
        if ENABLE_VALIDATION_LAYERS && !check_validation_layer_support() {
            warn!("Validation layers requested, but not available !");
        }

        let supported_extensions = InstanceExtensions::supported_by_core()
            .expect("Failed to retrieve supported extensions !");
        trace!("Supported extensions: {:?}", supported_extensions);

        let application_info = ApplicationInfo {
            application_name: Some(APPLICATION_NAME.into()),
            application_version: Some(Version {
                major: env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap(),
                minor: env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap(),
                patch: env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap(),
            }),
            engine_name: Some("Vulkan".into()),
            engine_version: Some(Version {
                major: 1,
                minor: 0,
                patch: 0,
            }),
        };

        let required_extensions = get_required_extensions();

        if ENABLE_VALIDATION_LAYERS && check_validation_layer_support() {
            Instance::new(
                Some(&application_info),
                &required_extensions,
                VALIDATION_LAYERS.iter().cloned(),
            )
            .expect("Failed to create Vulkan instance !")
        } else {
            Instance::new(Some(&application_info), &required_extensions, None)
                .expect("Failed to create Vulkan instance !")
        }
    }

    fn create_surface(instance: &Arc<Instance>) -> (EventLoop<()>, Arc<Surface<Window>>) {
        trace!("Creating VK Surface");

        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .with_title("AL-Engine")
            .with_base_size(LogicalSize::new(
                f64::from(DIMENSIONS.0),
                f64::from(DIMENSIONS.1),
            ))
            .build_vk_surface(&event_loop, instance.clone())
            .expect("Failed to create window surface !");

        (event_loop, surface)
    }

    #[cfg(debug_assertions)]
    fn setup_debug_callback(instance: &Arc<Instance>) -> DebugCallback {
        trace!("Setting up validation layer callback");

        let msg_types = MessageType {
            general: true,
            performance: true,
            validation: true,
        };

        let severities = MessageSeverity {
            error: true,
            warning: true,
            information: true,
            verbose: true,
        };

        DebugCallback::new(&instance, severities, msg_types, |msg| {
            if msg.severity.error {
                error!(
                    "[Validation Layer] [{}] {}",
                    msg.layer_prefix, msg.description
                );
            } else if msg.severity.warning {
                warn!(
                    "[Validation Layer] [{}] {}",
                    msg.layer_prefix, msg.description
                );
            } else if msg.severity.information {
                info!(
                    "[Validation Layer] [{}] {}",
                    msg.layer_prefix, msg.description
                );
            } else {
                trace!(
                    "[Validation Layer] [{}] {}",
                    msg.layer_prefix,
                    msg.description
                );
            }
        })
        .expect("Failed to create debug callback")
    }
}

fn check_validation_layer_support() -> bool {
    let layers: Vec<_> = layers_list()
        .unwrap()
        .map(|l| l.name().to_owned())
        .collect();

    VALIDATION_LAYERS
        .iter()
        .all(|layer_name| layers.contains(&layer_name.to_string()))
}

fn get_required_extensions() -> InstanceExtensions {
    let mut extensions = vulkano_win::required_extensions();
    if ENABLE_VALIDATION_LAYERS {
        extensions.ext_debug_utils = true;
    }

    extensions
}
