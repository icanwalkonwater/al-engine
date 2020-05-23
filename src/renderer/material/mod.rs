use ash::vk;

pub(super) struct Material {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_passes: Vec<vk::RenderPass>,
}


