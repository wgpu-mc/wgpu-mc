struct CameraUniform {
    view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> projection: CameraUniform;

struct VertexResult {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] tex_uv: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] pos_in: vec3<f32>,
    [[location(1)]] color: u32,
    [[location(2)]] tex_uv: vec2<f32>,
    [[location(3)]] light_uv: u32
) -> VertexResult {
    var vr: VertexResult;
    vr.pos = projection.view_proj * vec4<f32>(pos_in, 1.0);
    vr.color = vec4<f32>(f32(color & 0xffu) / 255.0, f32((color >> 8u) & 0xffu) / 255.0, f32((color >> 16u) & 0xffu) / 255.0, f32((color >> 24u) & 0xffu) / 255.0);
    vr.tex_uv = tex_uv;

    return vr;
}

[[group(1), binding(0)]]
var t_texture: texture_2d<f32>;

[[group(1), binding(1)]]
var t_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexResult) -> [[location(0)]] vec4<f32> {
    return textureSample(t_texture, t_sampler, in.tex_uv) * in.color;
}