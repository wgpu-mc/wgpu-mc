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

@group(0) @binding(0) var<uniform> mat4_terrain_shift: mat4x4<f32>;
@group(0) @binding(1) var<uniform> mat4_view: mat4x4<f32>;
@group(0) @binding(2) var<uniform> mat4_persp: mat4x4<f32>;

@group(0) @binding(3) var t_texture: texture_2d<f32>;
@group(0) @binding(4) var t_sampler: sampler;

@group(1) @binding(0) var<storage> chunk_data: array<u32>;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_coords2: vec2<f32>,
    @location(2) blend: f32,
    @location(3) normal: vec3<f32>,
    @location(4) world_pos: vec3<f32>,
    @location(5) @interpolate(flat) light_coords: vec2<f32>,
    @location(6) section: u32,
    @location(7) ao: f32
};

var<push_constant> section_pos: vec3i;

@vertex
fn vert(
    @builtin(vertex_index) vi: u32,
    @builtin(instance_index) base_vertex: u32
) -> VertexResult {
    var vr: VertexResult;
    let id = vi*4u+base_vertex;
    let v1 = chunk_data[id];
    let v2 = chunk_data[id + 1u];
    let v3 = chunk_data[id + 2u];
    let v4 = chunk_data[id + 3u];

    var x: f32 = f32(v1 & 0xffu) * 0.0625;
    var y: f32 = f32((v1 >> 8u) & 0xffu) * 0.0625;
    var z: f32 = f32((v1 >> 16u) & 0xffu) * 0.0625;

    var u: f32 = f32((v2 >> 16u) & 0xffffu) * 0.00048828125;
    var v: f32 = f32(v3 & 0xffffu) * 0.00048828125;
    var ao: f32 = f32((v4 >> 8u) & 0xff) * 0.25;

    if(((v3 >> 29u) & 1u) == 1u) {
        x = 16.0;
    }

    if(((v3 >> 30u) & 1u) == 1u) {
        y = 16.0;
    }

    if((v3 >> 31u) == 1u) {
        z = 16.0;
    }
    var pos = vec3<f32>(x, y, z);

    var world_pos = pos + vec3<f32>(f32(section_pos.x) * 16.0, f32(section_pos.y) * 16.0, f32(section_pos.z) * 16.0);

    vr.pos = mat4_persp * mat4_view * mat4_terrain_shift * vec4(world_pos, 1.0);
    vr.tex_coords = vec2<f32>(u, v);
    vr.tex_coords2 = vec2(0.0, 0.0);
    vr.world_pos = world_pos;

    var light_coords = vec2<u32>(v4 & 15u, (v4 >> 4u) & 15u);
    vr.light_coords = vec2(f32(light_coords.x) / 15.0, f32(light_coords.y) / 15.0);

    vr.blend = 0.0;
    vr.ao = ao;

    return vr;
}

fn minecraft_sample_lighting(uv: vec2<u32> ) -> f32 {
    return f32(max(uv.x, uv.y)) / 15.0;
}

@fragment
fn frag(
    in: VertexResult
) -> @location(0) vec4<f32> {
    var ao: f32 = (in.ao * 0.7) + 0.3;
    let col = vec4(ao, ao, ao, 1.0) * textureSample(t_texture, t_sampler, in.tex_coords);

//    let light = textureSample(lightmap_texture, lightmap_sampler, vec2(max(in.light_coords.x, in.light_coords.y), 0.0));
    let light = max(in.light_coords.x, in.light_coords.y);

    if(col.a == 0.0f){
        discard;
    }
    return vec4(col.rgb*light,col.a);
}
