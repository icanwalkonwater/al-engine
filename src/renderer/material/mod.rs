use ash::version::DeviceV1_0;
use ash::vk;

mod pipeline_creator;
mod pipeline_parts;

pub(super) struct Material {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

impl Material {
    #[inline]
    pub unsafe fn bind_pipeline(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline,
        );
    }

    #[inline]
    pub unsafe fn bind_descriptor_sets(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
    ) {
        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            &self.descriptor_sets,
            &[],
        );
    }
}
