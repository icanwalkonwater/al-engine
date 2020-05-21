#[cfg(debug_assertions)]
use crate::renderer::debug_utils::REQUIRED_VALIDATION_LAYERS;
use crate::renderer::{API_VERSION, ENGINE_VERSION, WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use crate::APPLICATION_VERSION;
#[cfg(debug_assertions)]
use ash::extensions::ext::DebugUtils;
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
#[cfg(debug_assertions)]
use core::ffi;
use std::ffi::CString;
use winit::event_loop::EventLoop;

pub struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    window: winit::window::Window,
    physical_device: vk::PhysicalDevice,

    #[cfg(debug_assertions)]
    debug_utils_loader: DebugUtils,
    #[cfg(debug_assertions)]
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanApp {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let entry = ash::Entry::new().expect("Failed to acquire Vulkan entry point !");
        let window = Self::create_window(event_loop);
        let instance = Self::create_instance(&entry, &window);
        let physical_device = Self::pick_physical_device(&instance);

        #[cfg(debug_assertions)]
        let (debug_utils_loader, debug_utils_messenger) =
            Self::setup_debug_utils(&entry, &instance);

        Self {
            _entry: entry,
            window,
            instance,
            physical_device,

            #[cfg(debug_assertions)]
            debug_utils_loader,
            #[cfg(debug_assertions)]
            debug_utils_messenger,
        }
    }

    fn create_window(event_loop: &EventLoop<()>) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("Failed to create window")
    }

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
            .api_version(API_VERSION)
            .build();

        // Platform specific extensions to enable
        let extension_names = {
            #[allow(unused_mut)]
            let mut extension_names = ash_window::enumerate_required_extensions(window)
                .expect("Failed to gather required Vulkan extensions !")
                .into_iter()
                .map(|extension| extension.as_ptr())
                .collect::<Vec<*const i8>>();

            // Add the debug extension if requested
            #[cfg(debug_assertions)]
            extension_names.push(DebugUtils::name().as_ptr());

            extension_names
        };

        #[allow(unused_mut)]
        let mut create_info_builder = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        #[cfg(debug_assertions)]
        {
            create_info_builder.p_next = &Self::get_messenger_create_info()
                as *const vk::DebugUtilsMessengerCreateInfoEXT
                as *const ffi::c_void;
        }

        // Create owned storage of raw names
        // !!! Need to be alive until the instance is created
        #[cfg(debug_assertions)]
        let required_layers_raw_names = REQUIRED_VALIDATION_LAYERS
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect::<Vec<CString>>();

        #[cfg(debug_assertions)]
        let required_layers_names = required_layers_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect::<Vec<*const i8>>();

        #[cfg(debug_assertions)]
        let create_info_builder = create_info_builder.enabled_layer_names(&required_layers_names);

        let create_info = create_info_builder.build();

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create Vulkan instance !")
        };

        instance
    }
}

impl VulkanApp {
    pub fn draw_frame(&self) {
        // dbg!()
    }
}

impl VulkanApp {
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            #[cfg(debug_assertions)]
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

            self.instance.destroy_instance(None);
        }
    }
}
