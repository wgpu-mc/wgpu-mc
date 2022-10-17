struct CameraUniform {
    view_proj: mat4x4<f32>
};

@group(0) @binding(0)
var<uniform> projection: CameraUniform;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
};

@vertex
fn vs_main(
    @location(0) pos_in: vec3<f32>,
    @location(1) tex_coords: vec2<f32>
) -> VertexResult {
    var vr: VertexResult;
    vr.pos = projection.view_proj * vec4<f32>(pos_in, 1.0);
    vr.tex_coords = tex_coords;

    return vr;
}

@group(1) @binding(0)
var t_texture: texture_2d<f32>;

@group(1) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: VertexResult) -> @location(0) vec4<f32> {
    return textureSample(t_texture, t_sampler, in.tex_coords);
}