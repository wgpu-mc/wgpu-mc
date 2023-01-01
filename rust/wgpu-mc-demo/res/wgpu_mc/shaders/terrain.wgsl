struct CameraUniform {
    view_proj: mat4x4<f32>
};

struct UV {
    uv1: vec2<f32>,
    uv2: vec2<f32>,
    blend: f32,
    padding: f32
};

struct UVs {
    uvs: array<UV>
};

struct ChunkOffset {
    x: i32,
    z: i32
}

struct PushConstants {
    chunk_x: i32,
    chunk_z: i32,
    fb_width: f32,
    fb_height: f32
}

var<push_constant> push_constants: PushConstants;

@group(0) @binding(0)
var<uniform> camera_uniform: CameraUniform;

@group(3) @binding(0)
var<uniform> inverse_camera_uniform: CameraUniform;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_coords2: vec2<f32>,
    @location(2) blend: f32,
    @location(3) normal: vec3<f32>,
    @location(4) world_pos: vec3<f32>
//    @location(4) screen_pos: vec4<f32>
};

@vertex
fn vert(
    @location(0) pos_in: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(6) uv_offset: u32
) -> VertexResult {
    // var uv = uv_offsets.uvs[uv_offset];

    var vr: VertexResult;

    var world_pos = pos_in + vec3<f32>(f32(push_constants.chunk_x) * 16.0, 0.0, f32(push_constants.chunk_z) * 16.0);

    vr.world_pos = world_pos;
    vr.pos = camera_uniform.view_proj * vec4<f32>(world_pos, 1.0);
    vr.tex_coords = tex_coords;
    vr.tex_coords2 = tex_coords;
    vr.blend = 1.0;
    // vr.tex_coords = tex_coords + uv.uv1;
    // vr.tex_coords2 = tex_coords + uv.uv2;
    // vr.blend = uv.blend;
    vr.normal = normal;
//    vr.screen_pos =

    return vr;
}

@group(1) @binding(0)
var t_texture: texture_2d<f32>;

@group(1) @binding(1)
var t_sampler: sampler;

@group(2) @binding(0)
var shadow_texture: texture_depth_2d;

@group(2) @binding(1)
var shadow_sampler: sampler;

@fragment
fn frag(
    in: VertexResult
) -> @location(0) vec4<f32> {
    let col1 = textureSample(t_texture, t_sampler, in.tex_coords);
    let col2 = textureSample(t_texture, t_sampler, in.tex_coords2);

    let col = mix(col1, col2, in.blend);

    let uv = in.pos.xy / vec2<f32>(push_constants.fb_width, push_constants.fb_height);

//    let depth = textureSample(shadow_texture, shadow_sampler, uv);

    return col1;
}