use ash::vk;
use ash::vk::Extent2D;
use std::ops::Deref;

pub(super) struct PipelineViewportStateContainer {
    viewports: Vec<vk::Viewport>,
    scissors: Vec<vk::Rect2D>,
    create_info: vk::PipelineViewportStateCreateInfo,
}

impl PipelineViewportStateContainer {
    fn new(extent: Extent2D) -> Self {
        let viewports = vec![vk::Viewport::builder()
            .x(0.)
            .y(0.)
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0.)
            .max_depth(1.)
            .build()];

        let scissors = vec![vk::Rect2D::builder()
            .offset(vk::Offset2D::builder().x(0).y(0).build())
            .extent(extent)
            .build()];

        let create_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors)
            .build();

        Self {
            viewports,
            scissors,
            create_info,
        }
    }
}

impl Deref for PipelineViewportStateContainer {
    type Target = vk::PipelineViewportStateCreateInfo;

    fn deref(&self) -> &Self::Target {
        &self.create_info
    }
}

pub(super) fn create_pipeline_viewport_state(
    extent: vk::Extent2D,
) -> PipelineViewportStateContainer {
    PipelineViewportStateContainer::new(extent)
}

pub(super) fn create_pipeline_rasterization_state(
) -> vk::PipelineRasterizationStateCreateInfoBuilder<'static> {
    vk::PipelineRasterizationStateCreateInfo::builder()
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.)
        .rasterizer_discard_enable(false)
        .depth_clamp_enable(false)
        .depth_bias_enable(false)
}

pub(super) fn create_pipeline_multisample_state(
) -> vk::PipelineMultisampleStateCreateInfoBuilder<'static> {
    vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .sample_shading_enable(false)
        .alpha_to_one_enable(false)
        .alpha_to_coverage_enable(false)
}

pub(super) fn create_pipeline_depth_stencil_state(
) -> vk::PipelineDepthStencilStateCreateInfoBuilder<'static> {
    let stencil_state = vk::StencilOpState::builder()
        .fail_op(vk::StencilOp::KEEP)
        .pass_op(vk::StencilOp::KEEP)
        .depth_fail_op(vk::StencilOp::KEEP)
        .compare_op(vk::CompareOp::ALWAYS)
        .build();

    vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(false)
        .depth_write_enable(false)
        .depth_bounds_test_enable(false)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
        .stencil_test_enable(false)
        .front(stencil_state)
        .back(stencil_state)
        .max_depth_bounds(1.)
        .min_depth_bounds(0.)
}

pub(super) struct PipelineColorBlendStateContainer {
    color_blend_attachment_states: Vec<vk::PipelineColorBlendAttachmentState>,
    create_info: vk::PipelineColorBlendStateCreateInfo,
}

impl PipelineColorBlendStateContainer {
    pub fn new() -> Self {
        let color_blend_attachment_states = vec![vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::all())
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build()];

        let create_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachment_states)
            .blend_constants([0., 0., 0., 0.])
            .build();

        Self {
            color_blend_attachment_states,
            create_info,
        }
    }
}

impl Deref for PipelineColorBlendStateContainer {
    type Target = vk::PipelineColorBlendStateCreateInfo;

    fn deref(&self) -> &Self::Target {
        &self.create_info
    }
}

pub(super) fn create_pipeline_color_blend_state() -> PipelineColorBlendStateContainer {
    PipelineColorBlendStateContainer::new()
}
