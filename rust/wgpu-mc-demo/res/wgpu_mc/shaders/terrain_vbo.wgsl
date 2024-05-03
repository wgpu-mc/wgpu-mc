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

//var<push_constant> push_constants: PushConstants;

@group(0) @binding(0) var<uniform> model_mat: mat4x4<f32>;
@group(0) @binding(1) var<uniform> view_mat: mat4x4<f32>;
@group(0) @binding(2) var<uniform> proj_mat: mat4x4<f32>;

@group(0) @binding(3) var terrain_texture: texture_2d<f32>;
@group(0) @binding(4) var terrain_sampler: sampler;

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
    @location(0) v1: u32,
    @location(1) v2: u32,
    @location(2) v3: u32,
    @location(3) v4: u32,
    @builtin(vertex_index) vertex_index: u32
) -> VertexResult {
    // var uv = uv_offsets.uvs[uv_offset];

    var vr: VertexResult;

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

//    var world_pos = pos + vec3<f32>(f32(push_constants.chunk_x) * 16.0, f32(push_constants.chunk_y) * 16.0, f32(push_constants.chunk_z) * 16.0);
    var world_pos = pos;

    vr.pos = proj_mat * view_mat * vec4(world_pos, 1.0);
    vr.tex_coords = vec2<f32>(u, v);
    vr.light_coords = vec2<u32>(v4 & 15u, (v4 >> 4u) & 15u);
    vr.blend = 0.0;

    return vr;
}

@fragment
fn frag(
    in: VertexResult
) -> @location(0) vec4<f32> {
    let col1 = textureSample(terrain_texture, terrain_sampler, in.tex_coords);
    let col2 = textureSample(terrain_texture, terrain_sampler, in.tex_coords2);

    let col = mix(col1, col2, in.blend);

    return col;
}