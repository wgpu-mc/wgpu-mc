use std::collections::HashMap;
use std::fmt::Write;
use std::str::FromStr;
use codespan_reporting::files::SimpleFiles;
use glsl::parser::Parse;
use glsl::syntax::{BinaryOp, Block, Declaration, Expr, ExternalDeclaration, FunIdentifier, LayoutQualifierSpec, Preprocessor, PreprocessorPragma, ShaderStage, SingleDeclaration, StorageQualifier, TranslationUnit, TypeQualifier, TypeQualifierSpec, TypeSpecifierNonArray};
use glsl::transpiler::glsl::{show_expr, show_translation_unit};
use glsl::visitor::{HostMut, Visit, VisitorMut};
use wgpu_mc_jni::preprocessing;
use wgpu_mc_jni::preprocessing::{shim_samplers, IncrementingAnnotator, OrphanDestroyer, RewriteFetches, SamplerBufferRewriter, UniformAnnotator};
fn main() {
    let mut vert_stage = ShaderStage::parse(r#"
#version 330

#line 0 1
/*#version 330*/

layout(std140) uniform Fog {
    vec4 FogColor;
    float FogEnvironmentalStart;
    float FogEnvironmentalEnd;
    float FogRenderDistanceStart;
    float FogRenderDistanceEnd;
    float FogSkyEnd;
    float FogCloudsEnd;
};

float linear_fog_value(float vertexDistance, float fogStart, float fogEnd) {
    if (vertexDistance <= fogStart) {
        return 0.0;
    } else if (vertexDistance >= fogEnd) {
        return 1.0;
    }

    return (vertexDistance - fogStart) / (fogEnd - fogStart);
}

float total_fog_value(float sphericalVertexDistance, float cylindricalVertexDistance, float environmentalStart, float environmantalEnd, float renderDistanceStart, float renderDistanceEnd) {
    return max(linear_fog_value(sphericalVertexDistance, environmentalStart, environmantalEnd), linear_fog_value(cylindricalVertexDistance, renderDistanceStart, renderDistanceEnd));
}

vec4 apply_fog(vec4 inColor, float sphericalVertexDistance, float cylindricalVertexDistance, float environmentalStart, float environmantalEnd, float renderDistanceStart, float renderDistanceEnd, vec4 fogColor) {
    float fogValue = total_fog_value(sphericalVertexDistance, cylindricalVertexDistance, environmentalStart, environmantalEnd, renderDistanceStart, renderDistanceEnd);
    return vec4(mix(inColor.rgb, fogColor.rgb, fogValue * fogColor.a), inColor.a);
}

float fog_spherical_distance(vec3 pos) {
    return length(pos);
}

float fog_cylindrical_distance(vec3 pos) {
    float distXZ = length(pos.xz);
    float distY = abs(pos.y);
    return max(distXZ, distY);
}
#line 0 2
/*#version 330*/

layout(std140) uniform DynamicTransforms {
    mat4 ModelViewMat;
    vec4 ColorModulator;
    vec3 ModelOffset;
    mat4 TextureMat;
};
#line 0 3
/*#version 330*/

layout(std140) uniform Projection {
    mat4 ProjMat;
};

vec4 projection_from_position(vec4 position) {
    vec4 projection = position * 0.5;
    projection.xy = vec2(projection.x + projection.w, projection.y + projection.w);
    projection.zw = position.zw;
    return projection;
}
#line 5 0

in vec3 Position;
in vec4 Color;
in vec2 UV0;
in ivec2 UV2;
in vec3 Normal;

out float sphericalVertexDistance;
out float cylindricalVertexDistance;
out vec4 vertexColor;
out vec2 texCoord0;

void main() {
    gl_Position = ProjMat * ModelViewMat * vec4(Position, 1.0);

    sphericalVertexDistance = fog_spherical_distance(Position);
    cylindricalVertexDistance = fog_cylindrical_distance(Position);
    vertexColor = Color;
    texCoord0 = UV0;
}
    "#).unwrap();

    // dbg!(&vert_stage);

    let mut uniform_map = HashMap::new();
    uniform_map.insert("Fog".into(), 0);
    uniform_map.insert("DynamicTransforms".into(), 1);
    uniform_map.insert("Projection".into(), 2);
    // uniform_map.insert("Fog".into(), 0);
    // uniform_map.insert("Fog".into(), 0);

    let mut out_annotator = IncrementingAnnotator {
        offset: 0,
        target: StorageQualifier::Out,
        found: false,
        insert_location: None,
        map: HashMap::new(),
    };

    let mut uniform_annotator = UniformAnnotator {
        uniform_found: false,
        uniform_set: None,
        uniform_sets: uniform_map.clone(),
        active: false,
    };

    let mut in_annotator = IncrementingAnnotator {
        offset: 0,
        target: StorageQualifier::In,
        found: false,
        insert_location: None,
        map: Default::default(),
    };



    vert_stage.visit_mut(&mut uniform_annotator);

    let mut rewriter = SamplerBufferRewriter {
        is_sampler_buffer: false,
        set: 0,
        binding: 0,
        buffers: vec![],
    };

    vert_stage.visit_mut(&mut rewriter);
    vert_stage.visit_mut(&mut RewriteFetches {
        buffers: &rewriter.buffers,
    });

    vert_stage.visit_mut(&mut out_annotator);
    vert_stage.visit_mut(&mut in_annotator);

    let mut out = String::new();

    shim_samplers(&mut vert_stage, true);

    show_translation_unit(&mut out, &vert_stage);

    println!("## Vert ##\n{out}\n");
}