use ash::vk;

macro_rules! vulkan_format_trans {
    (vec1) => {ash::vk::Format::R32_SFLOAT};
    (vec2) => {ash::vk::Format::R32G32_SFLOAT};
    (vec3) => {ash::vk::Format::R32G32B32_SFLOAT};
}

macro_rules! impl_vertex {
    (
        $type:ty;
        $( layout(location = $location:literal) in $format:ident $attribute:ident; )*
    ) => {
        impl $crate::renderer::vertex::Vertex for $type {
            fn get_binding_descriptions() -> [ash::vk::VertexInputBindingDescription; 1] {
                [
                    ash::vk::VertexInputBindingDescription::builder()
                        .binding(0)
                        .stride(std::mem::size_of::<Self>() as u32)
                        .input_rate(vk::VertexInputRate::VERTEX)
                        .build()
                ]
            }

            fn get_attribute_descriptions() -> Vec<ash::vk::VertexInputAttributeDescription> {
                vec![$(
                    ash::vk::VertexInputAttributeDescription::builder()
                        .binding(0)
                        .location($location)
                        .format(vulkan_format_trans!($format))
                        .offset(memoffset::offset_of!(Self, $attribute) as u32)
                        .build(),
                )*]
            }
        }
    };
}

pub trait Vertex {
    fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1];
    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
}

#[repr(C)]
#[derive(Debug, Clone)]
pub(super) struct Vertex2DRgb {
    pos: [f32; 2],
    color: [f32; 3],
}

impl_vertex! {
    Vertex2DRgb;
    layout(location = 0) in vec2 pos;
    layout(location = 1) in vec3 color;
}

pub(super) const TRIANGLE_VERTICES: [Vertex2DRgb; 4] = [
    Vertex2DRgb {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex2DRgb {
        pos: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex2DRgb {
        pos: [0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex2DRgb {
        pos: [-0.5, 0.5],
        color: [1.0, 1.0, 1.0],
    },
];

pub(super) const TRIANGLE_INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];
