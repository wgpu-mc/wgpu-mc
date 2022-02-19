#version 450

layout(location=0) out vec4 position_out;
layout(location=1) out vec2 uv_out;
layout(location=2) out vec3 normal_out;

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

void main() {
    vec4 diffuse_color = texture(sampler2D(t_diffuse, s_diffuse), vec2(v_tex_coords.x, v_tex_coords.y));
    float bad_lighting = dot(normal, vec3(0.5, 0.5, 0.5))*0.5 + 0.5;

    f_color = diffuse_color * bad_lighting;
}