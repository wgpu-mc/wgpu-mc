use crate::mc::datapack::NamespacedResource;
use crate::model::BindableTexture;
use crate::texture::{TextureSamplerView, UV};
use guillotiere::euclid::Size2D;
use guillotiere::AtlasAllocator;
use image::imageops::overlay;
use image::{GenericImageView, ImageBuffer, Rgba};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use std::sync::Arc;

use arc_swap::ArcSwap;
use parking_lot::RwLock;
use wgpu::Extent3d;

use crate::render::pipeline::RenderPipelineManager;
use crate::{WgpuState, WmRenderer};

pub const ATLAS_DIMENSIONS: i32 = 1024;

pub struct Atlas {
    pub allocator: RwLock<AtlasAllocator>,
    pub image: RwLock<image::ImageBuffer<Rgba<u8>, Vec<u8>>>,
    pub uv_map: RwLock<HashMap<NamespacedResource, UV>>,
    pub bindable_texture: ArcSwap<BindableTexture>,
}

impl Debug for Atlas {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Atlas {{ uv_map: {:?} }}", self.uv_map.read())
    }
}

impl Atlas {
    pub fn new(wgpu_state: &WgpuState, pipelines: &RenderPipelineManager) -> Self {
        let bindable_texture = BindableTexture::from_tsv(
            wgpu_state,
            pipelines,
            TextureSamplerView::from_rgb_bytes(
                wgpu_state,
                &[0u8; (ATLAS_DIMENSIONS * ATLAS_DIMENSIONS) as usize * 4],
                Extent3d {
                    width: ATLAS_DIMENSIONS as u32,
                    height: ATLAS_DIMENSIONS as u32,
                    depth_or_array_layers: 1,
                },
                None,
                wgpu::TextureFormat::Rgba8Unorm,
            )
            .unwrap(),
        );

        Self {
            allocator: RwLock::new(AtlasAllocator::new(Size2D {
                width: ATLAS_DIMENSIONS,
                height: ATLAS_DIMENSIONS,
                _unit: Default::default(),
            })),
            image: RwLock::new(ImageBuffer::new(
                ATLAS_DIMENSIONS as u32,
                ATLAS_DIMENSIONS as u32,
            )),
            uv_map: Default::default(),
            bindable_texture: ArcSwap::new(Arc::new(bindable_texture)),
        }
    }

    pub fn allocate<T: AsRef<[u8]>>(&self, images: &[(&NamespacedResource, T)]) {
        let mut allocator = self.allocator.write();
        let mut image_buffer = self.image.write();
        let mut map = self.uv_map.write();

        images.iter().for_each(|(name, slice)| {
            self.allocate_one(
                &mut *image_buffer,
                &mut *map,
                &mut *allocator,
                name,
                slice.as_ref(),
            )
        });
    }

    fn allocate_one(
        &self,
        image_buffer: &mut image::ImageBuffer<Rgba<u8>, Vec<u8>>,
        map: &mut HashMap<NamespacedResource, UV>,

        allocator: &mut AtlasAllocator,
        id: &NamespacedResource,
        image_bytes: &[u8],
    ) {
        let image = image::load_from_memory(image_bytes).unwrap();

        let allocation = allocator
            .allocate(Size2D::new(image.width() as i32, image.height() as i32))
            .unwrap();

        overlay(
            image_buffer,
            &image,
            allocation.rectangle.min.x as u32,
            allocation.rectangle.min.y as u32,
        );

        map.insert(
            id.clone(),
            (
                (
                    allocation.rectangle.min.x as f32,
                    allocation.rectangle.min.y as f32,
                ),
                (
                    allocation.rectangle.max.x as f32,
                    allocation.rectangle.max.y as f32,
                ),
            ),
        );
    }

    pub fn upload(&self, wm: &WmRenderer) {
        let tsv = TextureSamplerView::from_rgb_bytes(
            &*wm.wgpu_state,
            &self.image.read(),
            Extent3d {
                width: ATLAS_DIMENSIONS as u32,
                height: ATLAS_DIMENSIONS as u32,
                depth_or_array_layers: 1,
            },
            None,
            wgpu::TextureFormat::Rgba8Unorm,
        )
        .unwrap();
        let bindable_texture = BindableTexture::from_tsv(
            &*wm.wgpu_state,
            &*wm.render_pipeline_manager.load_full(),
            tsv,
        );
        self.bindable_texture.store(Arc::new(bindable_texture));
    }
}

///Stores uplodaded textures which will be automatically updated whenever necessary
pub struct TextureManager {
    ///Using RwLock<HashMap>> instead of DashMap because when doing a resource pack reload,
    /// we need potentially a lot of textures to be updated and it's better to be able to
    /// have some other thread work on building a new HashMap, and then just blocking any other
    /// readers for a bit to update the whole map
    pub textures: RwLock<HashMap<NamespacedResource, Arc<BindableTexture>>>,

    pub atlases: ArcSwap<HashMap<String, Arc<ArcSwap<Atlas>>>>,
}

impl TextureManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            textures: RwLock::new(HashMap::new()),
            atlases: ArcSwap::new(Arc::new(HashMap::new())),
        }
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}
