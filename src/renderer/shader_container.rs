use std::ffi::CStr;
use std::fs::File;
use std::path::PathBuf;

use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::SHADERS_LOCATION;

pub const VERTEX_MAIN: &str = "main\0";
pub const FRAGMENT_MAIN: &str = "main\0";

pub(super) struct ShaderContainer<'a> {
    device: &'a ash::Device,
    vert: vk::ShaderModule,
    frag: vk::ShaderModule,
}

impl<'a> ShaderContainer<'a> {
    pub fn new(device: &'a ash::Device, vertex_shader: &str, fragment_shader: &str) -> Self {
        let vert = Self::create_shader_module(device, &Self::read_shader_code(vertex_shader));
        let frag = Self::create_shader_module(device, &Self::read_shader_code(fragment_shader));

        Self { device, vert, frag }
    }
}

impl ShaderContainer<'_> {
    pub fn as_shader_stages(&self) -> Vec<vk::PipelineShaderStageCreateInfo> {
        let vertex_main = unsafe { CStr::from_ptr(VERTEX_MAIN.as_ptr() as *mut i8) };
        let fragment_main = unsafe { CStr::from_ptr(FRAGMENT_MAIN.as_ptr() as *mut i8) };

        vec![
            vk::PipelineShaderStageCreateInfo::builder()
                .module(self.vert)
                .name(vertex_main)
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(self.frag)
                .name(fragment_main)
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

impl Drop for ShaderContainer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.vert, None);
            self.device.destroy_shader_module(self.frag, None);
        }
    }
}
