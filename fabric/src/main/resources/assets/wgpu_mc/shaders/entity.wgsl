struct PushConstants {
    parts_per_entity: u32
}

var<push_constant> push_constants: PushConstants;

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> persp_proj: mat4x4<f32>;
@group(0) @binding(2) var e_sampler: sampler;

@group(1) @binding(0) var<storage> transforms: array<mat4x4<f32>>;
@group(1) @binding(1) var e_texture: texture_2d<f32>;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) overlay: vec4<f32>
};

@vertex
fn vert(
    @location(0) pos_in: vec3<f32>,
    @location(1) tex_coords_u32: u32,
    @location(2) normal: vec3<f32>,
    @location(3) part_id: u32,
    //Instance vertex start
    @location(4) entity_texture_offset: vec2<f32>,
    @location(5) overlay: u32,
    @builtin(instance_index) entity_index: u32
) -> VertexResult {
    var vr: VertexResult;

//    var tex_coords: vec2<f32> = vec2<f32>(f32(tex_coords_u32 & 0xffffu), f32(tex_coords_u32 >> 16u)) * vec2<f32>(0.00048828125, 0.00048828125);
    var tex_coords: vec2<f32> = vec2<f32>(f32(tex_coords_u32 & 0xffffu), f32(tex_coords_u32 >> 16u)) * vec2<f32>(0.015625, 0.015625);

    var part_transform_index: u32 = (entity_index * push_constants.parts_per_entity) + part_id;
    var part_transform: mat4x4<f32> = transforms[part_transform_index];

    var overlay_color: vec4<f32> = vec4<f32>(
        f32(overlay & 0xffu) / 255.0,
        f32((overlay >> 8u) & 0xffu) / 255.0,
        f32((overlay >> 16u) & 0xffu) / 255.0,
        f32(overlay >> 24u) / 255.0,
    );

    vr.pos = persp_proj * view_proj * ((part_transform * vec4<f32>(pos_in, 1.0)));

    vr.tex_coords = tex_coords + entity_texture_offset;
    vr.normal = vec3(1.0, 0.0, 0.0);
    vr.overlay = overlay_color;

    return vr;
}

@fragment
fn frag(in: VertexResult) -> @location(0) vec4<f32> {
   return vec4<f32>(textureSample(e_texture, e_sampler, in.tex_coords).rgb, 1.0);
}