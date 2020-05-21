//! This module extends `VulkanApp` to implement physical device scoring and selection.

use ash::version::InstanceV1_0;
use ash::vk;
use crate::renderer::vulkan_app::VulkanApp;
#[cfg(debug_assertions)]
use crate::renderer::debug_utils::debug_physical_device;

pub struct QueueFamilies {
    pub graphics: usize,
}

#[derive(Default)]
struct QueueFamiliesBuilder {
    graphics: Option<usize>,
    graphics_score: u32,
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

    fn is_complete(&self) -> bool {
        self.graphics.is_some()
    }

    fn build(&self) -> Option<QueueFamilies> {
        if self.is_complete() {
            Some(QueueFamilies {
                graphics: self.graphics.unwrap(),
            })
        } else {
            None
        }
    }
}

impl VulkanApp {
    pub(in crate::renderer) fn pick_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
        let devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices !")
        };

        let (device, _) = devices
            .into_iter()
            .filter_map(|device| {
                #[cfg(debug_assertions)]
                debug_physical_device(instance, device);
                if let Some(score) = Self::check_and_score_device(instance, &device) {
                    Some((device, score))
                } else {
                    None
                }
            })
            .max_by_key(|(_, score)| *score)
            .expect("Failed to pick a physical device !");

        device
    }

    fn check_and_score_device(instance: &ash::Instance, device: &vk::PhysicalDevice) -> Option<u32> {
        let _families = {
            let families = Self::find_queue_families(instance, device);
            if families.is_none() {
                return None;
            }
            families.unwrap()
        };

        // TODO: check extensions

        Some(Self::score_device(instance, device))
    }

    fn score_device(instance: &ash::Instance, device: &vk::PhysicalDevice) -> u32 {
        let mut score = 0;

        let device_properties = unsafe { instance.get_physical_device_properties(*device) };

        score += match device_properties.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
            vk::PhysicalDeviceType::VIRTUAL_GPU => 500,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
            _ => 0,
        };

        // TODO: score also with memory size

        score
    }

    fn find_queue_families(
        instance: &ash::Instance,
        device: &vk::PhysicalDevice,
    ) -> Option<QueueFamilies> {
        let mut families_builder = QueueFamiliesBuilder::default();

        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(*device) };

        for (id, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                families_builder.try_graphics(id, queue_family);
            }

            if families_builder.is_complete() {
                break;
            }
        }

        families_builder.build()
    }
}
