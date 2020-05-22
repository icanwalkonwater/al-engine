use crate::renderer::vulkan_app::VulkanApp;
use crate::renderer::MAX_FRAMES_IN_FLIGHT;
use ash::version::DeviceV1_0;
use ash::vk;

#[derive(Default)]
pub struct SyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
}

impl VulkanApp {
    /// Create semaphores and fences used to synchronize the rendering steps.
    pub(super) fn create_sync_objects(device: &ash::Device) -> SyncObjects {
        let mut sync_objects = SyncObjects::default();

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();

        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore !");

                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore !");

                let inflight_fence = device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create Fence !");

                sync_objects
                    .image_available_semaphores
                    .push(image_available_semaphore);
                sync_objects
                    .render_finished_semaphores
                    .push(render_finished_semaphore);
                sync_objects.inflight_fences.push(inflight_fence);
            }
        }

        sync_objects
    }
}
