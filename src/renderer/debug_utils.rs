//! This module extends [`VulkanApp`] to implement validation layer hooks and routines.

use crate::renderer::vulkan_app::VulkanApp;
use crate::utils::vk_to_owned_string;
use ash::extensions::ext::DebugUtils;
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
use core::ffi;
use log::{error, info, log_enabled, trace, warn, Level};
use std::ffi::{CStr, CString};

pub(in crate::renderer) const REQUIRED_VALIDATION_LAYERS: [&str; 1] =
    ["VK_LAYER_KHRONOS_validation"];

impl VulkanApp {
    pub(super) fn setup_debug_utils(
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

    pub(super) fn get_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(vulkan_debug_utils_callback))
            .build()
    }

    pub(super) fn check_validation_layer_support(entry: &ash::Entry) -> bool {
        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate instance layer properties !");

        if layer_properties.is_empty() {
            warn!("No available validation layers !");
            false
        } else {
            // Save us the trouble of iterating if we won't see it logged
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

    /// # Access Violation
    /// This methods returns a Vec of owned string that need to stay in scope for as long as
    /// the pointer of the second Vec are in use
    pub(super) fn get_validation_layers_raw_owned() -> (Vec<CString>, Vec<*const i8>) {
        // Create owned storage of raw names
        // !!! Need to be alive until the instance is created
        let required_layers_raw_names = REQUIRED_VALIDATION_LAYERS
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect::<Vec<_>>();

        let required_layers_names = required_layers_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect::<Vec<_>>();

        (required_layers_raw_names, required_layers_names)
    }
}

/// Validation layer logging hook
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

/// Utils to print every useful information about a physical device.
pub(in crate::renderer) fn debug_physical_device(
    instance: &ash::Instance,
    device: vk::PhysicalDevice,
) {
    let properties = unsafe { instance.get_physical_device_properties(device) };
    let features = unsafe { instance.get_physical_device_features(device) };
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };

    let name = vk_to_owned_string(&properties.device_name);
    let device_type = match properties.device_type {
        vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
        vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
        vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
        vk::PhysicalDeviceType::CPU => "CPU",
        _ => "Other",
    };

    trace!(
        "Device: {}, id: {:x}:{:x}, type: {}",
        name,
        properties.vendor_id,
        properties.device_id,
        device_type
    );

    trace!("\tQueue families: {}", queue_families.len());
    trace!("\tIndex | Queue Count | Graphics | Compute | Transfer | Sparse Binding");
    for (index, family) in queue_families.iter().enumerate() {
        let graphics = family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
        let compute = family.queue_flags.contains(vk::QueueFlags::COMPUTE);
        let transfer = family.queue_flags.contains(vk::QueueFlags::TRANSFER);
        let sparse = family.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING);

        trace!(
            "\t{:^5} | {:^11} | {:^8} | {:^7} | {:^8} | {:^14}",
            index,
            family.queue_count,
            graphics,
            compute,
            transfer,
            sparse
        );
    }

    trace!("\tGeometry shader: {}", features.geometry_shader != 0);
    trace!(
        "\tTesselation shader: {}",
        features.tessellation_shader != 0
    );
}
