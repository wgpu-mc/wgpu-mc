#version 450

layout(location=0) in vec2 a_tex_coords;
layout(location=0) out vec2 v_tex_coords;

layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

const vec3[6] vertices = vec3[6] (
    vec3(0.0, 0.0, 1.0),
    vec3(1.0, 0.0, 1.0),
    vec3(1.0, 1.0, 1.0),
    vec3(0.0, 0.0, 1.0),
    vec3(1.0, 1.0, 1.0),
    vec3(0.0, 1.0, 1.0)
);

//layout(location=5) in mat4 model_matrix;

void main() {
    v_tex_coords = a_tex_coords;

    gl_Position = u_view_proj * vec4(vertices[gl_VertexIndex], 1.0);
}