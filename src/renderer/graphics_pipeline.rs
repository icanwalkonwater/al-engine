use crate::renderer::vulkan_app::VulkanApp;
use crate::renderer::SHADERS_LOCATION;
use ash::version::DeviceV1_0;
use ash::vk;
use std::ffi::CString;
use std::fs::File;
use std::path::PathBuf;

impl VulkanApp {
    pub(in crate::renderer) fn create_graphics_pipeline(device: &ash::Device) {
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

        unsafe {
            device.destroy_shader_module(vert_shader, None);
            device.destroy_shader_module(frag_shader, None);
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
