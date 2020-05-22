use ash::vk;

#[repr(C)]
#[derive(Debug, Clone)]
pub(super) struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(Self, pos) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(memoffset::offset_of!(Self, color) as u32)
                .build(),
        ]
    }
}

pub(super) const TRIANGLE_VERTICES: [Vertex; 4] = [
    Vertex {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5],
        color: [1.0, 1.0, 1.0],
    },
];

pub(super) const TRIANGLE_INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];
