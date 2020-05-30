use crate::errors::*;
use crate::renderer::command_buffer_creator::CommandBufferCreator;
use crate::renderer::vertex::Vertex;
use ash::vk;
use std::ops::{Deref, DerefMut};

pub struct BufferAllocation<'a> {
    allocator: &'a VulkanAllocator,
    pub buffer: vk::Buffer,
    allocation: vk_mem::Allocation,
}

impl BufferAllocation<'_> {
    pub fn write(&self) -> Result<TemporaryMemoryMapping> {
        let mapped = self.allocator.vma_allocator.map_memory(&self.allocation)?;

        Ok(TemporaryMemoryMapping {
            allocation: self,
            mapping: mapped,
        })
    }

    pub fn allocation_info(&self) -> vk_mem::AllocationInfo {
        self.allocator
            .vma_allocator
            .get_allocation_info(&self.allocation)
            .unwrap()
    }
}

impl Drop for BufferAllocation<'_> {
    fn drop(&mut self) {
        self.allocator
            .vma_allocator
            .destroy_buffer(self.buffer, &self.allocation)
            .unwrap();
    }
}

pub struct TemporaryMemoryMapping<'a> {
    allocation: &'a BufferAllocation<'a>,
    mapping: *mut u8,
}

impl TemporaryMemoryMapping<'_> {
    pub fn as_ptr<T>(&self) -> *mut T {
        self.mapping as *mut T
    }
}

impl Drop for TemporaryMemoryMapping<'_> {
    fn drop(&mut self) {
        self.allocation
            .allocator
            .vma_allocator
            .unmap_memory(&self.allocation.allocation)
            .unwrap();
    }
}

pub struct VulkanAllocator {
    vma_allocator: vk_mem::Allocator,
}

impl VulkanAllocator {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        let allocator = vk_mem::Allocator::new(&vk_mem::AllocatorCreateInfo {
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            ..Default::default()
        })?;

        Ok(Self {
            vma_allocator: allocator,
        })
    }

    pub fn create_vertex_buffer_with_staging<V: Vertex>(
        &self,
        command_creator: &CommandBufferCreator,
        vertices: &[V],
    ) -> Result<BufferAllocation> {
        self.create_buffer_with_staging(
            command_creator,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vertices,
        )
    }

    pub fn create_index_buffer_with_staging<I>(
        &self,
        command_creator: &CommandBufferCreator,
        indices: &[I],
    ) -> Result<BufferAllocation> {
        self.create_buffer_with_staging(
            command_creator,
            vk::BufferUsageFlags::INDEX_BUFFER,
            indices,
        )
    }

    pub fn create_uniform_buffer<Ubo>(&self) -> Result<BufferAllocation> {
        let size = std::mem::size_of::<Ubo>() as vk::DeviceSize;

        let (buffer, allocation, _) = self.vma_allocator.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(size)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::CpuToGpu,
                required_flags: vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
                ..Default::default()
            },
        )?;

        Ok(BufferAllocation {
            allocator: self,
            buffer,
            allocation,
        })
    }

    fn create_buffer_with_staging<D>(
        &self,
        command_creator: &CommandBufferCreator,
        usage: vk::BufferUsageFlags,
        data: &[D],
    ) -> Result<BufferAllocation> {
        let size = std::mem::size_of_val(data) as vk::DeviceSize;

        // Allocate buffers
        let staging_buffer = self.allocate_staging_buffer(size)?;
        let data_buffer =
            self.allocate_gpu_buffer(size, usage | vk::BufferUsageFlags::TRANSFER_DST)?;

        // Stage indices
        unsafe {
            let mapping = staging_buffer.write()?;
            let data_ptr: *mut D = mapping.as_ptr();
            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len())
        }

        // Copy staging buffer to vertex bugger
        let command_buffer = command_creator.create_one_time_command_buffer()?;
        command_buffer.copy(&staging_buffer, &data_buffer, size);
        command_creator.submit_blocking(command_buffer)?;

        Ok(data_buffer)
    }

    fn allocate_gpu_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> Result<BufferAllocation> {
        let (buffer, allocation, _) = self.vma_allocator.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::GpuOnly,
                ..Default::default()
            },
        )?;

        Ok(BufferAllocation {
            allocator: self,
            buffer,
            allocation,
        })
    }

    fn allocate_staging_buffer(&self, size: vk::DeviceSize) -> Result<BufferAllocation> {
        let (buffer, allocation, _) = self.vma_allocator.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::CpuToGpu,
                required_flags: vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
                ..Default::default()
            },
        )?;

        Ok(BufferAllocation {
            allocator: self,
            buffer,
            allocation,
        })
    }
}
