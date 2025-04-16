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


@group(0) @binding(0) var<uniform> mat4_model: mat4x4<f32>;
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
    @location(5) light_coords: vec2<f32>,
    @location(6) section: u32,
    @location(7) ao: f32,
    @interpolate(flat) @location(8) lc1: vec2<f32>,
    @interpolate(flat) @location(9) lc2: vec2<f32>,
    @interpolate(flat) @location(10) lc3: vec2<f32>,
    @interpolate(flat) @location(11) lc4: vec2<f32>,
    @interpolate(flat) @location(12) ao1: f32,
    @interpolate(flat) @location(13) ao2: f32,
    @interpolate(flat) @location(14) ao3: f32,
    @interpolate(flat) @location(15) ao4: f32,
    @location(16) light_uv: vec2<f32>,
    @interpolate(flat) @location(17) int: u32,
    @location(18) color: vec4<f32>
};

var<push_constant> section_pos: vec3i;

@vertex
fn vert(
    @builtin(vertex_index) vi: u32,
    @builtin(instance_index) base_vertex: u32
) -> VertexResult {
//    var vert1_i = (vi >> 2) << 4;
//    var vert1_i = (vi << 2) & 0xfffffffc;
//    var vert1_i = ((vi >> 2u) << 2u)+base_vertex;

    var offset = vi & 3;
    var vert1_i = vi & ~3u;

    var id = ((vert1_i + offset) << 2u) + base_vertex;

    var vert1_base = ((vert1_i) << 2u) + base_vertex;

    var vert1_v4 = chunk_data[vert1_base + 3u];
    var vert2_v4 = chunk_data[vert1_base + 7u];
    var vert3_v4 = chunk_data[vert1_base + 11u];
    var vert4_v4 = chunk_data[vert1_base + 15u];

    var v1_lc = 0.066666666666667 * vec2(f32(vert1_v4 & 15u), f32((vert1_v4 >> 4u) & 15u));
    var v2_lc = 0.066666666666667 * vec2(f32(vert2_v4 & 15u), f32((vert2_v4 >> 4u) & 15u));
    var v3_lc = 0.066666666666667 * vec2(f32(vert3_v4 & 15u), f32((vert3_v4 >> 4u) & 15u));
    var v4_lc = 0.066666666666667 * vec2(f32(vert4_v4 & 15u), f32((vert4_v4 >> 4u) & 15u));
    var v1_ao = f32((vert1_v4 >> 8u) & 0xff) * 0.333333;
    var v2_ao = f32((vert2_v4 >> 8u) & 0xff) * 0.333333;
    var v3_ao = f32((vert3_v4 >> 8u) & 0xff) * 0.333333;
    var v4_ao = f32((vert4_v4 >> 8u) & 0xff) * 0.333333;

    var uv = array<vec2<f32>,4>(
            vec2(1.0,1.0),
            vec2(0.0,1.0),
            vec2(0.0,0.0),
            vec2(1.0,0.0));

    var light_uv = uv[vi & 3];

    var vr: VertexResult;
    vr.int = vi & 3;
    vr.lc1 = v1_lc;
    vr.lc2 = v2_lc;
    vr.lc3 = v3_lc;
    vr.lc4 = v4_lc;
    vr.ao1 = v1_ao;
    vr.ao2 = v2_ao;
    vr.ao3 = v3_ao;
    vr.ao4 = v4_ao;

    vr.light_uv = light_uv;

    var v1 = chunk_data[id];
    var v2 = chunk_data[id + 1u];
    var v3 = chunk_data[id + 2u];
    var v4 = chunk_data[id + 3u];

    var x: f32 = f32(v1 & 0xffu) * 0.0625;
    var y: f32 = f32((v1 >> 8u) & 0xffu) * 0.0625;
    var z: f32 = f32((v1 >> 16u) & 0xffu) * 0.0625;

    var r: u32 = (v1 >> 24u) & 0xff;
    var g: u32 = (v2 & 0xff);
    var b: u32 = (v2 >> 8u) & 0xff;

    vr.color = vec4(f32(r) * 0.003921568627451, f32(g) * 0.003921568627451, f32(b) * 0.003921568627451, 1.0);

    var ao: f32 = f32((v4 >> 8u) & 0xff) * 0.33333;

    var u: f32 = f32((v2 >> 16u) & 0xffffu) * 0.00048828125;
    var v: f32 = f32(v3 & 0xffffu) * 0.00048828125;

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

    vr.pos = mat4_persp * mat4_view * mat4_model * vec4(world_pos, 1.0);
    vr.tex_coords = vec2<f32>(u, v);
    vr.tex_coords2 = vec2(0.0, 0.0);
    vr.world_pos = world_pos;
    vr.ao = ao;

    var light_coords = vec2<u32>(v4 & 15u, (v4 >> 4u) & 15u);
    vr.light_coords = 0.066666666666666 * vec2(f32(light_coords.x), f32(light_coords.y));

    vr.blend = 0.0;

    return vr;
}

fn minecraft_sample_lighting(uv: vec2<f32>) -> vec3<f32> {
    return mix(uv.x * vec3(0.32156, 0.32156, 0.5) * 0.5 + uv.y * 0.5, vec3(1.0, 1.0, 1.0), uv.y);
}

@fragment
fn frag(
    in: VertexResult
) -> @location(0) vec4<f32> {
//    var ao: f32 = (in.ao * 0.7) + 0.3;

    var lc = mix(mix(in.lc3, in.lc4, in.light_uv.x), mix(in.lc2, in.lc1, in.light_uv.x), in.light_uv.y);
    var ao = 0.6 + 0.4 * mix(mix(in.ao3, in.ao4, in.light_uv.x), mix(in.ao2, in.ao1, in.light_uv.x), in.light_uv.y);
//    var ao = mix(mix(0.0, 0.0, in.light_uv.x), mix(0.0, 1.0, in.light_uv.x), in.light_uv.y);

    var light = max(lc.x, lc.y) * 0.7 + 0.3;

    let col = in.color * vec4(light, light, light, 1.0) * vec4(ao, ao, ao, 1.0) * textureSample(t_texture, t_sampler, in.tex_coords);

//    let light = textureSample(lightmap_texture, lightmap_sampler, vec2(max(in.light_coords.x, in.light_coords.y), 0.0));

    if(col.a == 0.0f){
        discard;
    }
    return col;
}
