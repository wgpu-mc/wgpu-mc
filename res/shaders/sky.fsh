#version 450

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform textureCube t_skybox;
layout(set = 0, binding = 1) uniform sampler s_skybox;

void main() {
    f_color = texture(samplerCube(t_skybox, s_skybox), vec3(1.0, 1.0, 1.0));
}