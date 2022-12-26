struct CameraUniform {
    view_proj: mat4x4<f32>
};

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

var<push_constant> chunk_offset: ChunkOffset;

@group(0) @binding(0)
var<uniform> camera_uniform: CameraUniform;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_coords2: vec2<f32>,
    @location(2) blend: f32,
    @location(3) normal: vec3<f32>
};

@vertex
fn vert(
    @location(0) pos_in: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(6) uv_offset: u32
) -> VertexResult {
    var vr: VertexResult;

    var world_pos = pos_in + vec3<f32>(f32(chunk_offset.x) * 16.0, 0.0, f32(chunk_offset.z) * 16.0);

    vr.pos = camera_uniform.view_proj * vec4<f32>(world_pos, 1.0);
    vr.tex_coords = tex_coords;
    vr.tex_coords2 = tex_coords;
    vr.blend = 1.0;

    vr.normal = normal;

    return vr;
}

@fragment
fn frag(in: VertexResult) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}