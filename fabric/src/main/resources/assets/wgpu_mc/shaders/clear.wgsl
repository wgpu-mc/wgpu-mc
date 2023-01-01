struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct PushConstant {
    color: vec3<f32>
}

var<push_constant> pc: PushConstant;

const VERTS = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
);

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32
) -> VertexResult {
    var vr: VertexResult;
    vr.pos = vec4<f32>(VERTS[index], 0.0, 1.0);
    vr.color = pc.color;

    return vr;
}

@fragment
fn fs_main(in: VertexResult) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}