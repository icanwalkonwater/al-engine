#version 450

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 projection;
} ubo;

layout(location = 0) in vec2 position;

out gl_PerVertex {
    vec4 gl_Position;
};

void frag() {
    gl_Position = vec4(position, 0.0, 1.0);
}
