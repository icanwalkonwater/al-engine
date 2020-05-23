use crate::renderer::device_selection::QueueFamilies;
use crate::renderer::ubo::UniformBufferObject;
use crate::renderer::vertex::{Vertex2DRgb, TRIANGLE_INDICES, TRIANGLE_VERTICES};
use crate::renderer::vulkan_app::VulkanApp;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

#[derive(Copy, Clone)]
pub(super) struct BufferAllocation {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

impl BufferAllocation {
    pub unsafe fn free(self, device: &ash::Device) {
        device.destroy_buffer(self.buffer, None);
        device.free_memory(self.memory, None);
    }
}

impl VulkanApp {
    /// Create framebuffers to receive the output of a render pass.
    pub(super) fn create_framebuffers(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &[vk::ImageView],
        extent: vk::Extent2D,
    ) -> Vec<vk::Framebuffer> {
        image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view];
                let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);

                unsafe {
                    device
                        .create_framebuffer(&framebuffer_create_info, None)
                        .expect("Failed to create Framebuffer !")
                }
            })
            .collect::<Vec<_>>()
    }

    /// Create a command pool used to create command buffers.
    pub(super) fn create_command_pool(
        device: &ash::Device,
        queue_families: &QueueFamilies,
    ) -> vk::CommandPool {
        let command_pool_create_info =
            vk::CommandPoolCreateInfo::builder().queue_family_index(queue_families.graphics);

        unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create command pool")
        }
    }

    /// Create command buffers configured with a pipeline and a render pass.
    pub(super) fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        pipeline: vk::Pipeline,
        framebuffers: &[vk::Framebuffer],
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        pipeline_layout: vk::PipelineLayout,
        descriptor_sets: &[vk::DescriptorSet],
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(framebuffers.len() as u32)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers !")
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

            unsafe {
                device
                    .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                    .expect("Failed to begin recording of Command Buffer !");
            }

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0., 0., 0., 1.],
                },
            }];

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .framebuffer(framebuffers[i])
                .render_pass(render_pass)
                .render_area(
                    vk::Rect2D::builder()
                        .offset(vk::Offset2D::builder().build())
                        .extent(extent)
                        .build(),
                )
                .clear_values(&clear_values);

            unsafe {
                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );

                device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);

                let vertex_buffers = [vertex_buffer];
                let offsets = [0u64];
                let descriptor_sets_to_bind = [descriptor_sets[i]];

                device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                device.cmd_bind_index_buffer(
                    command_buffer,
                    index_buffer,
                    0,
                    vk::IndexType::UINT32,
                );
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0,
                    &descriptor_sets_to_bind,
                    &[],
                );

                device.cmd_draw_indexed(command_buffer, TRIANGLE_INDICES.len() as u32, 1, 0, 0, 0);

                device.cmd_end_render_pass(command_buffer);

                device
                    .end_command_buffer(command_buffer)
                    .expect("Failed to end recording of Command Buffer !");
            }
        }

        command_buffers
    }

    pub(super) fn create_vertex_buffer(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_size = std::mem::size_of_val(&TRIANGLE_VERTICES) as vk::DeviceSize;
        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device_memory_properties,
        );

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Vertex Buffer Memory !")
                as *mut Vertex2DRgb;

            data_ptr.copy_from_nonoverlapping(TRIANGLE_VERTICES.as_ptr(), TRIANGLE_VERTICES.len());

            device.unmap_memory(staging_buffer_memory);
        }

        let (vertex_buffer, vertex_buffer_memory) = Self::create_buffer(
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device_memory_properties,
        );

        Self::copy_buffer(
            device,
            submit_queue,
            command_pool,
            staging_buffer,
            vertex_buffer,
            buffer_size,
        );

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (vertex_buffer, vertex_buffer_memory)
    }

    pub(super) fn create_index_buffer(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_size = std::mem::size_of_val(&TRIANGLE_INDICES) as vk::DeviceSize;
        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device_memory_properties,
        );

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Vertex Buffer Memory !")
                as *mut u32;

            data_ptr.copy_from_nonoverlapping(TRIANGLE_INDICES.as_ptr(), TRIANGLE_INDICES.len());

            device.unmap_memory(staging_buffer_memory);
        }

        let (index_buffer, index_buffer_memory) = Self::create_buffer(
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device_memory_properties,
        );

        Self::copy_buffer(
            device,
            submit_queue,
            command_pool,
            staging_buffer,
            index_buffer,
            buffer_size,
        );

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (index_buffer, index_buffer_memory)
    }

    pub(super) fn create_uniform_buffers(
        device: &ash::Device,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        swapchain_image_count: usize,
    ) -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>) {
        let buffer_size = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;

        (0..swapchain_image_count)
            .map(|_| {
                let (uniform_buffer, uniform_buffer_memory) = Self::create_buffer(
                    device,
                    buffer_size,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                    device_memory_properties,
                );

                (uniform_buffer, uniform_buffer_memory)
            })
            .unzip()
    }

    fn create_buffer(
        device: &ash::Device,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        required_memory_properties: vk::MemoryPropertyFlags,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        // Buffer creation

        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device
                .create_buffer(&buffer_create_info, None)
                .expect("Failed to Create Buffer !")
        };

        // Memory Allocation

        let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory_type = Self::find_memory_type(
            memory_requirements.memory_type_bits,
            required_memory_properties,
            device_memory_properties,
        );

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type);

        let buffer_memory = unsafe {
            device
                .allocate_memory(&allocate_info, None)
                .expect("Failed to Allocate Buffer Memory !")
        };

        // Bind Memory

        unsafe {
            device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to Bind Buffer !");
        }

        (buffer, buffer_memory)
    }

    fn copy_buffer(
        device: &ash::Device,
        submit_queue: vk::Queue,
        command_pool: vk::CommandPool,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: vk::DeviceSize,
    ) {
        // Allocate a Command buffer

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&allocate_info)
                .expect("Failed to allocate Command Buffer !")
        };
        let command_buffer = command_buffers[0];

        // Record commands

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Failed to begin Command Buffer !");
        }

        let copy_regions = [vk::BufferCopy::builder().size(size).build()];

        unsafe {
            device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);
            device
                .end_command_buffer(command_buffer)
                .expect("Failed to end Command Buffer !");
        }

        // Submit commands

        // WARN: lifetimes discarded
        let submit_info = [vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build()];

        unsafe {
            device
                .queue_submit(submit_queue, &submit_info, vk::Fence::null())
                .expect("Failed to Submit Queue !");
            device
                .queue_wait_idle(submit_queue)
                .expect("Failed to wait for Queue Idle");

            device.free_command_buffers(command_pool, &command_buffers);
        }
    }

    fn find_memory_type(
        type_filter: u32,
        required_properties: vk::MemoryPropertyFlags,
        memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> u32 {
        for (i, memory_type) in memory_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << i as u32)) > 0
                && memory_type.property_flags.contains(required_properties)
            {
                return i as u32;
            }
        }

        panic!("Failed to find suitable memory type !");
    }
}
