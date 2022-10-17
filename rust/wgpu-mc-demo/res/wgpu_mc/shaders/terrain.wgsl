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

@group(1) @binding(0)
var<uniform> camera_uniform: CameraUniform;

// @group(2), binding(0)
// var<storage> uv_offsets: UVs;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_coords2: vec2<f32>,
    @location(2) blend: f32,
    @location(3) normal: vec3<f32>
};

@vertex
fn vs_main(
    @location(0) pos_in: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(6) uv_offset: u32
) -> VertexResult {
    // var uv = uv_offsets.uvs[uv_offset];

    var vr: VertexResult;
    vr.pos = camera_uniform.view_proj * vec4<f32>(pos_in, 1.0);
    vr.tex_coords = tex_coords;
    vr.tex_coords2 = tex_coords;
    vr.blend = 1.0;
    // vr.tex_coords = tex_coords + uv.uv1;
    // vr.tex_coords2 = tex_coords + uv.uv2;
    // vr.blend = uv.blend;
    vr.normal = normal;

    return vr;
}

@group(0) @binding(0)
var t_texture: texture_2d<f32>;

@group(0) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: VertexResult) -> @location(0) vec4<f32> {
    let col1 = textureSample(t_texture, t_sampler, in.tex_coords);
    let col2 = textureSample(t_texture, t_sampler, in.tex_coords2);

    let col = mix(col1, col2, in.blend);
    return col;
}