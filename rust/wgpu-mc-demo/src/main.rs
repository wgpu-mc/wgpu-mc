use std::collections::HashMap;
use std::fmt::Write;
use std::str::FromStr;
use codespan_reporting::files::SimpleFiles;
use cyntax_common::ast::{Keyword, PreprocessingToken, Whitespace};
use cyntax_common::ctx::ParseContext;
use cyntax_common::spanned::Spanned;
use glsl::parser::Parse;
use glsl::syntax::{Block, ExternalDeclaration, Preprocessor, PreprocessorPragma, ShaderStage, SingleDeclaration, StorageQualifier, TranslationUnit, TypeQualifier, TypeQualifierSpec};
use glsl::transpiler::glsl::show_translation_unit;
use glsl::visitor::{HostMut, Visit, VisitorMut};
use wgpu_mc_jni::preprocessing;
use wgpu_mc_jni::preprocessing::{shim_samplers, IncrementingAnnotator, OrphanDestroyer, UniformAnnotator};


fn main() {
    let mut vert_stage = ShaderStage::parse(r#"

uniform samplerCube splitMe;

vec3 sampleIt(samplerCube yuppp) {
    return texture(yuppp, vec3(0.0, 0.0, 0.0));
}

void main() {
    gl_FragColor = sampleIt(splitMe);
}

"#).unwrap();

    let mut uniform_map = HashMap::new();
    uniform_map.insert("splitMe".into(), 0);

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
    vert_stage.visit_mut(&mut out_annotator);
    vert_stage.visit_mut(&mut in_annotator);

    let mut out = String::new();

    shim_samplers(&mut vert_stage, true);

    show_translation_unit(&mut out, &vert_stage);

    println!("## Vert ##\n{out}\n");
}