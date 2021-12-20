#version 450

layout(location=0) in vec3 pos;
layout(location=1) in vec2 uv_in;

layout(location=0) out vec2 uv_out;

layout(set=1, binding=0)
uniform Uniforms {
    mat4 matrix;
};

//layout(location=5) in mat4 model_matrix;

void main() {
    uv_out = uv_in;
    gl_Position = matrix * vec4(pos, 1.0);
}