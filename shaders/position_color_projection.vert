#version 450

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 projection;
} ubo;

layout(location = 0) in vec2 position;
layout(location = 1) in vec3 color;

layout(location = 0) out vec3 fragColor;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = ubo.projection * vec4(position, 0.0, 1.0);
}
