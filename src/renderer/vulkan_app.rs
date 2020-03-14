use crate::renderer::physical_device_selection::{
    find_queue_families, pick_physical_device, required_extensions,
};
use crate::renderer::{APPLICATION_NAME, DIMENSIONS, ENABLE_VALIDATION_LAYERS, VALIDATION_LAYERS};
use log::{error, info, trace, warn};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::sync::Arc;
use vulkano::device::{Device, Features, Queue};
use vulkano::instance::debug::{DebugCallback, MessageSeverity, MessageType};
use vulkano::instance::{
    layers_list, ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice, Version,
};
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct VulkanApplication {
    instance: Arc<Instance>,
    #[cfg(debug_assertions)]
    debug_callback: DebugCallback,

    surface: Arc<Surface<Window>>,
    physical_device_id: usize,

    device: Arc<Device>,

    graphics_queue: Arc<Queue>,
    presentation_queue: Arc<Queue>,
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

        // Create device
        let physical_device_id = pick_physical_device(&instance, &surface).index();
        let (device, graphics_queue, presentation_queue) =
            Self::create_logical_device(&instance, &surface, physical_device_id);

        (
            Self {
                instance,
                #[cfg(debug_assertions)]
                debug_callback,
                surface,
                physical_device_id,
                device,
                graphics_queue,
                presentation_queue,
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
            .with_inner_size(LogicalSize::new(
                f64::from(DIMENSIONS.0),
                f64::from(DIMENSIONS.1),
            ))
            .build_vk_surface(&event_loop, instance.clone())
            .expect("Failed to create window surface !");

        (event_loop, surface)
    }

    fn create_logical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
        physical_device_index: usize,
    ) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
        trace!("Creating logical device");

        let physical_device = PhysicalDevice::from_index(&instance, physical_device_index).unwrap();
        let indices = find_queue_families(surface, &physical_device).unwrap();

        let families = [indices.graphics, indices.presentation];
        let unique_queue_families: HashSet<&u32> = HashSet::from_iter(families.iter());

        let queue_priority = 1.0;
        let queue_families = unique_queue_families.into_iter().map(|i| {
            (
                physical_device.queue_family_by_id(*i).unwrap(),
                queue_priority,
            )
        });

        let (device, mut queues) = Device::new(
            physical_device,
            &Features::none(),
            &required_extensions(),
            queue_families,
        )
        .expect("Failed to create logical device !");

        let graphics_queue = queues.next().unwrap();
        let presentation_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());

        (device, graphics_queue, presentation_queue)
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
