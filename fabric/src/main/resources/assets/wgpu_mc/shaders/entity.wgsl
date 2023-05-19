struct Uniforms {
    view_proj: mat4x4<f32>
};

struct Transforms {
    mats: array<mat4x4<f32>>
};

struct PushConstants {
    parts_per_entity: u32
}

var<push_constant> push_constants: PushConstants;

@group(0) @binding(0)
var<uniform> uniform_data: Uniforms;

@group(1) @binding(0)
var<storage> transforms: Transforms;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>
};

@vertex
fn vert(
    @location(0) pos_in: vec3<f32>,
    @location(1) tex_coords_u32: u32,
    @location(2) normal: vec3<f32>,
    @location(3) part_id: u32,
    @location(4) entity_index: u32,
    @location(5) entity_texture_offset: vec2<f32>,
) -> VertexResult {
    var vr: VertexResult;

    var tex_coords: vec2<f32> = vec2<f32>(f32(tex_coords_u32 & 0xffffu), f32(tex_coords_u32 >> 16u)) * vec2<f32>(0.00048828125, 0.00048828125);

    var part_transform_index: u32 = (entity_index * push_constants.parts_per_entity) + part_id;
    var part_transform: mat4x4<f32> = transforms.mats[part_transform_index];

    vr.pos = uniform_data.view_proj * part_transform * vec4<f32>(pos_in + vec3<f32>(0.0, 64.0, 0.0), 1.0);

    vr.tex_coords = tex_coords + entity_texture_offset;
    vr.normal = mat3x3<f32>(part_transform[0].xyz, part_transform[1].xyz, part_transform[2].xyz) * normal;

    return vr;
}

@group(2) @binding(0)
var t_texture: texture_2d<f32>;

@group(2) @binding(1)
var t_sampler: sampler;

@fragment
fn frag(in: VertexResult) -> @location(0) vec4<f32> {
   return vec4<f32>(textureSample(t_texture, t_sampler, in.tex_coords).rgb, 1.0);
}