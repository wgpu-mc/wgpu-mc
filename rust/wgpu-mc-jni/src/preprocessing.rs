use std::ffi::{c_char, CStr, CString};
use std::io::Cursor;
use std::str::FromStr;
use glsl::parser::{Parse, ParseError};
use glsl::syntax::{ArraySpecifier, ArraySpecifierDimension, ArrayedIdentifier, Declaration, Expr, ExternalDeclaration, FullySpecifiedType, FunIdentifier, FunctionParameterDeclaration, FunctionParameterDeclarator, FunctionPrototype, Identifier, InitDeclaratorList, Initializer, LayoutQualifier, LayoutQualifierSpec, NonEmpty, Preprocessor, ShaderStage, SingleDeclaration, StorageQualifier, TranslationUnit, TypeName, TypeQualifier, TypeQualifierSpec, TypeSpecifier, TypeSpecifierNonArray};
use glsl::transpiler::glsl::show_translation_unit;
use glsl::visitor::{Host, HostMut, Visit, Visitor, VisitorMut};
use wgpu_mc::wgpu;

#[derive(Debug)]
struct SamplerFinder {
    layout_qualifiers: Option<[LayoutQualifierSpec; 2]>,
    names: Vec<String>,
    uniform: bool,
    sampler: bool
}

#[derive(Debug)]
struct TypeChanger {
    new_t: Option<TypeSpecifierNonArray>,
    new_binding: Option<LayoutQualifierSpec>,
    name_ext: String
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
        self.names.push(decl.name.as_ref().unwrap().0.clone());

        Visit::Children
    }

    fn visit_layout_qualifier_spec(&mut self, spec: &LayoutQualifierSpec) -> Visit {
        // dbg!(spec);

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
        self.sampler |= matches!(t, TypeSpecifierNonArray::Sampler2D);

        Visit::Children
    }
}

struct FunctionCallExpandSampler<'a> {
    local_funcs: &'a [String],
    samplers: &'a [String]
}

struct BuiltinFunctionCallMergeSampler2D<'a> {
    local_funcs: &'a [String],
    samplers: &'a [String]
}

impl<'a> VisitorMut for BuiltinFunctionCallMergeSampler2D<'a> {

    //This visitor is called on calls to built-in functions

    fn visit_expr(&mut self, expr: &mut Expr) -> Visit {
        //Function calls can contain other function calls, so we have to deal with that
        if let Expr::FunCall(FunIdentifier::Identifier(func_name), params) = expr {
            //Calling a locally defined function
            if self.local_funcs.contains(&func_name.0) {
                let mut v = FunctionCallExpandSampler {
                    local_funcs: self.local_funcs,
                    samplers: self.samplers,
                };
                expr.visit_mut(&mut v);
            }
        } else if let Expr::Variable(ident) = expr {
            //This variable is referencing a sampler, and we're in a built-in function call
            if self.samplers.contains(&ident.0) {
                *expr = Expr::FunCall(FunIdentifier::Identifier(Identifier("sampler2D".into())), vec![
                    Expr::Variable(Identifier(format!("{}_wm_t2d", ident.0))),
                    Expr::Variable(Identifier(format!("{}_wm_sampler", ident.0))),
                ]);
            }
        }

        Visit::Children
    }

}

impl<'a> VisitorMut for FunctionCallExpandSampler<'a> {
    fn visit_expr(&mut self, expr: &mut Expr) -> Visit {
        if let Expr::FunCall(FunIdentifier::Identifier(func_name), params) = expr {
            if !self.local_funcs.contains(&func_name.0) {
                let mut v = BuiltinFunctionCallMergeSampler2D {
                    local_funcs: &[],
                    samplers: self.samplers,
                };
                expr.visit_mut(&mut v);
            } else {
                *params = params.iter().map(|p| {
                    if let Expr::Variable(var_name) = p && self.samplers.contains(&var_name.0) {
                        vec![
                            Expr::Variable(Identifier(format!("{var_name}_wm_t2d"))),
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

struct Sampler2DExpansion {
    samplers: Vec<String>,
    local_functions: Vec<String>
}

impl VisitorMut for Sampler2DExpansion {
    fn visit_function_prototype(&mut self, proto: &mut FunctionPrototype) -> Visit {
        self.local_functions.push(proto.name.0.clone());

        proto.parameters = proto.parameters.iter().map(|param| {
            match &param {
                FunctionParameterDeclaration::Unnamed(_, _) => unimplemented!(),
                FunctionParameterDeclaration::Named(qual, decl) => {
                    if decl.ty.ty == TypeSpecifierNonArray::Sampler2D {
                        if !self.samplers.contains(&decl.ident.ident.0) { self.samplers.push(decl.ident.ident.0.clone()); }

                        vec![
                            FunctionParameterDeclaration::Named(
                                qual.clone(),
                                FunctionParameterDeclarator {
                                    ty: TypeSpecifier { ty: TypeSpecifierNonArray::TypeName(TypeName("texture2D".into())), array_specifier: None },
                                    ident: ArrayedIdentifier {
                                        ident: Identifier(format!("{}_wm_t2d", decl.ident.ident.0)),
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
                samplers: &self.samplers,
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
                    ident.0 = "gl_VertexIndex".into();
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn prepare_shader_for_naga(glsl: *const c_char, directives: *mut u8, explicit_mip: bool) -> Box<String> {
    let directives = unsafe { Box::from_raw(directives as *mut Vec<(u32, ExternalDeclaration)>) };

    Box::new(shim_samplers(
        unsafe { CStr::from_ptr(glsl).to_str().unwrap() },
        directives,
        explicit_mip
    ))
}

pub fn shim_samplers(glsl: &str, directives: Box<Vec<(u32, ExternalDeclaration)>>, explicit_mip: bool) -> String {

    let mut shader_stage = ShaderStage::parse(glsl).unwrap();

    for (i, d) in directives.into_iter().rev() {
        shader_stage.0.0.insert(i as usize, d);
    }

    let mut decl_to_remove = vec![];
    let mut decl_to_add = vec![];
    let mut sampler_uniform_names = vec![];

    for (ext_id, ext) in shader_stage.0.0.iter().enumerate() {
        let mut finder = SamplerFinder { layout_qualifiers: None, names: vec![], uniform: false, sampler: false };
        ext.visit(&mut finder);

        if let Some([a,b]) = finder.layout_qualifiers && finder.uniform && finder.sampler {
            let mut texture_uniform = ext.clone();
            let mut sampler_uniform = ext.clone();

            sampler_uniform_names.extend(finder.names);

            texture_uniform.visit_mut(&mut TypeChanger {
                new_t: Some(TypeSpecifierNonArray::TypeName(TypeName("texture2D".into()))),
                new_binding: Some(a),
                name_ext: "_wm_t2d".to_string(),
            });

            sampler_uniform.visit_mut(&mut TypeChanger {
                new_t: Some(TypeSpecifierNonArray::TypeName(TypeName("sampler".into()))),
                new_binding: Some(b),
                name_ext: "_wm_sampler".to_string(),
            });

            decl_to_add.extend([texture_uniform, sampler_uniform]);
            decl_to_remove.push(ext_id);
        }
    }

    shader_stage.visit_mut(&mut NagaFixConstArrayExplicit { size: None });

    let mut expander = Sampler2DExpansion {
        samplers: sampler_uniform_names,
        local_functions: vec![],
    };

    shader_stage.visit_mut(&mut expander);

    //in reverse because removing from an array shifts to the left, so starting from the right keeps the remaining indices intact
    for id in decl_to_remove.iter().rev() {
        shader_stage.0.0.remove(*id);
    }

    if explicit_mip {
        shader_stage.visit_mut(&mut ExplicitMipWhenSampling);
    }

    shader_stage.visit_mut(&mut RewriteGLBuiltinSemantics);

    //Insert after version decl
    shader_stage.0.0.splice(1..1, decl_to_add);

    let mut output = String::new();

    show_translation_unit(&mut output, &shader_stage);

    output

}