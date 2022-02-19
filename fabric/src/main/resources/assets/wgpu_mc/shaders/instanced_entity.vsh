#version 450

layout(location=0) in vec3 position_in;
layout(location=1) in vec2 uv_in;
layout(location=2) in vec3 normal_in;
layout(location=3) in uint part_id;

layout(location=4) in uint entity_index;
layout(location=5) in uint entity_texture_index;
layout(location=6) in vec4 posRotMat1;
layout(location=7) in vec4 posRotMat2;
layout(location=8) in vec4 posRotMat3;
layout(location=9) in vec4 posRotMat4;

layout(set=0, binding=0)
uniform Instances {
    mat4[][] parts;
};

layout(set=2, binding=0) uniform mat4 projection;

layout(location=0) out vec4 position_out;
layout(location=1) out vec2 uv_out;
layout(location=2) out vec3 normal_out;

void main() {
    mat4 posRotMat = mat4(posRotMat1, posRotMat2, posRotMat3, posRotMat4);
    mat4 part_instance_mat = parts[entity_index][part_id];

    position_out = posRotMat * part_instance_mat * projection * vec4(position_in, 1.0);
    normal_out = mat3(posRotMat) * mat3(part_instance_mat) * normal_in;
    uv_out = uv_in;
}