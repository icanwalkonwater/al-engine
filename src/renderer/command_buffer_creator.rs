use crate::errors::*;
use crate::renderer::allocation::BufferAllocation;
use crate::renderer::render_object::RenderObject;
use ash::version::DeviceV1_0;
use ash::vk;
use std::borrow::Borrow;

pub(super) struct OneTimeCommandBuffer<'a> {
    device: &'a ash::Device,
    command_buffer: vk::CommandBuffer,
}

impl<'a> OneTimeCommandBuffer<'a> {
    #[inline]
    fn begin(device: &'a ash::Device, command_buffer: vk::CommandBuffer) -> Result<Self> {
        unsafe {
            device.begin_command_buffer(
                command_buffer,
                &vk::CommandBufferBeginInfo::builder()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;
        }

        Ok(Self {
            device,
            command_buffer,
        })
    }
}

impl OneTimeCommandBuffer<'_> {
    #[inline]
    pub fn copy(&self, src: &BufferAllocation, dst: &BufferAllocation, size: vk::DeviceSize) {
        let copy_op = [vk::BufferCopy::builder()
            .size(size)
            .src_offset(src.allocation_info().get_offset() as vk::DeviceSize)
            .dst_offset(dst.allocation_info().get_offset() as vk::DeviceSize)
            .build()];

        unsafe {
            self.device
                .cmd_copy_buffer(self.command_buffer, src.buffer, dst.buffer, &copy_op);
        }
    }

    #[inline]
    pub fn finish(self) -> vk::CommandBuffer {
        unsafe {
            self.device.end_command_buffer(self.command_buffer).unwrap();
        }
        self.command_buffer
    }
}

pub(super) struct DrawingCommandBuffer<'a> {
    device: &'a ash::Device,
    command_buffer: vk::CommandBuffer,
}

impl<'a> DrawingCommandBuffer<'a> {
    #[inline]
    pub fn begin(device: &'a ash::Device, command_buffer: vk::CommandBuffer) -> Result<Self> {
        unsafe {
            device.begin_command_buffer(
                command_buffer,
                &vk::CommandBufferBeginInfo::builder()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;
        }

        Ok(Self {
            device,
            command_buffer,
        })
    }
}

impl DrawingCommandBuffer<'_> {
    fn begin_render_pass(
        &self,
        extent: vk::Extent2D,
        framebuffer: vk::Framebuffer,
        render_pass: vk::RenderPass,
    ) {
        unsafe {
            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    int32: [0, 0, 0, 1],
                },
            }];

            self.device.cmd_begin_render_pass(
                self.command_buffer,
                &vk::RenderPassBeginInfo::builder()
                    .framebuffer(framebuffer)
                    .render_pass(render_pass)
                    .render_area(
                        vk::Rect2D::builder()
                            .offset(vk::Offset2D::default())
                            .extent(extent)
                            .build(),
                    )
                    .clear_values(&clear_values),
                vk::SubpassContents::INLINE,
            );
        }
    }

    #[inline]
    fn draw_object(&self, object: &RenderObject) {
        unsafe { object.draw_to_buffer(self.device, self.command_buffer) }
    }

    #[inline]
    fn finish(self) -> vk::CommandBuffer {
        unsafe {
            self.device.cmd_end_render_pass(self.command_buffer);
            self.device.end_command_buffer(self.command_buffer).unwrap();
        }
        self.command_buffer
    }
}

pub(super) struct CommandBufferCreator<'a> {
    device: &'a ash::Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
}

impl CommandBufferCreator<'_> {
    #[inline]
    pub fn create_one_time_command_buffer(&self) -> Result<OneTimeCommandBuffer> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe { self.device.allocate_command_buffers(&allocate_info)? };

        Ok(OneTimeCommandBuffer::begin(
            self.device,
            command_buffers[0],
        )?)
    }

    #[inline]
    pub fn create_drawing_command_buffer(&self) -> Result<DrawingCommandBuffer> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe { self.device.allocate_command_buffers(&allocate_info)? };

        Ok(DrawingCommandBuffer::begin(
            self.device,
            command_buffers[0],
        )?)
    }

    #[inline]
    pub fn submit(&self, command_buffer: vk::CommandBuffer) -> Result<vk::Fence> {
        let command_buffers = [command_buffer];

        let submit_info = [vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build()];

        unsafe {
            let fence = self
                .device
                .create_fence(&vk::FenceCreateInfo::builder(), None)?;

            self.device.queue_submit(self.queue, &submit_info, fence)?;

            Ok(fence)
        }
    }

    #[inline]
    pub fn submit_blocking(&self, command_buffer: vk::CommandBuffer) -> Result<()> {
        let fence = self.submit(command_buffer)?;

        unsafe {
            let fences = [fence];
            self.device.wait_for_fences(&fences, true, std::u64::MAX)?;
        }

        Ok(())
    }
}
