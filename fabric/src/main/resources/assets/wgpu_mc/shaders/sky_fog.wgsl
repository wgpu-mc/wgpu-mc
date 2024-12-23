struct VO {
    @builtin(position) pos: vec4<f32>,
    @location(0) og_pos: vec3<f32>,
}

const PI = 3.14159265;

@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

@group(0) @binding(1)
var<uniform> view: mat4x4<f32>;

@group(0) @binding(2)
var<uniform> model: mat4x4<f32>;

struct PushConstants {
    angle: f32,
    brightness: f32,
    star_shimmer: f32,
    fog_start: f32,
    fog_end: f32,
    fog_shape: f32,
    fog_color_r: f32,
    fog_color_g: f32,
    fog_color_b: f32,
    fog_color_a: f32,
    color_modulator_r: f32,
    color_modulator_g: f32,
    color_modulator_b: f32,
    dimension_fog_color_r: f32,
    dimension_fog_color_g: f32,
    dimension_fog_color_b: f32,
    dimension_fog_color_a: f32,
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

fn rotateZ(degrees: f32) -> mat4x4<f32> {
    var theta = radians(degrees);
    var c = cos(theta);
    var s = sin(theta);
    return mat4x4<f32>(
        vec4(c, -s, 0.0, 0.0),
        vec4(s, c, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0),  
    );
}

fn radians(degrees: f32) -> f32 {
    return degrees * (PI/180.0);
}

@vertex
fn vert(
    @location(0) pos: vec3<f32>,
) -> VO {
    var vo: VO;
    vo.og_pos = pos;

    var pos_z_modify = 1.0;
    if pos.z != 0.0 {
        pos_z_modify = data.dimension_fog_color_a;
    }

    var f1 = 0.0;

    if sin(data.angle) < 0.0 {
        f1 = 180.0;
    }

    var transformation_matrix = rotateX(90.0) * rotateZ(f1) * rotateZ(90.0);

    vo.pos = projection * view * transformation_matrix * vec4<f32>(pos.xy, pos.z * pos_z_modify, 1.0);
    return vo;
}

@fragment
fn frag(in: VO) -> @location(0) vec4<f32> {
    if in.og_pos.z == 0.0 {
        return vec4<f32>(data.dimension_fog_color_r, data.dimension_fog_color_g, data.dimension_fog_color_b, data.dimension_fog_color_a);
    }

    return vec4<f32>(data.dimension_fog_color_r, data.dimension_fog_color_g, data.dimension_fog_color_b, 0.0);
}
