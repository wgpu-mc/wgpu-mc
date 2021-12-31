#version 450

layout(location=0) in vec3 pos;
layout(location=1) in vec3 color_in;

layout(location=0) out vec4 color_out;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 matrix;
};

//layout(location=5) in mat4 model_matrix;

void main() {
    color_out = vec4(color_in, 1.0);
    gl_Position = matrix * vec4(pos, 1.0);
}