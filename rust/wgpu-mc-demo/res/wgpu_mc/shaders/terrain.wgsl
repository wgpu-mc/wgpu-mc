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

struct PushConstants {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    fb_width: f32,
    fb_height: f32
}

var<push_constant> push_constants: PushConstants;

@group(0) @binding(0) var<uniform> camera_uniform: CameraUniform;

@group(1) @binding(0) var t_texture: texture_2d<f32>;
@group(1) @binding(1) var t_sampler: sampler;

@group(2) @binding(0) var<storage> vertex_data: array<u32>;
@group(3) @binding(0) var<storage> index_data: array<u32>;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_coords2: vec2<f32>,
    @location(2) blend: f32,
    @location(3) normal: vec3<f32>,
    @location(4) world_pos: vec3<f32>,
    @location(5) light_coords: vec2<u32>
};

@vertex
fn vert(
    @builtin(vertex_index) vertex_index: u32
) -> VertexResult {
    // var uv = uv_offsets.uvs[uv_offset];

    var vr: VertexResult;

    var index: u32 = index_data[vertex_index];

    var v1 = vertex_data[index * 4u];
    var v2 = vertex_data[(index * 4u) + 1u];
    var v3 = vertex_data[(index * 4u) + 2u];
    var v4 = vertex_data[(index * 4u) + 3u];

    var x: f32 = f32(v1 & 0xffu) * 0.0625;
    var y: f32 = f32((v1 >> 8u) & 0xffu) * 0.0625;
    var z: f32 = f32((v1 >> 16u) & 0xffu) * 0.0625;

    var u: f32 = f32((v2 >> 16u) & 0xffffu) * 0.00048828125;
    var v: f32 = f32(v3 & 0xffffu) * 0.00048828125;

    if(((v3 >> 61u) & 1u) == 1u) {
        x = 16.0;
    }

    if(((v3 >> 62u) & 1u) == 1u) {
        y = 16.0;
    }

    if((v3 >> 63u) == 1u) {
        z = 16.0;
    }

    var pos = vec3<f32>(x, y, z);

    var world_pos = pos + vec3<f32>(f32(push_constants.chunk_x) * 16.0, f32(push_constants.chunk_y) * 16.0, f32(push_constants.chunk_z) * 16.0);

    vr.pos = camera_uniform.view_proj * vec4(world_pos, 1.0);
    vr.tex_coords = vec2<f32>(u, v);
    vr.light_coords = vec2<u32>(v4 & 15u, (v4 >> 4u) & 15u);
    vr.blend = 0.0;

    return vr;
}

@fragment
fn frag(
    in: VertexResult
) -> @location(0) vec4<f32> {
    let col1 = textureSample(t_texture, t_sampler, in.tex_coords);
    let col2 = textureSample(t_texture, t_sampler, in.tex_coords2);

    let col = mix(col1, col2, in.blend);

    return col1;
}