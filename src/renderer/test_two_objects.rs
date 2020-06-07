use ash::version::DeviceV1_0;
use ash::vk;

use crate::impl_ubo;
use crate::impl_vertex;
use crate::renderer::render_object::RenderObject;
use crate::renderer::shader_container::ShaderContainer;
use crate::renderer::vertex::Vertex;
use nalgebra::Matrix4;
use crate::renderer::material::Material;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vertex1 {
    pub position: [f32; 2],
}

impl_vertex! {
    Vertex1;
    layout(location = 0) in vec2 position;
}

pub const WHITE_CUBE: [Vertex1; 4] = [
    Vertex1 { position: [0., 0.] },
    Vertex1 { position: [1., 0.] },
    Vertex1 { position: [1., 1.] },
    Vertex1 { position: [0., 1.] },
];

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vertex2 {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl_vertex! {
    Vertex2;
    layout(location = 0) in vec2 position;
    layout(location = 1) in vec3 color;
}

#[repr(C)]
#[derive(Debug)]
pub struct ProjectionUbo {
    pub projection: Matrix4<f32>,
}

impl_ubo! {
    layout(binding = 0) uniform ProjectionUbo[1];
}

pub fn get_object1() {
    // x--x  -1
    // |  |
    // x--x  0  1

    // 0--2
    // |  |
    // 1--3

    let plane = [
        Vertex1 {
            position: [-2., -1.],
        },
        Vertex1 {
            position: [-2., 0.],
        },
        Vertex1 {
            position: [-1., -1.],
        },
        Vertex1 {
            position: [-1., -0.],
        },
    ];

    let indices = [0, 2, 1, 2, 3, 1];

    // let material = Material::new
    // TODO
}
