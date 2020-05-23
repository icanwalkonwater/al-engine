use ash::vk;
use crate::renderer::buffers::BufferAllocation;

pub struct RenderObject {
    pub(super) pipeline: vk::Pipeline,
    pub(super) pipeline_layout: vk::PipelineLayout,
    pub(super) render_pass: vk::RenderPass,

    pub(super) vertex_buffer: BufferAllocation,
    pub(super) index_buffer: BufferAllocation,
}