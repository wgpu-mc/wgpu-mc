struct Uniforms {
    view_proj: mat4x4<f32>
};

@group(0) @binding(0)
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

//@group(1), binding(0)
//var t_texture: texture_cube<f32>;
//@group(1), binding(1)
//var t_sampler: sampler;

@fragment
fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32> {
    //return textureSample(t_texture, t_sampler, vec3<f32>(0.0, 0.0, 0.0));
    let pos: vec4<f32> = normalize(uniform_data.view_proj * vec4<f32>((frag_pos.x * 2.0) - 1.0, frag_pos.y, 1.0, 1.0));
    let fac: f32 = dot(pos.xyz, vec3<f32>(0.0, 1.0, 0.0));

    return vec4<f32>(fac, fac, fac, 1.0);
}