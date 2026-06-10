use codespan_reporting::files::SimpleFiles;
use glsl::parser::Parse;
use glsl::syntax::{BinaryOp, Block, Declaration, Expr, ExternalDeclaration, FunIdentifier, LayoutQualifierSpec, Preprocessor, PreprocessorPragma, PreprocessorVersion, ShaderStage, SingleDeclaration, StorageQualifier, TranslationUnit, TypeQualifier, TypeQualifierSpec, TypeSpecifierNonArray};
use glsl::transpiler::glsl::{show_expr, show_translation_unit};
use glsl::visitor::{HostMut, Visit, VisitorMut};
use std::collections::HashMap;
use std::fmt::Write;
use std::str::FromStr;
use wgpu_mc_jni::preprocessing;
use wgpu_mc_jni::preprocessing::{IncrementingAnnotator, OrphanDestroyer, RemovePointSize, RewriteFetches, SamplerBufferRewriter, UniformAnnotator, shim_samplers, MatrixPatcher, process_shaders, apply_layouts, InAnnotator, IncrementingUniformAnnotator};
fn main() {
    let mut uniform_locations = HashMap::new();

    // let vert_source = include_str!("097_sodium_gbuffers_textured_lit.vsh");
    // let frag_source = include_str!("097_sodium_gbuffers_textured_lit.fsh");

    let vert_source = "";
    let frag_source = "";

    let preprocessed_vert = cyntax::preprocess_str(&vert_source, &[]);
    let preprocessed_frag = cyntax::preprocess_str(&frag_source, &[]);

    let mut vert_stage = ShaderStage::parse(preprocessed_vert).unwrap();
    let mut frag_stage = ShaderStage::parse(preprocessed_frag).unwrap();

    vert_stage.visit_mut(&mut MatrixPatcher);
    // frag_stage_ast.visit_mut(&mut ForceWhite);

    let mut sampler_types = HashMap::new();

    //Split the samplers, as well as do some other pre-processing
    sampler_types.extend(shim_samplers(&mut vert_stage, true));
    sampler_types.extend(shim_samplers(&mut frag_stage, false));

    //Apply the set and binding layouts to the uniforms
    let mut out_annotator = IncrementingAnnotator {
        offset: 0,
        target: StorageQualifier::Out,
        found: false,
        insert_location: None,
        map: HashMap::new(),
    };

    let mut uniform_annotator = IncrementingUniformAnnotator {
        uniform_found: false,
        accum: 0,
        uniform_binding: None,
        uniform_sets: uniform_locations.clone(),
        active: false,
    };

    let mut vsh_in_annotator = IncrementingAnnotator {
        offset: 0,
        target: StorageQualifier::In,
        insert_location: None,
        map: HashMap::new(),
        found: false,
    };

    vert_stage.visit_mut(&mut out_annotator);
    vert_stage.visit_mut(&mut vsh_in_annotator);
    vert_stage.visit_mut(&mut uniform_annotator);
    // dbg!(&uniform_annotator.uniform_sets);

    let mut in_annotator = InAnnotator {
        in_found: false,
        insert_location: None,
        map: out_annotator.map,
    };

    let mut rewriter = SamplerBufferRewriter {
        is_sampler_buffer: false,
        buffers: vec![],
        uniform_sets: &uniform_locations,
    };

    vert_stage.visit_mut(&mut rewriter);
    vert_stage.visit_mut(&mut RewriteFetches {
        buffers: &rewriter.buffers,
    });

    uniform_annotator.uniform_found = false;
    uniform_annotator.uniform_binding = None;

    frag_stage.visit_mut(&mut in_annotator);
    frag_stage.visit_mut(&mut uniform_annotator);

    frag_stage.visit_mut(&mut rewriter);
    frag_stage.visit_mut(&mut RewriteFetches {
        buffers: &rewriter.buffers,
    });

    vert_stage.0.0.insert(
        0,
        ExternalDeclaration::Preprocessor(Preprocessor::Version(PreprocessorVersion {
            version: 440,
            profile: None,
        })),
    );

    frag_stage.0.0.insert(
        0,
        ExternalDeclaration::Preprocessor(Preprocessor::Version(PreprocessorVersion {
            version: 440,
            profile: None,
        })),
    );

    frag_stage.visit_mut(&mut RemovePointSize {
        is_point_var: false,
    });
    vert_stage.visit_mut(&mut RemovePointSize {
        is_point_var: false,
    });

    let mut vert = String::new();
    let mut frag = String::new();

    show_translation_unit(&mut vert, &vert_stage);
    show_translation_unit(&mut frag, &frag_stage);

    let p = env!("CARGO_MANIFEST_DIR");

    std::fs::write(format!("{p}/src/out.vsh"), vert).unwrap();
    std::fs::write(format!("{p}/src/out.fsh"), frag).unwrap();

    // println!("## VERT\n\n{vert} ## END VERT ##\n\n\n## FRAG\n\n{frag}\n\n\n\n\n");
}
