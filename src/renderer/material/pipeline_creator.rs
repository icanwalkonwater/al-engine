use ash::vk;
use ash::version::DeviceV1_0;

pub struct PipelineCreator<'a> {
    device: &'a ash::Device,
}

impl<'a> PipelineCreator<'a> {
    pub fn new(device: &ash::Device,)
}

impl PipelineCreator<'_> {
    fn create_pipeline(device: &ash::Device, descriptor_sets: &[vk::DescriptorSetLayout], push_constants: &[vk::PushConstantRange]) -> Result<vk::Pipeline> {
        let pipeline_layout = unsafe {
            device.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(descriptor_sets)
                .push_constant_ranges(push_constants),
                                          None)?
        };


    }
}