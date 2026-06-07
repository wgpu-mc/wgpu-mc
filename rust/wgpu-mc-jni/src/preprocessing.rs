use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::io::Cursor;
use std::iter::Flatten;
use std::str::FromStr;
use glsl::parser::{Parse, ParseError};
use glsl::syntax::{ArraySpecifier, ArraySpecifierDimension, ArrayedIdentifier, Block, Declaration, Expr, ExternalDeclaration, FullySpecifiedType, FunIdentifier, FunctionParameterDeclaration, FunctionParameterDeclarator, FunctionPrototype, Identifier, InitDeclaratorList, Initializer, LayoutQualifier, LayoutQualifierSpec, NonEmpty, Preprocessor, ShaderStage, SingleDeclaration, StorageQualifier, TranslationUnit, TypeName, TypeQualifier, TypeQualifierSpec, TypeSpecifier, TypeSpecifierNonArray};
use glsl::transpiler::glsl::{show_expr, show_translation_unit};
use glsl::visitor::{Host, HostMut, Visit, Visitor, VisitorMut};
use wgpu_mc::wgpu;

struct VersionFixer;
impl VisitorMut for VersionFixer {
    fn visit_preprocessor_version(&mut self,pv: &mut glsl::syntax::PreprocessorVersion) -> Visit {
        pv.version = 440;
        Visit::Parent
    }
}
#[derive(Debug)]
struct SamplerFinder {
    layout_qualifiers: Option<[LayoutQualifierSpec; 2]>,
    names: HashMap<String, TypeSpecifierNonArray>,
    uniform: bool,
    sampler: Option<TypeSpecifierNonArray>
}

#[derive(Debug)]
struct TypeChanger {
    new_t: Option<TypeSpecifierNonArray>,
    new_binding: Option<LayoutQualifierSpec>,
    name_ext: String
}

struct FlattenSets {
    accum: u32
}

impl VisitorMut for FlattenSets {
    fn visit_layout_qualifier_spec(&mut self, spec: &mut LayoutQualifierSpec) -> Visit {
        if let LayoutQualifierSpec::Identifier(a, Some(b)) = spec {
            if let Expr::IntConst(set) = &mut **b && &a.0 == "set" {
                *set = 0;
            } else if let Expr::IntConst(binding)  = &mut **b && &a.0 == "binding" {
                *binding = self.accum as i32;
                self.accum += 1;
            }
        }

        Visit::Children
    }

}

impl VisitorMut for TypeChanger {

    fn visit_layout_qualifier(&mut self, qualifier: &mut LayoutQualifier) -> Visit {
        qualifier.ids.0 = qualifier.ids.0.clone().into_iter().filter(|spec| {
            match spec {
                LayoutQualifierSpec::Identifier(key, _) => key.0 != "binding",
                LayoutQualifierSpec::Shared => true,
            }
        }).collect();

        qualifier.ids.0.push(self.new_binding.take().unwrap());

        Visit::Parent
    }

    fn visit_single_declaration(&mut self, decl: &mut SingleDeclaration) -> Visit {
        decl.name.as_mut().unwrap().0.extend(self.name_ext.chars());

        Visit::Children
    }

    fn visit_type_specifier_non_array(&mut self, s: &mut TypeSpecifierNonArray) -> Visit {
        *s = self.new_t.take().unwrap();

        Visit::Children
    }

}

impl Visitor for SamplerFinder {
    fn visit_single_declaration(&mut self, decl: &SingleDeclaration) -> Visit {
        self.names.insert(decl.name.as_ref().unwrap().0.clone(), decl.ty.ty.ty.clone());

        Visit::Children
    }

    fn visit_layout_qualifier_spec(&mut self, spec: &LayoutQualifierSpec) -> Visit {
        match spec {
            LayoutQualifierSpec::Identifier(Identifier(v), Some(expr)) if v == "set" => match &**expr {
                Expr::IntConst(_) | Expr::UIntConst(_) => {
                    self.layout_qualifiers = Some([
                        LayoutQualifierSpec::Identifier(Identifier("binding".into()), Some(Box::new(Expr::IntConst(0)))),
                        LayoutQualifierSpec::Identifier(Identifier("binding".into()), Some(Box::new(Expr::IntConst(1))))
                    ]);

                    Visit::Children
                },
                _ => Visit::Children
            },
            _ => Visit::Children
        }

    }

    fn visit_type_qualifier_spec(&mut self, t: &TypeQualifierSpec) -> Visit {
        self.uniform |= matches!(t, TypeQualifierSpec::Storage(StorageQualifier::Uniform));

        Visit::Children
    }

    fn visit_type_specifier_non_array(&mut self, t: &TypeSpecifierNonArray) -> Visit {
        match t {
            TypeSpecifierNonArray::Sampler2D => {

            }
            _ => {}
        }

        if matches!(t,
            TypeSpecifierNonArray::Sampler2D | TypeSpecifierNonArray::SamplerCube
        ) {
            self.sampler = Some(t.clone());
        }

        Visit::Children
    }
}

struct FunctionCallExpandSampler<'a> {
    local_funcs: &'a [String],
    samplers: HashMap<String, String>
}

struct BuiltinFunctionCallMergeSampler<'a> {
    local_funcs: &'a [String],
    samplers: HashMap<String, String>
}

impl<'a> VisitorMut for BuiltinFunctionCallMergeSampler<'a> {

    //This visitor is called on calls to built-in functions

    fn visit_expr(&mut self, expr: &mut Expr) -> Visit {
        //Function calls can contain other function calls, so we have to deal with that
        if let Expr::FunCall(FunIdentifier::Identifier(func_name), params) = expr {
            //Calling a locally defined function
            if self.local_funcs.contains(&func_name.0) {
                let mut v = FunctionCallExpandSampler {
                    local_funcs: self.local_funcs,
                    samplers: self.samplers.clone(),
                };
                expr.visit_mut(&mut v);
            }
        } else if let Expr::Variable(ident) = expr {
            //This variable is referencing a sampler, and we're in a built-in function call
            match self.samplers.get(&ident.0) {
                None => {}
                Some(constructor) => {
                    *expr = Expr::FunCall(FunIdentifier::Identifier(Identifier(constructor.to_string())), vec![
                        Expr::Variable(Identifier(format!("{}_wm_texshim", ident.0))),
                        Expr::Variable(Identifier(format!("{}_wm_sampler", ident.0))),
                    ]);
                }
            }
        }

        Visit::Children
    }

}

impl<'a> VisitorMut for FunctionCallExpandSampler<'a> {
    fn visit_expr(&mut self, expr: &mut Expr) -> Visit {
        if let Expr::FunCall(FunIdentifier::Identifier(func_name), params) = expr {
            if !self.local_funcs.contains(&func_name.0) {
                let mut v = BuiltinFunctionCallMergeSampler {
                    local_funcs: &[],
                    samplers: self.samplers.clone(),
                };
                expr.visit_mut(&mut v);
            } else {
                *params = params.iter().map(|p| {
                    if let Expr::Variable(var_name) = p && self.samplers.contains_key(&var_name.0) {
                        vec![
                            Expr::Variable(Identifier(format!("{var_name}_wm_texshim"))),
                            Expr::Variable(Identifier(format!("{var_name}_wm_sampler"))),
                        ]
                    } else {
                        vec![p.clone()]
                    }
                }).flatten().collect();
            }
        }

        Visit::Children
    }

}

fn get_sampler_constructor_for_glsl_type(specifier: &TypeSpecifierNonArray) -> String {
    match specifier {
        TypeSpecifierNonArray::Sampler2D => "sampler2D".into(),
        TypeSpecifierNonArray::SamplerCube => "samplerCube".into(),
        _ => unreachable!()
    }
}

struct SamplerExpansion {
    samplers: HashMap<String, String>,
    local_functions: Vec<String>
}

impl VisitorMut for SamplerExpansion {
    fn visit_function_prototype(&mut self, proto: &mut FunctionPrototype) -> Visit {
        self.local_functions.push(proto.name.0.clone());

        proto.parameters = proto.parameters.iter().map(|param| {
            match &param {
                FunctionParameterDeclaration::Unnamed(_, _) => unimplemented!(),
                FunctionParameterDeclaration::Named(qual, decl) => {
                    if matches!(decl.ty.ty, TypeSpecifierNonArray::Sampler2D | TypeSpecifierNonArray::SamplerCube) {
                        if !self.samplers.contains_key(&decl.ident.ident.0) {
                            self.samplers.insert(decl.ident.ident.0.clone(), get_sampler_constructor_for_glsl_type(&decl.ty.ty));
                        }

                        vec![
                            FunctionParameterDeclaration::Named(
                                qual.clone(),
                                FunctionParameterDeclarator {
                                    ty: TypeSpecifier { ty: TypeSpecifierNonArray::TypeName(TypeName("texture2D".into())), array_specifier: None },
                                    ident: ArrayedIdentifier {
                                        ident: Identifier(format!("{}_wm_texshim", decl.ident.ident.0)),
                                        array_spec: None,
                                    },
                                }
                            ),
                            FunctionParameterDeclaration::Named(
                                qual.clone(),
                                FunctionParameterDeclarator {
                                    ty: TypeSpecifier { ty: TypeSpecifierNonArray::TypeName(TypeName("sampler".into())), array_specifier: None },
                                    ident: ArrayedIdentifier {
                                        ident: Identifier(format!("{}_wm_sampler", decl.ident.ident.0)),
                                        array_spec: None,
                                    },
                                }
                            )
                        ]
                    } else {
                        vec![param.clone()]
                    }
                }
            }
        }).flatten().collect();

        Visit::Parent
    }

    fn visit_expr(&mut self, expr: &mut Expr) -> Visit {
        if let Expr::FunCall(FunIdentifier::Identifier(_), _) = expr {
            let mut f = FunctionCallExpandSampler {
                local_funcs: &self.local_functions,
                samplers: self.samplers.clone(),
            };

            expr.visit_mut(&mut f);

            Visit::Parent
        } else {
            Visit::Children
        }
    }

}

struct ExplicitMipWhenSampling;

impl VisitorMut for ExplicitMipWhenSampling {
    fn visit_expr(&mut self, call: &mut Expr) -> Visit {
        if let Expr::FunCall(FunIdentifier::Identifier(id), params) = call && id.0 == "texture" {
            id.0 = "textureLod".into();

            params.push(Expr::FloatConst(0.0));
        }

        Visit::Children
    }

}

pub struct NagaFixConstArrayExplicit {
    size: Option<u32>
}

impl VisitorMut for NagaFixConstArrayExplicit {

    fn visit_init_declarator_list(&mut self, idl: &mut InitDeclaratorList) -> Visit {

        if let Some(TypeQualifier { qualifiers: NonEmpty(specs) }) = &mut idl.head.ty.qualifier {
            if specs.iter().any(|x| matches!(x, TypeQualifierSpec::Storage(StorageQualifier::Const))) {
                idl.head.initializer.visit_mut(self);
                idl.head.ty.visit_mut(self);
            }
        }

        Visit::Parent

    }

    fn visit_array_specifier_dimension(&mut self, dim: &mut ArraySpecifierDimension) -> Visit {
        match self.size.take() {
            None => {}
            Some(size) => *dim = ArraySpecifierDimension::ExplicitlySized(Box::new(Expr::IntConst(size as i32)))
        }

        Visit::Parent
    }

    fn visit_initializer(&mut self, i: &mut Initializer) -> Visit {
        match i {
            Initializer::Simple(simple) => {
                match &**simple {
                    Expr::FunCall(FunIdentifier::Expr(expr), params) => {
                        if let Expr::Bracket(_, ArraySpecifier { dimensions: NonEmpty(d)  }) = &**expr {
                            self.size = Some(params.len() as u32);
                        }
                    },
                    _ => {}
                }
            }
            _ => {}
        }

        Visit::Parent
    }

}

struct RewriteGLBuiltinSemantics;

impl VisitorMut for RewriteGLBuiltinSemantics {

    fn visit_expr(&mut self, expr: &mut Expr) -> Visit {
        if let Expr::Variable(ident) = expr {
            match &ident.0[..] {
                "gl_VertexID" => {
                    ident.0 = "int(gl_VertexIndex)".into();
                },
                "gl_InstanceID" => {
                    ident.0 = "gl_InstanceIndex".into();
                }
                _ => {}
            }
        }

        Visit::Children
    }

}

pub struct IncrementingAnnotator {
    pub offset: u32,
    pub target: StorageQualifier,
    pub found: bool,
    pub insert_location: Option<u32>,
    pub map: HashMap<String, u32>,
}

pub struct InAnnotator {
    pub in_found: bool,
    pub insert_location: Option<u32>,
    pub map: HashMap<String, u32>,
}

pub struct UniformAnnotator {
    pub uniform_found: bool,
    pub uniform_set: Option<u32>,
    pub uniform_sets: HashMap<String, u32>,
    pub active: bool
}

pub struct OrphanDestroyer {
    pub uniform_found: bool,
    pub active: bool,
    pub orphan_found: bool,
    pub uniform_set: HashMap<String, u32>
}


pub struct SamplerBufferRewriter {
    pub is_sampler_buffer: bool,
    pub set: u32,
    pub binding: u32,
    pub buffers: Vec<String>
}

pub struct RewriteFetches<'a> {
    pub buffers: &'a [String],
}

impl<'a> VisitorMut for RewriteFetches<'a> {
    fn visit_expr(&mut self, expression_base: &mut Expr) -> Visit {
        if let Expr::FunCall(FunIdentifier::Identifier(ident), e) = expression_base && ident.0 == "texelFetch" {
            if let Expr::Variable(ident) = e.first().unwrap() {
                if self.buffers.contains(&ident.0) {
                    let op = e.get(1).unwrap();

                    let mut expr_out = String::new();

                    show_expr(&mut expr_out, op);

                    *expression_base = Expr::parse(
                        format!("ivec2({}.inner[{expr_out}], 0)", ident.0)
                    ).unwrap();
                }
            }
        }

        Visit::Children
    }

}

impl VisitorMut for SamplerBufferRewriter {
    fn visit_declaration(&mut self, decl: &mut Declaration) -> Visit {
        if let Declaration::InitDeclaratorList(i) = decl {
            i.visit_mut(self);

            let name = &i.head.name.as_ref().unwrap().0;

            if self.is_sampler_buffer {
                self.buffers.push(name.clone());

                *decl = Declaration::parse(format!(
                    "layout(std430, set = {}, binding = {}) readonly buffer {name}Block {{ int[] inner; }} {name};", self.set, self.binding
                )).unwrap();
            }
        }

        Visit::Children
    }

    fn visit_layout_qualifier_spec(&mut self, spec: &mut LayoutQualifierSpec) -> Visit {
        match spec {
            LayoutQualifierSpec::Identifier(key, Some(val)) => {
                if let Expr::IntConst(set)  = &**val && &key.0 == "set" {
                    self.set = *set as u32;
                } else if let Expr::IntConst(binding)  = &**val && &key.0 == "binding" {
                    self.binding = *binding as u32;
                }
            }
            _ => {}
        }

        Visit::Children
    }

    fn visit_type_specifier_non_array(&mut self, t: &mut TypeSpecifierNonArray) -> Visit {
        self.is_sampler_buffer = matches!(t, TypeSpecifierNonArray::ISamplerBuffer);

        Visit::Parent
    }

}

impl VisitorMut for OrphanDestroyer {
    fn visit_block(&mut self, block: &mut Block) -> Visit {
        self.uniform_found = false;

        self.active = true;
        block.qualifier.to_owned().visit_mut(self);
        self.active = false;

        if self.uniform_found {
            self.orphan_found = !self.uniform_set.contains_key(&block.name.0);
        }

        Visit::Children
    }

    fn visit_translation_unit(&mut self, unit: &mut TranslationUnit) -> Visit {
        let mut keep = vec![];

        for ex in &mut unit.0.0 {
            ex.visit_mut(self);

            if !self.orphan_found {
                keep.push(ex.clone());
            }

            self.orphan_found = false;
        }

        unit.0.0 = keep;

        Visit::Parent
    }

    fn visit_single_declaration(
        &mut self,
        single_decl: &mut SingleDeclaration,
    ) -> Visit {
        self.uniform_found = false;

        self.active = true;
        single_decl.ty.to_owned().visit_mut(self);
        self.active = false;

        if self.uniform_found {
            self.orphan_found = !self.uniform_set.contains_key(&single_decl.name.as_ref().unwrap().0);
        }

        Visit::Children
    }

    fn visit_storage_qualifier(&mut self, qual: &mut StorageQualifier) -> Visit {
        if !self.active { return Visit::Children; }

        self.uniform_found = matches!(qual, StorageQualifier::Uniform);

        Visit::Children
    }
}

impl VisitorMut for UniformAnnotator {
    fn visit_block(&mut self, block: &mut Block) -> Visit {
        self.uniform_found = false;

        self.active = true;
        block.qualifier.to_owned().visit_mut(self);
        self.active = false;

        if self.uniform_found {
            self.uniform_set = Some(self.uniform_sets.get(&block.name.0).copied().expect(&block.name.0));
        }

        Visit::Children
    }

    fn visit_single_declaration(
        &mut self,
        single_decl: &mut glsl::syntax::SingleDeclaration,
    ) -> Visit {
        self.uniform_found = false;

        self.active = true;
        single_decl.ty.to_owned().visit_mut(self);
        self.active = false;

        if self.uniform_found {
            self.uniform_set = Some(self.uniform_sets.get(
                &single_decl.name.as_ref().unwrap().0
            ).copied().expect(&single_decl.name.as_ref().unwrap().0));
        }

        Visit::Children
    }

    fn visit_type_qualifier(&mut self, qual: &mut TypeQualifier) -> Visit {
        match self.uniform_set.take() {
            Some(set) => {
                qual.qualifiers
                    .0
                    .insert(0, TypeQualifierSpec::parse(format!("layout(set = {set}, binding = 0)")).unwrap());

                return Visit::Parent;
            }
            None => {}
        }

        Visit::Children
    }

    fn visit_storage_qualifier(&mut self, qual: &mut StorageQualifier) -> Visit {
        if !self.active { return Visit::Children; }

        self.uniform_found = matches!(qual, StorageQualifier::Uniform);

        Visit::Children
    }
}

impl VisitorMut for InAnnotator {
    fn visit_single_declaration(
        &mut self,
        single_decl: &mut glsl::syntax::SingleDeclaration,
    ) -> Visit {
        self.in_found = false;
        single_decl.ty.to_owned().visit_mut(self);

        if self.in_found {
            let name = &single_decl.name.as_ref().unwrap().0;
            self.insert_location = Some(self.map.get(name).copied().unwrap());
        }

        Visit::Children
    }

    fn visit_type_qualifier(&mut self, qual: &mut glsl::syntax::TypeQualifier) -> Visit {
        match self.insert_location.take() {
            Some(offset) => {
                qual.qualifiers
                    .0
                    .insert(0, TypeQualifierSpec::parse(format!("layout(location = {offset})")).unwrap());
            }
            None => {}
        }

        Visit::Children
    }

    fn visit_storage_qualifier(&mut self, qual: &mut StorageQualifier) -> Visit {
        self.in_found = matches!(qual, StorageQualifier::In);

        Visit::Children
    }
}


impl VisitorMut for IncrementingAnnotator {
    fn visit_single_declaration(
        &mut self,
        single_decl: &mut glsl::syntax::SingleDeclaration,
    ) -> Visit {
        self.found = false;
        single_decl.ty.to_owned().visit_mut(self);

        if self.found {
            let name = single_decl.name.as_ref().unwrap().0.clone();
            self.insert_location = Some(self.offset);
            self.map.insert(name, self.offset);
            self.offset += 1;
        }

        Visit::Children
    }

    fn visit_type_qualifier(&mut self, qual: &mut glsl::syntax::TypeQualifier) -> Visit {
        match self.insert_location.take() {
            Some(offset) => {
                qual.qualifiers
                    .0
                    .insert(0, TypeQualifierSpec::parse(format!("layout(location = {offset})")).unwrap());
            }
            None => {}
        }

        Visit::Children
    }

    fn visit_storage_qualifier(&mut self, qual: &mut StorageQualifier) -> Visit {
        self.found = *qual == self.target;

        Visit::Children
    }
}


//Get all the directives, except for version
#[unsafe(no_mangle)]
pub unsafe extern "C" fn extract_directives(glsl: *const c_char) -> *mut u8 {

    let glsl = unsafe { CStr::from_ptr(glsl).to_str().unwrap() };

    let shader_stage = match ShaderStage::parse(glsl) {
        Ok(s) => s,
        Err(e) => panic!("{e:?}\nglsl:\n{}", glsl)
    };

    let mut offset = 0;

    let directives: Vec<(u32, ExternalDeclaration)> = shader_stage.0.0.into_iter().enumerate().filter_map(|(index, d)|
        if matches!(d, ExternalDeclaration::Preprocessor(_)) && !matches!(d, ExternalDeclaration::Preprocessor(Preprocessor::Version(_))) {
            offset += 1;
            Some((index as u32 - offset - 1, d))
        } else {
            None
        }
    ).collect();

    Box::into_raw(Box::new(directives)) as *mut u8
}
pub fn fix_version(shader_stage: &mut ShaderStage) {
    shader_stage.visit_mut(&mut VersionFixer);
}
pub fn apply_layouts(vert_stage: &mut ShaderStage, frag_stage: &mut ShaderStage, uniform_map: HashMap<String, u32>) {
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
        uniform_sets: uniform_map,
        active: false,
    };

    let mut in_annotator = IncrementingAnnotator {
        offset: 0,
        target: StorageQualifier::In,
        found: false,
        insert_location: None,
        map: Default::default(),
    };

    vert_stage.visit_mut(&mut out_annotator);
    vert_stage.visit_mut(&mut in_annotator);
    vert_stage.visit_mut(&mut uniform_annotator);

    let mut in_annotator = InAnnotator {
        in_found: false,
        insert_location: None,
        map: out_annotator.map,
    };

    uniform_annotator.uniform_found = false;
    uniform_annotator.uniform_set = None;

    frag_stage.visit_mut(&mut in_annotator);
    frag_stage.visit_mut(&mut uniform_annotator);
}

pub fn shim_samplers(shader_stage: &mut ShaderStage, explicit_mip: bool) -> String {

    let mut swap = vec![];
    let mut sampler_uniform_names = vec![];

    for (index, ext) in shader_stage.0.0.iter().enumerate() {
        let mut finder = SamplerFinder { layout_qualifiers: None, names: HashMap::new(), uniform: false, sampler: None };
        ext.visit(&mut finder);

        if let Some([a,b]) = finder.layout_qualifiers && finder.uniform && let Some(sampler_type) = finder.sampler {
            let mut texture_uniform = ext.clone();
            let mut sampler_uniform = ext.clone();

            sampler_uniform_names.extend(finder.names);

            let (texture_type, sampler_type) = match sampler_type {
                TypeSpecifierNonArray::Sampler2D => ("texture2D".to_string(), "sampler".to_string()),
                TypeSpecifierNonArray::SamplerCube => ("textureCube".to_string(), "sampler".to_string()),
                _ => unreachable!()
            };

            texture_uniform.visit_mut(&mut TypeChanger {
                new_t: Some(TypeSpecifierNonArray::TypeName(TypeName(texture_type))),
                new_binding: Some(a),
                name_ext: "_wm_texshim".to_string(),
            });

            sampler_uniform.visit_mut(&mut TypeChanger {
                new_t: Some(TypeSpecifierNonArray::TypeName(TypeName(sampler_type))),
                new_binding: Some(b),
                name_ext: "_wm_sampler".to_string(),
            });

            swap.push((index, texture_uniform, sampler_uniform));
        }
    }

    for (target, texture, sampler) in swap.into_iter().rev() {
        shader_stage.0.0.insert(target+1, sampler);
        shader_stage.0.0.insert(target+1, texture);
        shader_stage.0.0.remove(target);
    }

    let mut rewriter = SamplerBufferRewriter {
        is_sampler_buffer: false,
        set: 0,
        binding: 0,
        buffers: vec![],
    };

    shader_stage.visit_mut(&mut rewriter);
    shader_stage.visit_mut(&mut RewriteFetches {
        buffers: &rewriter.buffers,
    });

    shader_stage.visit_mut(&mut NagaFixConstArrayExplicit { size: None });

    let mut expander = SamplerExpansion {
        samplers: sampler_uniform_names.iter().map(|(l, r)| (l.clone(), get_sampler_constructor_for_glsl_type(&r))).collect(),
        local_functions: vec![],
    };

    shader_stage.visit_mut(&mut expander);

    if explicit_mip {
        shader_stage.visit_mut(&mut ExplicitMipWhenSampling);
    }

    shader_stage.visit_mut(&mut RewriteGLBuiltinSemantics);
    shader_stage.visit_mut(&mut FlattenSets {
        accum: 0,
    });

    let mut output = String::new();

    show_translation_unit(&mut output, &shader_stage);

    output

}