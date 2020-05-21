//! This module extends [`VulkanApp`] to implement physical device scoring and selection.

#[cfg(debug_assertions)]
use crate::renderer::debug_utils::debug_physical_device;
use crate::renderer::vulkan_app::{SurfaceContainer, VulkanApp};
use crate::renderer::REQUIRED_DEVICE_EXTENSIONS;
use crate::utils::vk_to_owned_string;
use ash::version::InstanceV1_0;
use ash::vk;
use log::{info, trace};
use std::collections::HashSet;

pub struct QueueFamilies {
    pub graphics: u32,
    pub presentation: u32,
}

#[derive(Default)]
struct QueueFamiliesBuilder {
    graphics: Option<usize>,
    graphics_score: u32,
    presentation: Option<usize>,
    presentation_score: u32,
}

impl QueueFamiliesBuilder {
    fn rank_family(family: &vk::QueueFamilyProperties) -> u32 {
        let mut score = 0;
        // Note: a family with 0 queues need to have a score of 0
        score += family.queue_count * 100;
        // TODO: take into account the min_image_transfer_granularity
        score
    }

    fn try_graphics(&mut self, index: usize, family: &vk::QueueFamilyProperties) {
        let score = Self::rank_family(family);
        if score > self.graphics_score {
            self.graphics_score = score;
            self.graphics = Some(index);
        }
    }

    fn try_presentation(&mut self, index: usize, family: &vk::QueueFamilyProperties) {
        let score = Self::rank_family(family);
        if score > self.presentation_score {
            self.presentation_score = score;
            self.presentation = Some(index);
        }
    }

    fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.presentation.is_some()
    }

    fn build(&self) -> Option<QueueFamilies> {
        if self.is_complete() {
            Some(QueueFamilies {
                graphics: self.graphics.unwrap() as u32,
                presentation: self.presentation.unwrap() as u32,
            })
        } else {
            None
        }
    }
}

impl VulkanApp {
    pub(in crate::renderer) fn pick_physical_device(
        instance: &ash::Instance,
        surface: &SurfaceContainer,
    ) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices !")
        };

        let (physical_device, _) = physical_devices
            .into_iter()
            .filter_map(|device| {
                #[cfg(debug_assertions)]
                debug_physical_device(instance, device);

                if let Some(score) =
                    Self::check_suitability_and_score_device(instance, device, &surface)
                {
                    Some((device, score))
                } else {
                    None
                }
            })
            .max_by_key(|(_, score)| *score)
            .expect("Failed to pick a physical device !");

        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        info!(
            "Device {} chosen.",
            vk_to_owned_string(&properties.device_name)
        );

        physical_device
    }

    fn check_suitability_and_score_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &SurfaceContainer,
    ) -> Option<u32> {
        let is_queue_family_supported =
            Self::find_queue_families(instance, physical_device, &surface).is_some();

        let is_device_extension_supported =
            Self::check_device_extension_support(instance, physical_device);

        let is_swapchain_supported = if is_device_extension_supported {
            let swapchain_support = Self::query_swapchain_support(physical_device, surface);
            !swapchain_support.formats.is_empty()
                && !swapchain_support.presentation_modes.is_empty()
        } else {
            false
        };

        if is_queue_family_supported && is_device_extension_supported && is_swapchain_supported {
            Some(Self::score_device(instance, physical_device))
        } else {
            None
        }
    }

    fn score_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> u32 {
        let mut score = 0;

        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };

        score += match device_properties.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
            vk::PhysicalDeviceType::VIRTUAL_GPU => 500,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
            _ => 0,
        };

        // TODO: score also with memory size

        score
    }

    pub(in crate::renderer) fn find_queue_families(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_container: &SurfaceContainer,
    ) -> Option<QueueFamilies> {
        let mut families_builder = QueueFamiliesBuilder::default();

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        for (index, family) in queue_families.iter().enumerate() {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                families_builder.try_graphics(index, family);
            }

            let is_presentation_supported = unsafe {
                surface_container
                    .surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface_container.surface,
                    )
                    .expect("Failed to query surface support")
            };

            if family.queue_count > 0 && is_presentation_supported {
                families_builder.try_presentation(index, family);
            }

            if families_builder.is_complete() {
                break;
            }
        }

        families_builder.build()
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .expect("Failed to enumerate device extension properties !")
        };

        #[cfg(debug_assertions)]
        trace!("Available extensions: {}", available_extensions.len());

        let available_extensions_names = available_extensions
            .into_iter()
            .map(|extension| {
                let name = vk_to_owned_string(&extension.extension_name);
                #[cfg(debug_assertions)]
                trace!("- {} v{}", name, extension.spec_version);

                name
            })
            .collect::<Vec<_>>();

        let mut required_extensions = HashSet::with_capacity(REQUIRED_DEVICE_EXTENSIONS.len());
        for ext in &REQUIRED_DEVICE_EXTENSIONS {
            required_extensions.insert(ext.to_string());
        }

        for extension in available_extensions_names {
            required_extensions.remove(&extension);
        }

        required_extensions.is_empty()
    }
}
