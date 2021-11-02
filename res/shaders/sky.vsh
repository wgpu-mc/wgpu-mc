#version 450

const vec3 vertices[3] = vec3[3](
    vec3(-1.0, -1.0, 1.0),
    vec3(-1.0, 1.0, 1.0),
    vec3(1.0, -1.0, 1.0)
);

layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    gl_Position = vec4(vertices[gl_VertexIndex], 1.0);
}