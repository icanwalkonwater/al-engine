use crate::errors::*;
use crate::renderer::descriptor_set_creator::DescriptorSetWrapper;
use crate::renderer::SHADERS_LOCATION;
use ash::version::DeviceV1_0;
use ash::vk;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use crate::utils::str_to_cstr;

pub(in super::super) struct ShaderHolder<'a> {
    device: &'a ash::Device,
    module: vk::ShaderModule,
    main: &'static str,
    stage: vk::ShaderStageFlags,
    pub descriptor_set_layout: Option<vk::DescriptorSetLayout>,
}

impl ShaderHolder<'_> {
    pub fn as_shader_stage(&self) -> vk::PipelineShaderStageCreateInfoBuilder<'_> {
        vk::PipelineShaderStageCreateInfo::builder()
            .module(self.module)
            .name(str_to_cstr(self.main))
            .stage(self.stage)
    }
}

impl Drop for ShaderHolder<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.module, None);
        }
    }
}

pub(in super::super) struct ShaderManager<'a> {
    device: &'a ash::Device,
    shaders: HashMap<&'static str, ShaderHolder<'a>>,
}

impl<'a> ShaderManager<'a> {
    pub fn get(&self, shader: &str) -> Option<&ShaderHolder> {
        self.shaders.get(shader)
    }

    pub fn register(
        &'a mut self,
        shader: &'static str,
        main: &'static str,
        stage: vk::ShaderStageFlags,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<()> {
        let module =
            self.create_shader_module(&Self::read_shader_code(&format!("{}.spv", shader))?)?;

        self.shaders.insert(
            shader,
            ShaderHolder::<'a> {
                device: self.device,
                module,
                main,
                stage,
                descriptor_set_layout: Some(descriptor_set_layout),
            },
        );

        Ok(())
    }

    fn create_shader_module(&self, code: &[u32]) -> Result<vk::ShaderModule> {
        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(code);

        Ok(unsafe {
            self.device
                .create_shader_module(&shader_module_create_info, None)?
        })
    }

    fn read_shader_code(shader_name: &str) -> Result<Vec<u32>> {
        let mut path: PathBuf = SHADERS_LOCATION.iter().collect();
        path.push(shader_name);

        let mut file = File::open(&path)
            .chain_err(|| format!("Failed to open SPIR-V file at {:?} !", path))?;

        Ok(ash::util::read_spv(&mut file)
            .chain_err(|| format!("Failed to read SPIR-V shader at {:?} !", path))?)
    }
}
