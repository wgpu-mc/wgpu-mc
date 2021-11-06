use guillotiere::AtlasAllocator;
use image::{Rgba, GenericImageView};
use crate::model::Material;
use crate::mc::datapack::Identifier;
use std::collections::HashMap;
use crate::texture::UV;
use guillotiere::euclid::Size2D;
use image::imageops::overlay;
use cgmath::Vector2;

pub const ATLAS_DIMENSIONS: i32 = 1024;

pub struct Atlas {
    pub allocator: AtlasAllocator,
    pub image: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub material: Option<Material>,
    pub map: Option<HashMap<Identifier, UV>>
}

impl Atlas {
    pub fn new() -> Self {
        Self {
            allocator: AtlasAllocator::new(guillotiere::Size::new(ATLAS_DIMENSIONS, ATLAS_DIMENSIONS)),
            image: image::ImageBuffer::new(ATLAS_DIMENSIONS as u32, ATLAS_DIMENSIONS as u32),
            material: None,
            map: None
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

        self.map.as_mut()?.insert(
            id.clone(),
            (
                Vector2::new(
                    allocation.rectangle.min.x as f32,
                    allocation.rectangle.min.y as f32,
                ),
                Vector2::new(
                    allocation.rectangle.max.x as f32,
                    allocation.rectangle.max.y as f32,
                ),
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
    pub textures: HashMap<Identifier, Vec<u8>>,
    pub atlases: Atlases
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            atlases: Atlases {
                block: Atlas::new(),
                gui: Atlas::new()
            }
        }
    }
}