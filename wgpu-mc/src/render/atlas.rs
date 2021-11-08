use guillotiere::AtlasAllocator;
use image::{Rgba, GenericImageView};
use crate::model::Material;
use crate::mc::datapack::Identifier;
use std::collections::HashMap;
use crate::texture::UV;
use guillotiere::euclid::Size2D;
use image::imageops::overlay;
use cgmath::Vector2;
use dashmap::DashMap;
use std::sync::Arc;
use parking_lot::RwLock;

pub const ATLAS_DIMENSIONS: i32 = 1024;

pub struct Atlas {
    pub allocator: AtlasAllocator,
    pub image: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub material: Option<Material>,
    pub map: HashMap<Identifier, UV>
}

impl Atlas {
    pub fn new() -> Self {
        Self {
            allocator: AtlasAllocator::new(guillotiere::Size::new(ATLAS_DIMENSIONS, ATLAS_DIMENSIONS)),
            image: image::ImageBuffer::new(ATLAS_DIMENSIONS as u32, ATLAS_DIMENSIONS as u32),
            material: None,
            map: HashMap::new()
        }
    }

    pub fn allocate(&mut self, id: &Identifier, image_bytes: &[u8]) -> Option<()> {
        let image = image::load_from_memory(&image_bytes[..]).ok()?;

        let allocation = self.allocator
            .allocate(Size2D::new(image.width() as i32, image.height() as i32))?;

        overlay(
            &mut self.image,
            &image,
            allocation.rectangle.min.x as u32,
            allocation.rectangle.min.y as u32,
        );

        self.map.insert(
            id.clone(),
            (
                (
                    allocation.rectangle.min.x as f32,
                    allocation.rectangle.min.y as f32,
                ),
                (
                    allocation.rectangle.max.x as f32,
                    allocation.rectangle.max.y as f32,
                )
            ),
        );

        Some(())
    }
}

pub struct Atlases {
    pub block: Atlas,
    pub gui: Atlas
}

pub struct TextureManager {
    pub textures: DashMap<Identifier, Vec<u8>>,
    pub atlases: RwLock<Atlases>
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: DashMap::new(),
            atlases: RwLock::new(Atlases {
                block: Atlas::new(),
                gui: Atlas::new()
            })
        }
    }

    pub fn insert_texture(&self, id: Identifier, data: Vec<u8>) {
        self.textures.insert(id, data);
    }
}