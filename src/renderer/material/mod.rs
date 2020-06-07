use ash::version::DeviceV1_0;
use ash::vk;
use crate::renderer::material::shader_manager::ShaderHolder;
use crate::renderer::material::pipeline_creator::{PipelineCreator, PipelineContainer};
use crate::renderer::descriptor_set_creator::DescriptorSetWrapper;

mod pipeline_creator;
mod pipeline_parts;
mod shader_manager;
mod material_builder;

pub struct Material<'a> {
    pub(super) pipeline: PipelineContainer<'a>,
    pub(super) descriptor_sets: Vec<DescriptorSetWrapper<'a>>,
}

impl Material<'_> {
    #[inline]
    pub unsafe fn bind_pipeline(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline.pipeline,
        );
    }

    #[inline]
    pub unsafe fn bind_descriptor_sets(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
    ) {
        let descriptor_sets = self.descriptor_sets.iter()
            .map(|descriptor_set| descriptor_set.0)
            .collect::<Vec<_>>();

        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline.layout,
            0,
            &descriptor_sets,
            &[],
        );
    }
}
