use crate::errors::*;
use crate::renderer::descriptor_set_creator::DescriptorSetCreator;
use crate::renderer::material::pipeline_creator::PipelineCreator;
use crate::renderer::material::shader_manager::ShaderHolder;
use crate::renderer::material::Material;
use crate::renderer::shader_container::ShaderContainer;
use ash::vk;
use crate::renderer::vertex::Vertex;

#[derive(Default)]
pub(in super::super) struct MaterialBuilder<'a> {
    vertex_shader: Option<&'a ShaderHolder<'a>>,
    fragment_shader: Option<&'a ShaderHolder<'a>>,
    extent: Option<vk::Extent2D>,
    render_pass: Option<vk::RenderPass>,
}

impl MaterialBuilder<'_> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> MaterialBuilder<'a> {
    #[inline]
    pub fn vertex_shader(mut self, shader: &'a ShaderHolder) -> Self {
        self.vertex_shader = Some(shader);
        self
    }

    #[inline]
    pub fn fragment_shader(mut self, shader: &'a ShaderHolder) -> Self {
        self.fragment_shader = Some(shader);
        self
    }

    #[inline]
    pub fn extent(mut self, extent: vk::Extent2D) -> Self {
        self.extent = Some(extent);
        self
    }

    #[inline]
    pub fn render_pass(mut self, render_pass: vk::RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }

    pub fn build<V: Vertex>(
        self,
        pipeline_creator: &'a PipelineCreator,
        descriptor_set_creator: &'a DescriptorSetCreator,
    ) -> Result<Material<'a>> {
        let vertex_shader = self.vertex_shader.ok_or("No vertex shader present")?;
        let fragment_shader = self.fragment_shader.ok_or("No fragment shader present")?;

        let shaders = [vertex_shader, fragment_shader];

        let descriptor_set_layouts = {
            let mut descriptor_set_layouts = Vec::with_capacity(2);
            if let Some(set_layout) = vertex_shader.descriptor_set_layout {
                descriptor_set_layouts.push(set_layout);
            }
            if let Some(set_layout) = fragment_shader.descriptor_set_layout {
                descriptor_set_layouts.push(set_layout);
            }

            descriptor_set_layouts
        };

        let pipeline = pipeline_creator.create_pipeline::<V>(
            self.extent.unwrap(),
            &shaders,
            &descriptor_set_layouts,
            &[],
            self.render_pass.unwrap(),
        )?;

        let descriptor_sets = {
            let mut descriptor_sets = Vec::new();
            for descriptor_set_layout in descriptor_set_layouts {
                descriptor_sets
                    .push(descriptor_set_creator.allocate_descriptor_set(descriptor_set_layout)?);
            }
            descriptor_sets
        };

        Ok(Material {
            pipeline,
            descriptor_sets,
        })
    }
}
