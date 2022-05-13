use std::borrow::Borrow;
use crate::mc::datapack::{AnimationData, NamespacedResource};
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
    pub animated_textures: ArcSwap<Vec<AnimatedTexture>>,
    pub animated_texture_offsets: ArcSwap<HashMap<NamespacedResource, u32>>,
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
            animated_textures: Default::default(),
            animated_texture_offsets: Default::default(),
        }
    }

    pub fn allocate<T: AsRef<[u8]>>(&self, images: &[(&NamespacedResource, T, Option<AnimationData>)]) {
        let mut allocator = self.allocator.write();
        let mut image_buffer = self.image.write();
        let mut map = self.uv_map.write();

        let mut animated_textures: Vec<AnimatedTexture> = Vec::new();
        let mut animated_texture_offsets: HashMap<NamespacedResource, u32> = HashMap::new();

        let mut i: u32 = 1;

        images.iter().for_each(|(name, slice, animation)| {
            let mut anim = self.allocate_one(
                &mut *image_buffer,
                &mut *map,
                &mut *allocator,
                name,
                slice.as_ref(),
                animation.as_ref(),
            );

            if anim.is_some() {
                animated_textures.push(anim.unwrap());
                animated_texture_offsets.insert((*name).clone(),i);
                i += 1;
            }
        });

        self.animated_textures.store(Arc::new(animated_textures));
        self.animated_texture_offsets.store(Arc::new(animated_texture_offsets));
    }

    fn allocate_one(
        &self,
        image_buffer: &mut image::ImageBuffer<Rgba<u8>, Vec<u8>>,
        map: &mut HashMap<NamespacedResource, UV>,

        allocator: &mut AtlasAllocator,
        id: &NamespacedResource,
        image_bytes: &[u8],
        animation: Option<&AnimationData>,
    ) -> Option<AnimatedTexture> {
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

        if animation.is_some() {
            let anim = animation.unwrap();

            let mut tex = AnimatedTexture::new(image.width(), image.height(), (image.width() as f32) / (ATLAS_DIMENSIONS as f32), AnimationData::clone(animation.unwrap()));

            map.insert(
                id.clone(),
                (
                    (
                        allocation.rectangle.min.x as f32,
                        allocation.rectangle.min.y as f32,
                    ),
                    (
                        (allocation.rectangle.min.x as u32 + tex.get_frame_size()) as f32,
                        (allocation.rectangle.min.y as u32 + tex.get_frame_size()) as f32,
                    ),
                ),
            );

            return Some(tex);
        }

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

        None
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

    pub fn update_textures(&self, subframe: u32) -> Vec<f32> {
        let mut out: Vec<f32> = Vec::new();

        out.push(0.0);
        out.push(0.0);
        out.push(0.0);
        out.push(0.0);
        out.push(0.0);
        out.push(0.0);

        for a in self.animated_textures.load_full().iter() {
            out.append(&mut a.update(subframe));
            out.push(0.0); //padding
        }

        out
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

pub struct AnimatedTexture {
    width: u32,
    height: u32,
    frame_size: u32,
    real_frame_size: f32,
    real_width: f32,
    animation: AnimationData,
    frame_count: u32,
    subframe: u32,
}

impl AnimatedTexture {
    pub fn new(width: u32, height: u32, real_width: f32, animation: AnimationData) -> Self {
        Self {
            width,
            height,
            frame_size: width,
            real_width,
            real_frame_size: real_width,
            animation,
            frame_count: height / width,
            subframe: 0,
        }
    }

    pub fn get_frame_size(&self) -> u32 {
        self.frame_size
    }

    pub fn update(&self, subframe: u32) -> Vec<f32> {

        let mut out: Vec<f32>  = Vec::new();
        let mut current_frame = (subframe / self.animation.frame_time) % self.frame_count;

        if self.animation.frames.is_some() { //if custom frame order is present translate to that
            current_frame = self.animation.frames.as_ref().unwrap()[current_frame as usize];
        }

        out.push(0.0);
        out.push(self.real_frame_size * (current_frame as f32));

        if self.animation.interpolate {
            let mut next_frame = ((subframe / self.animation.frame_time) + 1) % self.frame_count;

            if self.animation.frames.is_some() { //if custom frame order is present translate to that
                next_frame = self.animation.frames.as_ref().unwrap()[next_frame as usize];
            }

            out.push(0.0);
            out.push(self.real_frame_size * (next_frame as f32));
            out.push(((subframe % self.animation.frame_time) as f32) / (self.animation.frame_time as f32));
        } else {
            out.push(0.0);//The second frame
            out.push(0.0);
            out.push(0.0);//blend
        }

        out
    }
}
