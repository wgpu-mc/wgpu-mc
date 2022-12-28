struct VO {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) use_uv: u32
}

@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

@group(1) @binding(0)
var t_texture: texture_2d<f32>;

@group(1) @binding(1)
var t_sampler: sampler;

@vertex
fn vert(
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) use_uv: bool
) -> VO {
    var vo: VO;
    vo.pos = projection * vec4<f32>(pos, 1.0);
    vo.uv = uv;
    vo.color = color;
    vo.use_uv = use_uv;

    return vo;
}

@fragment
fn frag(in: VO) -> @location(0) vec4<f32> {
    if(in.use_uv == 1) {
        return in.color * textureSample(t_texture, t_sampler, in.uv) * in.color;
    } else {
        return in.color;
    }
}