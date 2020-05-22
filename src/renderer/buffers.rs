use crate::renderer::device_selection::QueueFamilies;
use crate::renderer::vulkan_app::VulkanApp;
use ash::version::DeviceV1_0;
use ash::vk;

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
                device.cmd_draw(command_buffer, 3, 1, 0, 0);
                device.cmd_end_render_pass(command_buffer);

                device
                    .end_command_buffer(command_buffer)
                    .expect("Failed to end recording of Command Buffer !");
            }
        }

        command_buffers
    }
}
