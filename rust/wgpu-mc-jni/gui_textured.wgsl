struct VO {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>
}

struct DynamicUniforms {
     model_view: mat4x4<f32>,
     color_mod: vec4<f32>,
     model_offset: vec3<f32>,
     texture_mat: mat4x4<f32>,
     line_width: f32
}

@group(0) @binding(0)
var t_texture: texture_2d<f32>;
@group(0) @binding(1)
var t_sampler: sampler;

@group(1) @binding(0)
var<storage> uniforms: DynamicUniforms;

@group(2) @binding(0)
var<storage> projection: mat4x4<f32>;

@vertex
fn vert(
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: u32
) -> VO {
    var vo: VO;

    var r = f32(extractBits(color, 0u, 8u));
    var g = f32(extractBits(color, 8u, 8u));
    var b = f32(extractBits(color, 16u, 8u));
    var a = f32(extractBits(color, 24u, 8u));

    var col = vec4<f32>(r, g, b, a) / 255.0;

    vo.pos = projection * uniforms.model_view * vec4(position.x, position.y, position.z, 1.0);
    vo.color = col;

    return vo;
}

@fragment
fn frag(in: VO) -> @location(0) vec4<f32> {
    return vec4(1.0, 1.0, 1.0, 1.0);

//    var color_mul_uv = in.color * textureSample(t_texture, t_sampler, in.uv);
//
//    if(in.use_uv == 1u) {
//        return color_mul_uv;
//    } else {
//        return in.color;
//    }
}