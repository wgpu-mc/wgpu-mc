use std::collections::HashMap;
use std::fmt::Write;
use std::str::FromStr;
use codespan_reporting::files::SimpleFiles;
use glsl::parser::Parse;
use glsl::syntax::{BinaryOp, Block, Declaration, Expr, ExternalDeclaration, FunIdentifier, LayoutQualifierSpec, Preprocessor, PreprocessorPragma, ShaderStage, SingleDeclaration, StorageQualifier, TranslationUnit, TypeQualifier, TypeQualifierSpec, TypeSpecifierNonArray};
use glsl::transpiler::glsl::{show_expr, show_translation_unit};
use glsl::visitor::{HostMut, Visit, VisitorMut};
use wgpu_mc_jni::preprocessing;
use wgpu_mc_jni::preprocessing::{shim_samplers, IncrementingAnnotator, OrphanDestroyer, RemovePointSize, RewriteFetches, SamplerBufferRewriter, UniformAnnotator};
fn main() {
    let mut vert_stage = ShaderStage::parse(r#"
#version 330

#line 0 1
/*#version 330*/

uniform vec2 Fog;
uniform sampler2D Sampler;
uniform vec2 Projection;

void main() {
    gl_PointSize = 1;
}
    "#).unwrap();

    let mut uniform_map = HashMap::new();
    uniform_map.insert("Fog".into(), (0, 0));
    uniform_map.insert("Sampler_wm_texshim".into(), (0, 1));
    uniform_map.insert("Sampler_wm_sampler".into(), (0, 2));
    uniform_map.insert("Projection".into(), (1, 0));

    let mut out_annotator = IncrementingAnnotator {
        offset: 0,
        target: StorageQualifier::Out,
        found: false,
        insert_location: None,
        map: HashMap::new(),
    };

    let mut uniform_annotator = UniformAnnotator {
        uniform_found: false,
        uniform_binding: None,
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

    let mut rewriter = SamplerBufferRewriter {
        is_sampler_buffer: false,
        buffers: vec![],
        uniform_sets: &uniform_map,
    };

    vert_stage.visit_mut(&mut rewriter);
    vert_stage.visit_mut(&mut RewriteFetches {
        buffers: &rewriter.buffers,
    });

    vert_stage.visit_mut(&mut RemovePointSize { is_point_var: false });

    vert_stage.visit_mut(&mut out_annotator);
    vert_stage.visit_mut(&mut in_annotator);

    let mut out = String::new();

    shim_samplers(&mut vert_stage, true);
    vert_stage.visit_mut(&mut uniform_annotator);

    show_translation_unit(&mut out, &vert_stage);

    println!("## Vert ##\n{out}\n");
}