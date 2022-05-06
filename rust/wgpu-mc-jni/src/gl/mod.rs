use std::vec::Vec;

use parking_lot::RwLock;

use pipeline::GLCommand;
use wgpu_mc::model::BindableTexture;

use std::sync::Arc;

use once_cell::sync::OnceCell;
use std::collections::HashMap;

pub mod pipeline;

pub static GL_ALLOC: OnceCell<RwLock<HashMap<i32, GlTexture>>> = OnceCell::new();
pub static GL_COMMANDS: OnceCell<RwLock<Vec<GLCommand>>> = OnceCell::new();

pub fn init() {
    GL_ALLOC.set(RwLock::new(HashMap::new()));
    GL_COMMANDS.set(RwLock::new(Vec::new()));
}

#[derive(Debug)]
pub struct GlTexture {
    pub width: u16,
    pub height: u16,
    pub bindable_texture: Option<Arc<BindableTexture>>,
    pub pixels: Vec<u8>,
}
