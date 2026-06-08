use codespan_reporting::files::SimpleFiles;
use glsl::parser::Parse;
use glsl::syntax::{
    BinaryOp, Block, Declaration, Expr, ExternalDeclaration, FunIdentifier, LayoutQualifierSpec,
    Preprocessor, PreprocessorPragma, ShaderStage, SingleDeclaration, StorageQualifier,
    TranslationUnit, TypeQualifier, TypeQualifierSpec, TypeSpecifierNonArray,
};
use glsl::transpiler::glsl::{show_expr, show_translation_unit};
use glsl::visitor::{HostMut, Visit, VisitorMut};
use std::collections::HashMap;
use std::fmt::Write;
use std::str::FromStr;
use wgpu_mc_jni::preprocessing;
use wgpu_mc_jni::preprocessing::{IncrementingAnnotator, OrphanDestroyer, RemovePointSize, RewriteFetches, SamplerBufferRewriter, UniformAnnotator, shim_samplers, MatrixPatcher};
fn main() {
    let mut vert_stage = ShaderStage::parse(r#"uniform vec2 Fog;
uniform sampler2D Sampler;
uniform vec2 Projection;

void main() {
}"#,
    )
    .unwrap();

    vert_stage.visit_mut(&mut MatrixPatcher);

    let mut out = String::new();

    show_translation_unit(&mut out, &vert_stage);

    println!("## Vert ##\n{out}\n");
}
