struct VO {
    @builtin(position) pos: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

struct PushConstants {
    r: f32,
    g: f32,
    b: f32,
    angle: f32,
    brightness: f32
}

var<push_constant> data: PushConstants;

@vertex
fn vert(
    @location(0) pos: vec3<f32>,
) -> VO {
    var vo: VO;
    vo.pos = vec4<f32>(pos, 1.0);

    return vo;
}

@fragment
fn frag(in: VO) -> @location(0) vec4<f32> {
    return vec4<f32>(data.r, data.g, data.b, 1.0);
}