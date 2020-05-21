use crate::renderer::device_selection::QueueFamilies;
use crate::renderer::swapchain::SwapchainContainer;
use crate::renderer::{ENGINE_VERSION, VULKAN_VERSION, WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use crate::APPLICATION_VERSION;
#[cfg(debug_assertions)]
use ash::extensions::ext::DebugUtils;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
#[cfg(debug_assertions)]
use core::ffi;
use std::collections::HashSet;
use std::ffi::CString;
use winit::event_loop::EventLoop;

pub struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    window: winit::window::Window,

    surface_container: SurfaceContainer,

    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    presentation_queue: vk::Queue,

    swapchain_container: SwapchainContainer,
    image_views: Vec<vk::ImageView>,

    #[cfg(debug_assertions)]
    debug_utils_loader: DebugUtils,
    #[cfg(debug_assertions)]
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
}

pub(in crate::renderer) struct SurfaceContainer {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
}

// Setup methods
impl VulkanApp {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let entry = ash::Entry::new().expect("Failed to acquire Vulkan entry point !");
        let window = Self::create_window(event_loop);
        let instance = Self::create_instance(&entry, &window);

        let surface_container = Self::create_surface(&entry, &instance, &window);

        let physical_device = Self::pick_physical_device(&instance, &surface_container);
        let (device, indices) =
            Self::create_logical_device(&instance, physical_device, &surface_container);

        let graphics_queue = unsafe { device.get_device_queue(indices.graphics, 0) };
        let presentation_queue = unsafe { device.get_device_queue(indices.presentation, 0) };

        let swapchain_container = Self::create_swapchain(
            &instance,
            &device,
            physical_device,
            &surface_container,
            &indices,
        );

        let image_views = Self::create_image_views(
            &device,
            swapchain_container.swapchain_format,
            &swapchain_container.swapchain_images,
        );

        let graphics_pipeline = Self::create_graphics_pipeline();

        #[cfg(debug_assertions)]
        let (debug_utils_loader, debug_utils_messenger) =
            Self::setup_debug_utils(&entry, &instance);

        Self {
            _entry: entry,
            window,
            instance,

            surface_container,

            physical_device,
            device,
            graphics_queue,
            presentation_queue,

            swapchain_container,
            image_views,

            #[cfg(debug_assertions)]
            debug_utils_loader,
            #[cfg(debug_assertions)]
            debug_utils_messenger,
        }
    }

    /// Create a [`winit::window::Window`].
    fn create_window(event_loop: &EventLoop<()>) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("Failed to create window")
    }

    /// Create a Vulkan instance.
    fn create_instance(entry: &ash::Entry, window: &winit::window::Window) -> ash::Instance {
        #[cfg(debug_assertions)]
        {
            if !Self::check_validation_layer_support(entry) {
                panic!("Validation layers requested, but not available !");
            }
        }

        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("AL Engine").unwrap();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .engine_name(&engine_name)
            .application_version(APPLICATION_VERSION)
            .engine_version(ENGINE_VERSION)
            .api_version(VULKAN_VERSION)
            .build();

        // Platform specific extensions to enable
        let extension_names = {
            #[allow(unused_mut)]
            let mut extension_names = ash_window::enumerate_required_extensions(window)
                .expect("Failed to gather required Vulkan extensions !")
                .into_iter()
                .map(|extension| extension.as_ptr())
                .collect::<Vec<_>>();

            // Add the debug extension if requested
            #[cfg(debug_assertions)]
            extension_names.push(DebugUtils::name().as_ptr());

            extension_names
        };

        // !!! _required_layers_raw_names contains owned data that need to stay in scope until the instance is created !
        #[cfg(debug_assertions)]
        let (_required_layers_raw_names, required_layers_names) =
            Self::get_validation_layers_raw_owned();

        let create_info = {
            #[allow(unused_mut)]
            let mut builder = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_extension_names(&extension_names);

            #[cfg(debug_assertions)]
            {
                builder.p_next = &Self::get_messenger_create_info()
                    as *const vk::DebugUtilsMessengerCreateInfoEXT
                    as *const ffi::c_void;
            }

            #[cfg(debug_assertions)]
            let builder = builder.enabled_layer_names(&required_layers_names);

            builder.build()
        };

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create Vulkan instance !")
        };

        instance
    }

    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> SurfaceContainer {
        let surface = unsafe { ash_window::create_surface(entry, instance, window, None) }
            .expect("Failed to create surface !");

        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        SurfaceContainer {
            surface_loader,
            surface,
        }
    }

    /// Create the logical device and queues from a physical device.
    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &SurfaceContainer,
    ) -> (ash::Device, QueueFamilies) {
        // We can unwrap safely
        let indices = Self::find_queue_families(instance, physical_device, surface).unwrap();

        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics);
        unique_queue_families.insert(indices.presentation);

        let queue_priorities = [1.0f32];
        let mut queue_create_infos = Vec::new();
        for family in unique_queue_families {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(family)
                    .queue_priorities(&queue_priorities)
                    .build(),
            )
        }

        // TODO: Add features/extensions here
        let features_to_enable = vk::PhysicalDeviceFeatures::builder().build();
        let enable_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];

        #[cfg(debug_assertions)]
        let (_required_layers_raw_names, required_layers_names) =
            Self::get_validation_layers_raw_owned();

        let device_create_info = {
            let builder = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_create_infos)
                .enabled_features(&features_to_enable)
                .enabled_extension_names(&enable_extensions);

            #[cfg(debug_assertions)]
            let builder = builder.enabled_layer_names(&required_layers_names);

            builder.build()
        };

        // Create the device
        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical device !")
        };

        (device, indices)
    }

    fn create_graphics_pipeline() {
        // TODO
    }
}

// Drawing methods
impl VulkanApp {
    pub fn draw_frame(&self) {
        // dbg!()
    }
}

// Accessors/Mutators
impl VulkanApp {
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            for &image_view in &self.image_views {
                self.device.destroy_image_view(image_view, None);
            }

            self.swapchain_container
                .swapchain_loader
                .destroy_swapchain(self.swapchain_container.swapchain, None);

            self.device.destroy_device(None);
            self.surface_container
                .surface_loader
                .destroy_surface(self.surface_container.surface, None);

            #[cfg(debug_assertions)]
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

            self.instance.destroy_instance(None);
        }
    }
}
