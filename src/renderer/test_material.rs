use std::sync::Arc;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::impl_vertex;
use vulkano::pipeline::vertex::BufferlessDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/identity.vert"
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/red.frag"
    }
}

pub struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    pub fn new(x: f32, y: f32) -> Self {
        Self { position: [x, y] }
    }
}

impl_vertex!(Vertex, position);

pub struct TestMaterial {
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

impl TestMaterial {
    pub fn new(
        device: &Arc<Device>,
        swap_chain_extent: [u32; 2],
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> Self {
        let vert_shader =
            vertex_shader::Shader::load(device.clone()).expect("Failed to create vertex shader !");
        let frag_shader = fragment_shader::Shader::load(device.clone())
            .expect("Failed to create fragment shader !");

        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0..1.0,
        };

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input(BufferlessDefinition {})
                .vertex_shader(vert_shader.main_entry_point(), ())
                .triangle_list()
                .primitive_restart(false)
                .viewports(vec![viewport])
                .fragment_shader(frag_shader.main_entry_point(), ())
                .depth_clamp(false)
                .polygon_mode_fill()
                .cull_mode_back()
                .front_face_clockwise()
                .blend_pass_through()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );
    }

    pub fn pipeline(&self) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        self.pipeline.clone()
    }
}
