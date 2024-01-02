struct VO {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) og_pos: vec3<f32>,
}

const PI = 3.14159265;

@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> view: mat4x4<f32>;

@group(2) @binding(0)
var<uniform> model: mat4x4<f32>;

@group(3) @binding(0) var sun_texture: texture_2d<f32>;
@group(3) @binding(1) var sun_sampler: sampler;

@group(4) @binding(0) var moon_texture: texture_2d<f32>;
@group(4) @binding(1) var moon_sampler: sampler;

struct PushConstants {
    r: f32,
    g: f32,
    b: f32,
    angle: f32,
    brightness: f32
}

var<push_constant> data: PushConstants;

fn rotateX(degrees: f32) -> mat4x4<f32> {
    var theta = radians(degrees);
    var c = cos(theta);
    var s = sin(theta);
    return mat4x4<f32>(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, c, s, 0.0),
        vec4(0.0, -s, c, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0),  
    );
}

fn rotateY(degrees: f32) -> mat4x4<f32> {
    var theta = radians(degrees);
    var c = cos(theta);
    var s = sin(theta);
    return mat4x4<f32>(
        vec4(c, 0.0, -s, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(s, 0.0, c, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0),  
    );
}

fn radians(degrees: f32) -> f32 {
    return degrees * (PI/180.0);
}

fn identity() -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0), 
    );
}

@vertex
fn vert(
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>
) -> VO {
    var vo: VO;
    vo.og_pos = pos;

    var transformation_matrix = model * rotateY(-90.0) * rotateX(data.angle * 360.0);
    var dir = transformation_matrix * vec4<f32>(pos, 1.0);

    vo.pos = projection * view * vec4<f32>(dir.xyz, 1.0);
    vo.tex_coords = tex_coords;
    return vo;
}

@fragment
fn frag(in: VO) -> @location(0) vec4<f32> {
    if(in.og_pos.y > 0.0) {
        return textureSample(sun_texture, sun_sampler, in.tex_coords);
    } else {
        return textureSample(moon_texture, moon_sampler, in.tex_coords);
    }
}