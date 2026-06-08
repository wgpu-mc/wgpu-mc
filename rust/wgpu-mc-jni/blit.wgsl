struct UV {
    uv1: vec2<f32>,
    uv2: vec2<f32>,
    blend: f32,
    padding: f32
};

struct UVs {
    uvs: array<UV>
};

struct ChunkOffset {
    x: i32,
    z: i32
}

@group(0) @binding(0) var t_texture: texture_2d<f32>;
@group(0) @binding(1) var t_sampler: sampler;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>
};

var<push_constant> section_pos: vec3i;

@vertex
fn vert(
    @builtin(vertex_index) vi: u32
) -> VertexResult {
    var vr: VertexResult;

    var verts = array(
            vec4(-1., -1., 0., 1.),
            vec4(1., 1., 1., 0.),
            vec4(-1., 1., 0., 0.),
            vec4(-1., -1., 0., 0.),
            vec4(1., -1., 1., 0.),
            vec4(1., 1., 1., 0.)
        );

    var pos = verts[vi];

    vr.pos = vec4(pos.x, pos.y, 0.0, 1.0);
    vr.uv = vec2(pos.z, pos.w);

    return vr;
}

fn minecraft_sample_lighting(uv: vec2<f32>) -> vec3<f32> {
    return mix(uv.x * vec3(0.32156, 0.32156, 0.5) * 0.5 + uv.y * 0.5, vec3(1.0, 1.0, 1.0), uv.y);
}

@fragment
fn frag(
    in: VertexResult
) -> @location(0) vec4<f32> {
    let col = textureSample(t_texture, t_sampler, in.uv);

    return col;
}
