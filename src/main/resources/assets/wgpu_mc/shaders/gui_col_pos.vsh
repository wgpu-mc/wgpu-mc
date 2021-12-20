#version 450

layout(location=0) in vec3 pos;
layout(location=1) in uint color_in;

layout(location=0) out vec4 color_out;

layout(set=1, binding=0)
uniform Uniforms {
    mat4 matrix;
};

//layout(location=5) in mat4 model_matrix;

void main() {
//    float r = float(color_in & 0xff) / 255.0;
//    float g = float((color_in >> 8) & 0xff) / 255.0;
//    float b = float((color_in >> 16) & 0xff) / 255.0;
//    float a = float((color_in >> 24) & 0xff) / 255.0;
//    color_out = vec4(r,g,b,a);
    color_out = vec4(1.0, 1.0, 1.0, 1.0);
    gl_Position = matrix * vec4(pos, 1.0);
}