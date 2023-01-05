struct Matrix4 {
    mat4: mat4x4<f32>
};

struct PushConstants {
    fb_width: f32,
    fb_height: f32
}

var<push_constant> push_constants: PushConstants;

@group(0) @binding(0)
var<uniform> rotation: Matrix4;

struct VertexResult {
    @builtin(position) pos: vec4<f32>
};

@vertex
fn vert(
    @location(0) pos_in: vec2<f32>
) -> VertexResult {
    var vr: VertexResult;
    vr.pos = vec4<f32>(pos_in, 0.1, 1.0);

    return vr;
}

@fragment
fn frag(
    in: VertexResult
) -> @location(0) vec4<f32> {
    let uv = in.pos.xy / vec2<f32>(push_constants.fb_width, push_constants.fb_height);
    let uv3d = vec4<f32>(uv, 0.0, 1.0);
    let uv3d_rotated = normalize(rotation.mat4 * uv3d).xyz;
    let uv_up = vec3<f32>(0.0, 0.0, 0.0);
    let blue_mix = dot(uv3d_rotated, uv_up);

    let color = mix(vec3<f32>(77.0 / 255.0, 178.0 / 255.0, 1.0), vec3<f32>(61.0 / 255.0, 135.0 / 255.0, 1.0), blue_mix);

    return vec4<f32>(color, 1.0);
}