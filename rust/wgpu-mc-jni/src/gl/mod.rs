use std::vec::Vec;

use parking_lot::RwLock;

use pipeline::GLCommand;
use wgpu_mc::texture::BindableTexture;

use std::sync::Arc;

use once_cell::sync::OnceCell;
use std::collections::HashMap;

pub mod pipeline;

pub static GL_ALLOC: RwLock<HashMap<i32, GlTexture>> = RwLock::new(HashMap::new());
pub static GL_COMMANDS: RwLock<Vec<GLCommand>> = RwLock::new(Vec::new());

#[derive(Debug)]
pub struct GlTexture {
    pub width: u16,
    pub height: u16,
    pub bindable_texture: Option<Arc<BindableTexture>>,
    pub pixels: Vec<u8>,
}
