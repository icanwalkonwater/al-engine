use crate::errors::*;
use crate::renderer::material::pipeline_parts::{
    create_pipeline_color_blend_state, create_pipeline_depth_stencil_state,
    create_pipeline_multisample_state, create_pipeline_rasterization_state,
    create_pipeline_viewport_state,
};
use crate::renderer::shader_container::ShaderContainer;
use crate::renderer::vertex::Vertex;
use ash::version::DeviceV1_0;
use ash::vk;
use std::ops::Deref;
use crate::renderer::material::shader_manager::ShaderHolder;

pub(in super::super) struct PipelineContainer<'a> {
    pub device: &'a ash::Device,
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

impl Deref for PipelineContainer<'_> {
    type Target = vk::Pipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}

impl Drop for PipelineContainer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device.destroy_pipeline(self.pipeline, None);
        }
    }
}

pub(in super::super) struct PipelineCreator<'a> {
    device: &'a ash::Device,
    pipeline_cache: vk::PipelineCache,
}

impl<'a> PipelineCreator<'a> {
    pub fn new(device: &'a ash::Device) -> Self {
        let cache = vk::PipelineCacheCreateInfo::builder();
        let pipeline_cache = unsafe { device.create_pipeline_cache(&cache, None).unwrap() };

        Self {
            device,
            pipeline_cache,
        }
    }
}

impl PipelineCreator<'_> {
    pub fn create_pipeline<'a, 'b, V: Vertex>(
        &'a self,
        extent: vk::Extent2D,
        shaders: &'b[&'b ShaderHolder<'b>],
        descriptor_set_layouts: &'b [vk::DescriptorSetLayout],
        push_constants: &'b [vk::PushConstantRange],
        render_pass: vk::RenderPass,
    ) -> Result<PipelineContainer<'a>> {
        let pipeline_layout = unsafe {
            self.device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::builder()
                    .set_layouts(descriptor_set_layouts)
                    .push_constant_ranges(push_constants),
                None,
            )?
        };

        let shader_stages = shaders.iter()
            .map(|shader_holder| shader_holder.as_shader_stage().build())
            .collect::<Vec<_>>();

        let vertex_info = V::get_pipeline_create_info();
        let viewport_state = create_pipeline_viewport_state(extent);
        let rasterization_state = create_pipeline_rasterization_state();
        let multisample_state = create_pipeline_multisample_state();
        let depth_stencil_state = create_pipeline_depth_stencil_state();
        let color_blend_state = create_pipeline_color_blend_state();

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_info.vertex_input_state)
            .input_assembly_state(&vertex_info.input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_index(-1);

        let pipelines = unsafe {
            let create_info = [pipeline_create_info.build()];
            self.device
                .create_graphics_pipelines(self.pipeline_cache, &create_info, None)
                .map_err(|(_, result)| result)?
        };

        Ok(PipelineContainer {
            device: &self.device,
            pipeline: pipelines[0],
            layout: pipeline_layout,
        })
    }
}
