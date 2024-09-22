struct VO {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) use_uv: u32
}

struct PushConstants {
    projection: mat4x4<f32>,
    color: vec4<f32>
}

var<push_constant> push_constants: PushConstants;

@group(0) @binding(0)
var t_texture: texture_2d<f32>;

@group(1) @binding(0)
var t_sampler: sampler;

@vertex
fn vert(
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) use_uv: u32
) -> VO {
    var vo: VO;
    vo.pos = push_constants.projection * vec4<f32>(pos, 1.0);
    vo.uv = uv;
    vo.color = push_constants.color * color;
    vo.use_uv = use_uv;
    vo.pos.z = 0.1;

    return vo;
}

@fragment
fn frag(in: VO) -> @location(0) vec4<f32> {
    var color_mul_uv = in.color * textureSample(t_texture, t_sampler, in.uv);

    if(in.use_uv == 1u) {
        return color_mul_uv;
    } else {
        return in.color;
    }
}