use crate::errors::*;
use crate::renderer::allocation::BufferAllocation;
use ash::version::DeviceV1_0;
use ash::vk;
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Clone, Copy)]
pub(super) struct DescriptorSetWrapper<'a, T: 'a = ()>(
    pub vk::DescriptorSet,
    std::marker::PhantomData<&'a T>,
);

pub(super) struct DescriptorSetCreator<'a> {
    device: &'a ash::Device,
    descriptor_pool: vk::DescriptorPool,
}

impl<'a> DescriptorSetCreator<'a> {
    pub fn new(device: &'a ash::Device, amount_uniforms: u32, max_sets: u32) -> Result<Self> {
        let pool_sizes = [vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(amount_uniforms)
            .build()];

        let descriptor_pool = unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::builder()
                    .pool_sizes(&pool_sizes)
                    .max_sets(max_sets),
                None,
            )?
        };

        Ok(Self {
            device,
            descriptor_pool,
        })
    }

    #[inline]
    pub fn allocate_descriptor_set(
        &self,
        layout: vk::DescriptorSetLayout,
    ) -> Result<DescriptorSetWrapper> {
        let descriptor_set = unsafe {
            let layouts = [layout];
            self.device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(self.descriptor_pool)
                    .set_layouts(&layouts),
            )?[0]
        };

        Ok(DescriptorSetWrapper(descriptor_set, PhantomData))
    }

    pub fn bind_ubo_to_descriptor_set<U>(
        &self,
        descriptor_set: vk::DescriptorSet,
        binding: u32,
        ubo: &BufferAllocation,
    ) {
        self.bind_buffer_to_descriptor_set::<U>(
            descriptor_set,
            binding,
            ubo,
            vk::DescriptorType::UNIFORM_BUFFER,
        );
    }

    #[inline]
    pub fn bind_buffer_to_descriptor_set<D>(
        &self,
        descriptor_set: vk::DescriptorSet,
        binding: u32,
        buffer: &BufferAllocation,
        ty: vk::DescriptorType,
    ) {
        let buffer_info = [vk::DescriptorBufferInfo::builder()
            .buffer(buffer.buffer)
            .offset(buffer.allocation_info().get_offset() as _)
            .range(std::mem::size_of::<D>() as _)
            .build()];

        let descriptor_write_sets = [vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(ty)
            .buffer_info(&buffer_info)
            .build()];

        unsafe {
            self.device
                .update_descriptor_sets(&descriptor_write_sets, &[]);
        }
    }
}
