use crate::renderer::material::Material;
use ash::version::DeviceV1_0;
use ash::vk;
use crate::renderer::allocation::BufferAllocation;

pub struct RenderObject<'a> {
    pub(super) material: Material,

    pub(super) vertex_buffer: BufferAllocation<'a>,
    pub(super) index_buffer: BufferAllocation<'a>,
    pub(super) index_count: u32,
}

impl RenderObject<'_> {
    pub unsafe fn draw_to_buffer(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        // Bind pipeline
        self.material.bind_pipeline(device, command_buffer);

        let vertex_buffers = [self.vertex_buffer.buffer];
        let offsets = [0];

        device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
        device.cmd_bind_index_buffer(
            command_buffer,
            self.index_buffer.buffer,
            0,
            vk::IndexType::UINT32,
        );

        self.material.bind_descriptor_sets(device, command_buffer);

        device.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0);
    }
}
