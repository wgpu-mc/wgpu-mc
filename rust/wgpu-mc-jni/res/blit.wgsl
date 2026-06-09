const TRI = array(
    vec4(-1.0, 3.0, 0.0, 1.0),
    vec4(3.0, -1.0, 0.0, 1.0),
    vec4(-1.0, -1.0, 0.0, 1.0)
);

const UV = array(
    vec2(0.0, 2.0),
    vec2(2.0, 0.0),
    vec2(0.0, 0.0),
);

@group(0) @binding(0) var t: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;

struct Vert {
    @builtin(position) pos: vec4<f32>,
    @location(0) col: vec4<f32>,
    @location(1) uv: vec2<f32>
}

@fragment
fn frag(
    in: Vert
) -> @location(0) vec4<f32> {
    return textureSample(t, s, in.uv);
}

@vertex
fn vert(
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) color: u32
) -> Vert {
    var v: Vert;

    v.pos = TRI[index];
    v.uv = UV[index];
    v.col = unpack4x8unorm(color).xyzw;

    return v;
}