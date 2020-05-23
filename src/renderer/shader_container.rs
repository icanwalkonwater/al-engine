use crate::renderer::SHADERS_LOCATION;
use ash::version::DeviceV1_0;
use ash::vk;
use std::ffi::CString;
use std::fs::File;
use std::path::PathBuf;

pub(super) struct ShaderContainer<'a> {
    device: &'a ash::Device,
    vert: vk::ShaderModule,
    frag: vk::ShaderModule,
}

impl<'a> ShaderContainer<'a> {
    pub fn new(device: &'a ash::Device, vertex_shader: &str, fragment_shader: &str) -> Self {
        let vert = Self::create_shader_module(device, &Self::read_shader_code(vertex_shader));
        let frag = Self::create_shader_module(device, &Self::read_shader_code(fragment_shader));

        Self {
            device,
            vert,
            frag,
        }
    }
}

impl ShaderContainer<'_> {
    pub fn as_shader_stages(&self) -> Vec<vk::PipelineShaderStageCreateInfo> {
        let entry_point = CString::new("main").unwrap();
        vec![
            vk::PipelineShaderStageCreateInfo::builder()
                .module(self.vert)
                .name(&entry_point)
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(self.frag)
                .name(&entry_point)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ]
    }
}

impl ShaderContainer<'_> {
    fn create_shader_module(device: &ash::Device, code: &[u32]) -> vk::ShaderModule {
        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(code);

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
