#version 450

layout(location=0) in vec3 in_position;
layout(location=1) in vec2 in_tex_coords;
layout(location=2) in vec2 lightmap_coords;
layout(location=3) in vec3 in_normal;

layout(location=0) out vec2 v_tex_coords;
layout(location=1) out vec3 v_normal;

layout(set=1, binding=0)
//layout(set=2, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    v_tex_coords = in_tex_coords;
    v_normal = in_normal;

    gl_Position = u_view_proj * vec4(in_position, 1.0);
}