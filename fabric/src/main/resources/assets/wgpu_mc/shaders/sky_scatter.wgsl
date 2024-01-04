struct VO {
    @builtin(position) pos: vec4<f32>,
    @location(0) vertex_distance: f32,
    @location(1) og_pos: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> view: mat4x4<f32>;

@group(2) @binding(0)
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

@vertex
fn vert(
    @location(0) pos: vec3<f32>,
) -> VO {
    var vo: VO;
    vo.og_pos = pos;
    vo.pos = projection * view * model * vec4<f32>(pos, 1.0);
    vo.vertex_distance = fog_distance(pos);
    return vo;
}

fn fog_distance(pos: vec3<f32>) -> f32 {
    var model_view = model * view;
    if data.fog_shape == 0.0 {
        return length((model_view * vec4(pos, 1.0)).xyz);
    }

    var dist_xz = length((model_view * vec4(pos.x, 0.0, pos.z, 1.0)).xyz);
    var dist_y = length((model_view * vec4(0.0, pos.y, 0.0, 1.0)).xyz);
    return max(dist_xz, dist_y);
}

@fragment
fn frag(in: VO) -> @location(0) vec4<f32> {
    // Light sky
    if in.og_pos.y == 16.0 {
        return linear_fog(vec4<f32>(
            data.color_modulator_r,
            data.color_modulator_g,
            data.color_modulator_b,
            1.0
        ), in.vertex_distance);
    }

    // Dark sky
    return linear_fog(vec4<f32>(
        data.color_modulator_r,
        data.color_modulator_g,
        data.color_modulator_b,
        1.0
    ), in.vertex_distance);
    
}

fn linear_fog(color: vec4<f32>, vertex_distance: f32) -> vec4<f32> {
    if vertex_distance <= data.fog_start {
        return color;
    }

    var fog_value = 1.0;
    if vertex_distance < data.fog_end { 
        fog_value = smoothstep(data.fog_start, data.fog_end, vertex_distance);
    }

    return vec4(mix(color.rgb, vec3(data.fog_color_r, data.fog_color_g, data.fog_color_b), fog_value * data.fog_color_a), color.a);
}