use crate::renderer::vulkan_app::VulkanApp;
use crate::utils::vk_to_owned_string;
use ash::extensions::ext::DebugUtils;
use ash::version::EntryV1_0;
use ash::vk;
use core::ffi;
use log::{error, info, log_enabled, trace, warn, Level};
use std::ffi::CStr;

pub(in crate::renderer) const REQUIRED_VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

impl VulkanApp {
    pub(in crate::renderer) fn setup_debug_utils(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> (DebugUtils, vk::DebugUtilsMessengerEXT) {
        let debug_utils_loader = DebugUtils::new(entry, instance);

        let utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&Self::get_messenger_create_info(), None)
                .expect("Failed to create debug util messenger")
        };

        (debug_utils_loader, utils_messenger)
    }

    pub(in crate::renderer) fn get_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(vulkan_debug_utils_callback))
            .build()
    }

    pub(in crate::renderer) fn check_validation_layer_support(entry: &ash::Entry) -> bool {
        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate instance layer properties !");

        if layer_properties.is_empty() {
            warn!("No available validation layers !");
            false
        } else {
            if log_enabled!(Level::Trace) {
                trace!("Available validation layers:");
                for layer in layer_properties.iter() {
                    let layer_name = vk_to_owned_string(&layer.layer_name);
                    trace!(" - {}", layer_name);
                }
            }

            for required_layer in REQUIRED_VALIDATION_LAYERS.iter() {
                let mut found = false;

                for layer in layer_properties.iter() {
                    let layer_name = vk_to_owned_string(&layer.layer_name);
                    if &layer_name == required_layer {
                        found = true;
                        break;
                    }
                }

                if !found {
                    return false;
                }
            }

            true
        }
    }
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut ffi::c_void,
) -> vk::Bool32 {
    let message_type = match message_types {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "General",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "Performance",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "Validation",
        _ => "Unknown",
    };

    let message = CStr::from_ptr((*p_callback_data).p_message);

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            trace!("[Validation Layer] [{}]: {:?}", message_type, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            info!("[Validation Layer] [{}]: {:?}", message_type, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            warn!("[Validation Layer] [{}]: {:?}", message_type, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            error!("[Validation Layer] [{}]: {:?}", message_type, message)
        }
        _ => {
            error!(
                "*Unknown severity* [Validation Layer] [{}]: {:?}",
                message_type, message
            );
        }
    };

    vk::FALSE
}
