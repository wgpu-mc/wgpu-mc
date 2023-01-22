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
    chunk_z: i32,
    fb_width: f32,
    fb_height: f32
}

var<push_constant> push_constants: PushConstants;

@group(0) @binding(0)
var<uniform> proj: CameraUniform;

struct VertexResult {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) light_coords: vec2<u32>,
    @location(2) normals: vec4<f32>,
    @location(3) blend: f32,
    @location(4) world_pos: vec3<f32>,
    // @location(4) color: vec4<f32>
};

@vertex
fn vert(
    @location(0) pos_in: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) light_coords: vec2<u32>,
    @location(3) normals: vec4<f32>,
    @location(4) tangent: vec4<f32>,
    @location(5) uv_offset: u32
) -> VertexResult {
    // var uv = uv_offsets.uvs[uv_offset];

    var vr: VertexResult;

    var world_pos = pos_in + vec3<f32>(f32(push_constants.chunk_x) * 16.0, 0.0, f32(push_constants.chunk_z) * 16.0);

    vr.world_pos = world_pos;
    vr.pos = proj.view_proj * vec4<f32>(world_pos, 1.0);
    vr.tex_coords = tex_coords;
    vr.light_coords = light_coords;
    vr.blend = 1.0;
    // vr.normal = normal;
    // vr.color = color;

    return vr;
}

@group(1) @binding(0)
var t_texture: texture_2d<f32>;

@group(1) @binding(1)
var t_sampler: sampler;
@group(2) @binding(0)
var t_lightmap: texture_2d<f32>;
@group(2) @binding(1)
var t_lightmap_sampler: sampler;

// fn minecraft_sample_lighting(uv: vec2<i32> ) -> vec4<f32> {
//     let float_uv = vec2<f32>(uv);
//     let v: vec2<f32> = clamp(float_uv / 256.0, vec2(0.5 / 16.0), vec2(15.5 / 16.0));

//     return textureSample(t_lightmap, t_lightmap_sampler, v);
// }
fn minecraft_sample_lighting(uv: vec2<u32> ) -> f32 {
    return f32((uv.x + uv.y) / 32u) / 32.0;
}
@fragment
fn frag(
    in: VertexResult
) -> @location(0) vec4<f32> {
    let col1 = textureSample(t_texture, t_sampler, in.tex_coords);
    let light = minecraft_sample_lighting(in.light_coords);
    // let light = textureSample(t_lightmap, t_sampler, in.light_coords);
    
    // let col2 = textureSample(t_texture, t_sampler, in.tex_coords2);

    // let col = mix(col1, col2, in.blend);

    return vec4<f32>(light + 0.1, light + 0.1, light + 0.1, 1.0) * col1;
    // return col1;
}
