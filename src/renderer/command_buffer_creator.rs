use crate::errors::*;
use crate::renderer::allocation::BufferAllocation;
use ash::version::DeviceV1_0;
use ash::vk;
use std::borrow::Borrow;

pub struct OneTimeCommandBuffer<'a> {
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
    fn finish(self) -> vk::CommandBuffer {
        unsafe {
            self.device.end_command_buffer(self.command_buffer).unwrap();
        }
        self.command_buffer
    }
}

pub struct CommandBufferCreator<'a> {
    device: &'a ash::Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
}

impl CommandBufferCreator<'_> {
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
    pub fn submit(&self, buffer: OneTimeCommandBuffer) -> Result<vk::Fence> {
        let command_buffer = buffer.finish();
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

    pub fn submit_blocking(&self, buffer: OneTimeCommandBuffer) -> Result<()> {
        let fence = self.submit(buffer)?;

        unsafe {
            let fences = [fence];
            self.device.wait_for_fences(&fences, true, std::u64::MAX)?;
        }

        Ok(())
    }
}
