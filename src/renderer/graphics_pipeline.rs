use crate::renderer::vulkan_app::VulkanApp;
use crate::renderer::SHADERS_LOCATION;
use ash::version::DeviceV1_0;
use ash::vk;
use std::ffi::CString;
use std::fs::File;
use std::path::PathBuf;

impl VulkanApp {
    pub(in crate::renderer) fn create_graphics_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> (vk::Pipeline, vk::PipelineLayout) {
        let vert_shader =
            Self::create_shader_module(device, &Self::read_shader_code("identity.vert.spv"));
        let frag_shader =
            Self::create_shader_module(device, &Self::read_shader_code("red.frag.spv"));

        let entry_point = CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(vert_shader)
                .name(&entry_point)
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(frag_shader)
                .name(&entry_point)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder().build();

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build();

        let viewports = [vk::Viewport::builder()
            .x(0.)
            .y(0.)
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0.)
            .max_depth(1.)
            .build()];

        let scissors = [vk::Rect2D::builder()
            .offset(vk::Offset2D::builder().x(0).y(0).build())
            .extent(extent)
            .build()];

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors)
            .build();

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.)
            .rasterizer_discard_enable(false)
            .depth_clamp_enable(false)
            .depth_bias_enable(false)
            .build();

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .alpha_to_one_enable(false)
            .alpha_to_coverage_enable(false)
            .build();

        let stencil_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .build();

        let depth_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_bounds_test_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .stencil_test_enable(false)
            .front(stencil_state)
            .back(stencil_state)
            .max_depth_bounds(1.)
            .min_depth_bounds(0.)
            .build();

        let color_attachment_states = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::all())
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build()];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_attachment_states)
            .blend_constants([0., 0., 0., 0.])
            .build();

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder().build();

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout !")
        };

        let graphics_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_index(-1)
            .build()];

        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &graphics_pipeline_create_infos,
                    None,
                )
                .expect("Failed to create graphic pipelines !")
        };

        unsafe {
            device.destroy_shader_module(vert_shader, None);
            device.destroy_shader_module(frag_shader, None);
        }

        (graphics_pipelines[0], pipeline_layout)
    }

    pub(in crate::renderer) fn create_render_pass(
        device: &ash::Device,
        format: vk::Format,
    ) -> vk::RenderPass {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref])
            .build();

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[color_attachment])
            .subpasses(&[subpass])
            .build();

        unsafe {
            device
                .create_render_pass(&render_pass_create_info, None)
                .expect("Failed to create render pass !")
        }
    }

    fn create_shader_module(device: &ash::Device, code: &[u32]) -> vk::ShaderModule {
        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(code).build();

        unsafe {
            device
                .create_shader_module(&shader_module_create_info, None)
                .expect(&format!("Failed to create shader module !"))
        }
    }

    fn read_shader_code(shader_name: &str) -> Vec<u32> {
        let mut path: PathBuf = SHADERS_LOCATION.iter().collect();
        path.push(shader_name);

        let mut file =
            File::open(&path).expect(&format!("Failed to open SPIR-V file at {:?} !", path));

        ash::util::read_spv(&mut file)
            .expect(&format!("Failed to read SPIR-V shader at {:?} !", path))
    }
}
