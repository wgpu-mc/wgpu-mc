#version 450

layout(location=0) in vec3 pos;
layout(location=1) in uint color_in;

layout(location=0) out vec4 color_out;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 matrix;
};

//layout(location=5) in mat4 model_matrix;

void main() {
    float r = float(color_in & uint(0xff)) / 255.0;
    float g = float((color_in >> 8) & uint(0xff)) / 255.0;
    float b = float((color_in >> 16) & uint(0xff)) / 255.0;
    float a = float((color_in >> 24) & uint(0xff)) / 255.0;
    color_out = vec4(r,g,b,a);
    gl_Position = matrix * vec4(pos, 1.0);
}