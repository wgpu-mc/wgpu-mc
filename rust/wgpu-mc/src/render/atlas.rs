use guillotiere::AtlasAllocator;
use image::{Rgba, GenericImageView};
use crate::model::Material;
use crate::mc::datapack::{TextureVariableOrResource, NamespacedResource};
use std::collections::HashMap;
use crate::texture::UV;
use guillotiere::euclid::Size2D;
use image::imageops::overlay;

use dashmap::DashMap;
use std::sync::Arc;

use arc_swap::ArcSwap;




pub const ATLAS_DIMENSIONS: i32 = 1024;

pub struct Atlas {
    pub allocator: AtlasAllocator,
    pub image: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub material: Option<Material>,
    pub map: HashMap<NamespacedResource, UV>
}

pub struct UploadedAtlas {
    pub image: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub material: Material,
    pub map: HashMap<TextureVariableOrResource, UV>
}

impl Atlas {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate(&mut self, id: &NamespacedResource, image_bytes: &[u8]) -> Option<()> {
        let image = image::load_from_memory(image_bytes).ok()?;

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

impl Default for Atlas {
    fn default() -> Self {
        Self {
            allocator: AtlasAllocator::new(guillotiere::Size::new(ATLAS_DIMENSIONS, ATLAS_DIMENSIONS)),
            image: image::ImageBuffer::new(ATLAS_DIMENSIONS as u32, ATLAS_DIMENSIONS as u32),
            material: None,
            map: HashMap::new()
        }
    }
}

pub struct Atlases {
    pub block: Atlas,
    pub gui: Atlas
}

pub struct TextureManager {
    pub textures: DashMap<NamespacedResource, Vec<u8>>,
    pub atlases: ArcSwap<Atlases>,
    // pub resource_provider: Arc<dyn ResourceProvider>
}

impl TextureManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            textures: DashMap::new(),
            atlases: ArcSwap::new(Arc::new(Atlases {
                block: Atlas::new(),
                gui: Atlas::new()
            }))
        }
    }

    // pub fn insert_texture(&self, id: NamespacedResource, data: Vec<u8>) {
    //     self.textures.insert(id, data);
    // }
    //
    // pub fn get_texture(&self, id: &NamespacedResource) -> Option<&[u8]> {
    //     match self.textures.get(id) {
    //         None => {
    //             let image = image::load_from_memory(
    //                 self.resource_provider.get_resource(&id.prepend("textures/"))?
    //             ).ok()?;
    //
    //             self.textures.
    //         }
    //         Some(data) => Some(&data.value()[..])
    //     }
    // }

}
