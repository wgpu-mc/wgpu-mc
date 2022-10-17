struct Uniforms {
    view_proj: mat4x4<f32>
};

@group(1) @binding(0)
var<uniform> uniform_data: Uniforms;

struct VertexResult {
    @builtin(position) pos: vec4<f32>
};

@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VertexResult {
    var vr: VertexResult;
    vr.pos = vec4<f32>(pos, 1.0);

    return vr;
}

@group(0) @binding(0)
var t_texture: texture_cube<f32>;
@group(0) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return textureSample(t_texture, t_sampler, vec3<f32>(0.0, 0.0, 0.0));
}