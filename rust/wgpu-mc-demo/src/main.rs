use std::collections::HashMap;
use glsl::parser::Parse;
use glsl::syntax::{Block, ShaderStage, StorageQualifier, TypeQualifierSpec};
use glsl::transpiler::glsl::show_translation_unit;

struct OutAnnotator {
    offset: u32,
    looking_for_out: bool,
    out_found: bool,
    insert_location: Option<u32>,
    map: HashMap<String, u32>,
}

struct InAnnotator {
    in_found: bool,
    insert_location: Option<u32>,
    map: HashMap<String, u32>,
}

use glsl::visitor::{HostMut, Visit, VisitorMut};

struct UniformAnnotator {
    uniform_found: bool,
    uniform_set: Option<u32>,
    uniform_sets: HashMap<String, u32>
}

impl VisitorMut for UniformAnnotator {
    fn visit_single_declaration(
        &mut self,
        single_decl: &mut glsl::syntax::SingleDeclaration,
    ) -> Visit {
        self.uniform_found = false;
        single_decl.ty.to_owned().visit_mut(self);

        if self.uniform_found {
            self.uniform_set = self.uniform_sets.get(&single_decl.name.as_ref().unwrap().0).copied();
        }

        Visit::Children
    }

    fn visit_block(&mut self, block: &mut Block) -> Visit {
        self.uniform_found = false;
        block.qualifier.to_owned().visit_mut(self);

        if self.uniform_found {
            self.uniform_set = self.uniform_sets.get(&block.name.0).copied();
        }

        Visit::Children
    }

    fn visit_type_qualifier(&mut self, qual: &mut glsl::syntax::TypeQualifier) -> Visit {
        match self.uniform_set.take() {
            Some(set) => {
                qual.qualifiers
                    .0
                    .insert(0, TypeQualifierSpec::parse(format!("layout(set = {set})")).unwrap());
            }
            None => {}
        }

        Visit::Children
    }

    fn visit_storage_qualifier(&mut self, qual: &mut StorageQualifier) -> Visit {
        self.uniform_found = matches!(qual, StorageQualifier::Uniform);

        Visit::Children
    }
}

impl VisitorMut for InAnnotator {
    fn visit_single_declaration(
        &mut self,
        single_decl: &mut glsl::syntax::SingleDeclaration,
    ) -> Visit {
        println!("visited");

        self.in_found = false;
        single_decl.ty.to_owned().visit_mut(self);

        if self.in_found {
            let name = &single_decl.name.as_ref().unwrap().0;
            self.insert_location = self.map.get(name).copied();
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


impl VisitorMut for OutAnnotator {
    fn visit_single_declaration(
        &mut self,
        single_decl: &mut glsl::syntax::SingleDeclaration,
    ) -> Visit {
        println!("visited");

        self.out_found = false;
        single_decl.ty.to_owned().visit_mut(self);

        if self.out_found {
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
        self.out_found = matches!(qual, StorageQualifier::Out);

        Visit::Children
    }
}

fn main() {
    let mut vert_stage = ShaderStage::parse(r#"
out vec3 test;
out float test1;
out vec2 test2;
out vec4 test3;

uniform Projection
{
    mat4 ermmLol;
};

uniform vec4 Lmao;

in vec3 pos;
in vec2 uv;

void main() {}

"#).unwrap();

    let mut frag_stage = ShaderStage::parse(r#"
in vec3 test;

uniform Projection
{
    mat4 ermmLol;
};

uniform vec4 Lmao;

void main() {}

"#).unwrap();

    let mut out_annotator = OutAnnotator {
        offset: 0,
        looking_for_out: false,
        out_found: false,
        insert_location: None,
        map: HashMap::new(),
    };

    let mut uniform_sets = HashMap::new();
    uniform_sets.insert("Projection".into(), 2);
    uniform_sets.insert("Lmao".into(), 3);

    let mut uniform_annotator = UniformAnnotator {
        uniform_found: false,
        uniform_set: None,
        uniform_sets,
    };

    let mut vert_layout = HashMap::new();
    vert_layout.insert("pos".into(), 0);
    vert_layout.insert("uv".into(), 1);

    let mut in_annotator = InAnnotator {
        in_found: false,
        insert_location: None,
        map: vert_layout,
    };

    vert_stage.visit_mut(&mut out_annotator);
    vert_stage.visit_mut(&mut in_annotator);
    vert_stage.visit_mut(&mut uniform_annotator);

    dbg!(&out_annotator.map);

    let mut in_annotator = InAnnotator {
        in_found: false,
        insert_location: None,
        map: out_annotator.map,
    };

    frag_stage.visit_mut(&mut in_annotator);
    frag_stage.visit_mut(&mut uniform_annotator);

    let mut out_vert = String::new();
    let mut out_frag = String::new();

    show_translation_unit(&mut out_vert, &vert_stage);
    show_translation_unit(&mut out_frag, &frag_stage);
    // dbg!(&vert_stage);
    println!("## Vert ##\n{out_vert}\n\n## Frag ##\n{out_frag}");
}