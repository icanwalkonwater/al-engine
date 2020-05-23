use crate::renderer::vulkan_app::VulkanApp;
use ash::version::DeviceV1_0;
use ash::vk;
use nalgebra::{IsometryMatrix3, Matrix4, Perspective3, Point3, Vector3};

#[repr(C)]
#[derive(Debug, Clone)]
pub(super) struct UniformBufferObject {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

impl VulkanApp {
    pub(super) fn create_ubo(extent: vk::Extent2D) -> UniformBufferObject {
        UniformBufferObject {
            model: Matrix4::identity(),
            view: IsometryMatrix3::look_at_rh(
                &Point3::new(2., 2., 2.),
                &Point3::origin(),
                &Vector3::z(),
            )
            .to_homogeneous(),
            proj: Perspective3::new(extent.width as f32 / extent.height as f32, 45., 0.1, 10.)
                .to_homogeneous(),
        }
    }

    pub(super) fn create_description_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
        let ubo_layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build()];

        let ubo_layout_create_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&ubo_layout_bindings);

        unsafe {
            device
                .create_descriptor_set_layout(&ubo_layout_create_info, None)
                .expect("Failed to create Descriptor Set Layout")
        }
    }

    pub(crate) fn create_descriptor_pool(
        device: &ash::Device,
        swapchain_images_size: usize,
    ) -> vk::DescriptorPool {
        let pool_sizes = [vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(swapchain_images_size as u32)
            .build()];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(swapchain_images_size as u32)
            .pool_sizes(&pool_sizes);

        unsafe {
            device
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to Create Descriptor Pool")
        }
    }

    pub(crate) fn create_descriptor_sets(
        device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &[vk::Buffer],
        swapchain_images_size: usize,
    ) -> Vec<vk::DescriptorSet> {
        let layouts = vec![descriptor_set_layout; swapchain_images_size];

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = unsafe {
            device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to Allocate Descriptor Sets")
        };

        for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
            // Can be safely built, no references
            let descriptor_buffer_info = [vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i])
                .offset(0)
                .range(std::mem::size_of::<UniformBufferObject>() as u64)
                .build()];

            // WARN: lifetimes lost
            let descriptor_write_sets = [vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&descriptor_buffer_info)
                .build()];

            unsafe {
                device.update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }

        descriptor_sets
    }
}
