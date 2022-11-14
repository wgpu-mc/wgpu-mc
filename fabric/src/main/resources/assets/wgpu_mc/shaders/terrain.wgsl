struct CameraUniform {
    view_proj: mat4x4<f32>
};

@group(1) @binding(0)
var<uniform> camera_uniform: CameraUniform;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>
};

@vertex
fn vs_main(
    @location(0) pos_in: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>
) -> VertexResult {
    var vr: VertexResult;
    vr.pos = camera_uniform.view_proj * vec4<f32>(pos_in, 1.0);
    vr.tex_coords = tex_coords;
    vr.normal = normal;

    return vr;
}

@group(0) @binding(0)
var t_texture: texture_2d<f32>;

@group(0) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: VertexResult) -> @location(0) vec4<f32> {
    var normal_shading: f32 = (dot(vec3<f32>(0.0, 1.0, 1.0), in.normal) * 0.5) + 0.5;
    var color = textureSample(t_texture, t_sampler, in.tex_coords);
    // return vec4<f32>(color.r * normal_shading, color.g * normal_shading, color.b * normal_shading, color.a);
    return color;
}