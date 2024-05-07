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
    total_chunk_sections: u32
}

struct Buffer {
    buffer: array<u32>
}

var<push_constant> push_constants: PushConstants;

@group(0) @binding(0) var<uniform> mat4_model: mat4x4<f32>;
@group(0) @binding(1) var<uniform> mat4_view: mat4x4<f32>;
@group(0) @binding(2) var<uniform> mat4_persp: mat4x4<f32>;

@group(0) @binding(3) var t_texture: texture_2d<f32>;
@group(0) @binding(4) var t_sampler: sampler;

@group(1) @binding(0) var<storage> vertex_data: binding_array<Buffer>;
@group(1) @binding(1) var<storage> index_data: binding_array<Buffer>;
@group(1) @binding(2) var<storage> ranges: array<u32>;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_coords2: vec2<f32>,
    @location(2) blend: f32,
    @location(3) normal: vec3<f32>,
    @location(4) world_pos: vec3<f32>,
    @location(5) @interpolate(flat) light_coords: vec2<f32>,
    @location(6) section: u32
};

fn search_iter(size: ptr<function, u32>, left: ptr<function, u32>, right: ptr<function, u32>, section_index: ptr<function, u32>, done: ptr<function, bool>, vertex_index: u32) {
    var mid = *left + *size / 2;

    var begin_range = ranges[mid * 5];
    var end_range = ranges[mid * 5 + 1];

    *left = select(*left, mid + 1, end_range < vertex_index);
    *right = select(*right, mid, begin_range > vertex_index);

    *section_index = select(mid, *section_index, *done);

    *done |= vertex_index >= begin_range && vertex_index < end_range;

    *size = *right - *left;
}

@vertex
fn vert(
    @builtin(vertex_index) vertex_index: u32
) -> VertexResult {
    var vr: VertexResult;

    var section_index: u32 = 0;

    var size = push_constants.total_chunk_sections;
    var left: u32 = 0u;
    var right: u32 = size;
    var done: bool = false;

    //18 binary search iterations, supports up to 2^18 chunk sections
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);
    search_iter(&size, &left, &right, &section_index, &done, vertex_index);

    vr.section = section_index;

    var section_vert_start = ranges[section_index * 5];
    var section_x: u32 = ranges[section_index * 5 + 2];
    var section_y: u32 = ranges[section_index * 5 + 3];
    var section_z: u32 = ranges[section_index * 5 + 4];

    var index: u32 = index_data[0].buffer[vertex_index - section_vert_start];

    var v1 = vertex_data[section_index].buffer[index * 4u];
    var v2 = vertex_data[section_index].buffer[(index * 4u) + 1u];
    var v3 = vertex_data[section_index].buffer[(index * 4u) + 2u];
    var v4 = vertex_data[section_index].buffer[(index * 4u) + 3u];

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

    var world_pos = pos + vec3<f32>(f32(section_x) * 16.0, f32(section_y) * 16.0, f32(section_z) * 16.0);

    vr.pos = mat4_persp * mat4_view * vec4(world_pos, 1.0);
    vr.tex_coords = vec2<f32>(u, v);
    vr.tex_coords2 = vec2(0.0, 0.0);
    vr.world_pos = world_pos;

    var light_coords = vec2<u32>(v4 & 15u, (v4 >> 4u) & 15u);
    vr.light_coords = vec2(f32(light_coords.x) / 15.0, f32(light_coords.y) / 15.0);

    vr.blend = 0.0;

    return vr;
}

fn minecraft_sample_lighting(uv: vec2<u32> ) -> f32 {
    return f32(max(uv.x, uv.y)) / 15.0;
}

@fragment
fn frag(
    in: VertexResult
) -> @location(0) vec4<f32> {
    let col1 = textureSample(t_texture, t_sampler, in.tex_coords);

//    let light = textureSample(lightmap_texture, lightmap_sampler, vec2(max(in.light_coords.x, in.light_coords.y), 0.0));
//    let light = max(in.light_coords.x, in.light_coords.y);

    return col1;
}
