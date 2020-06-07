use ash::vk;
use nalgebra::Matrix4;

pub trait UniformBufferObject {
    fn get_descriptor_set_layout_binding() -> vk::DescriptorSetLayoutBinding;
}

#[macro_export]
macro_rules! impl_ubo {
    ( layout(binding = $binding:literal) uniform $type:ty[$len:literal] $(;)? ) => {
        impl $crate::renderer::uniform_buffer_object::UniformBufferObject for $type {
            fn get_descriptor_set_layout_binding() -> ash::vk::DescriptorSetLayoutBinding {
                ash::vk::DescriptorSetLayoutBinding::builder()
                    .binding($binding)
                    .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count($len)
                    .stage_flags(ash::vk::ShaderStageFlags::VERTEX)
                    .build()
            }
        }
    };
}
