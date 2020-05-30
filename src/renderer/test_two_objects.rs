use ash::version::DeviceV1_0;
use ash::vk;

use crate::impl_vertex;
use crate::renderer::shader_container::ShaderContainer;
use crate::renderer::vertex::Vertex;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vertex1 {
    pub position: [f32; 2],
}

impl_vertex! {
    Vertex1;
    layout(location = 0) in vec2 position;
}

pub const WHITE_CUBE: [Vertex1; 4] = [
    Vertex1 { position: [0., 0.] },
    Vertex1 { position: [1., 0.] },
    Vertex1 { position: [1., 1.] },
    Vertex1 { position: [0., 1.] },
];

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vertex2 {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl_vertex! {
    Vertex2;
    layout(location = 0) in vec2 position;
    layout(location = 1) in vec3 color;
}

fn create_pipeline_viewport_state(
    extent: vk::Extent2D,
) -> (
    Vec<vk::Viewport>,
    Vec<vk::Rect2D>,
    vk::PipelineViewportStateCreateInfo,
) {
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

    (viewports, scissors, create_info)
}

fn create_pipeline_rasterization_multisample_state() -> (
    vk::PipelineRasterizationStateCreateInfo,
    vk::PipelineMultisampleStateCreateInfo,
) {
    (
        vk::PipelineRasterizationStateCreateInfo::builder()
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.)
            .rasterizer_discard_enable(false)
            .depth_clamp_enable(false)
            .depth_bias_enable(false)
            .build(),
        vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .alpha_to_one_enable(false)
            .alpha_to_coverage_enable(false)
            .build(),
    )
}

fn create_pipeline_depth_stencil_state() -> vk::PipelineDepthStencilStateCreateInfo {
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
        .build()
}

fn create_pipeline_color_blend_state() -> (
    Vec<vk::PipelineColorBlendAttachmentState>,
    vk::PipelineColorBlendStateCreateInfo,
) {
    let color_attachment_states = vec![vk::PipelineColorBlendAttachmentState::builder()
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
        .attachments(&color_attachment_states)
        .blend_constants([0., 0., 0., 0.])
        .build();

    (color_attachment_states, create_info)
}

fn create_pipeline_layout(
    device: &ash::Device,
    layouts: &[vk::DescriptorSetLayout],
) -> vk::PipelineLayout {
    let create_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(layouts);

    unsafe { device.create_pipeline_layout(&create_info, None).unwrap() }
}

pub fn create_pipeline_vertex1(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    ubo_layout: vk::DescriptorSetLayout,
) -> vk::GraphicsPipelineCreateInfo {
    let shaders = ShaderContainer::new(device, "position_projection.vert.spv", "white.frag.spv");
    let shader_stages = shaders.as_shader_stages();

    let (_vertex_bindings, _vertex_attributes, vertex_input_state, input_assembly_state) =
        Vertex1::get_pipeline_create_info();

    let (_viewports, _scissors, viewport_state) = create_pipeline_viewport_state(extent);

    let (rasterization_state, multisample_state) =
        create_pipeline_rasterization_multisample_state();

    let depth_stencil_state = create_pipeline_depth_stencil_state();

    let (_attachment_states, color_blend_state) = create_pipeline_color_blend_state();

    let pipeline_layout = create_pipeline_layout(device, &[ubo_layout]);

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .base_pipeline_index(-1)
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .depth_stencil_state(&depth_stencil_state)
        .color_blend_state(&color_blend_state);

    pipeline_create_info.build()
}

pub fn create_pipeline_vertex2(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    ubo_layout: vk::DescriptorSetLayout,
) -> vk::GraphicsPipelineCreateInfo {
    let shaders = ShaderContainer::new(
        device,
        "position_color_projection.vert.spv",
        "color.frag.spv",
    );
    let shader_stages = shaders.as_shader_stages();

    let (_vertex_bindings, _vertex_attributes, vertex_input_state, input_assembly_state) =
        Vertex2::get_pipeline_create_info();

    let (_viewports, _scissors, viewport_state) = create_pipeline_viewport_state(extent);

    let (rasterization_state, multisample_state) =
        create_pipeline_rasterization_multisample_state();

    let depth_stencil_state = create_pipeline_depth_stencil_state();

    let (_attachment_states, color_blend_state) = create_pipeline_color_blend_state();

    let pipeline_layout = create_pipeline_layout(device, &[ubo_layout]);

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .base_pipeline_index(-1)
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .depth_stencil_state(&depth_stencil_state)
        .color_blend_state(&color_blend_state);

    pipeline_create_info.build()
}

pub fn create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,
    extent: vk::Extent2D,
) {
    let command_buffers = unsafe {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(2);

        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to create command buffers")
    };

    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        // Begin command buffer
        unsafe {
            device
                .begin_command_buffer(
                    command_buffer,
                    &vk::CommandBufferBeginInfo::builder()
                        .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE),
                )
                .unwrap();
        }

        // Begin render pass
        unsafe {
            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0., 0., 0., 1.],
                },
            }];

            device.cmd_begin_render_pass(
                command_buffer,
                &vk::RenderPassBeginInfo::builder()
                    .render_pass(render_pass)
                    .framebuffer(framebuffer)
                    .render_area(
                        vk::Rect2D::builder()
                            .offset(vk::Offset2D::builder().build())
                            .extent(extent)
                            .build(),
                    )
                    .clear_values(&clear_values),
                vk::SubpassContents::INLINE,
            )
        }

        // Bind pipeline
        // Bind vertex/index/ubo buffers
        // Draw

        // End render pass

        // End command buffer
    }
}
