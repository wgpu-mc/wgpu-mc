use crate::mc::resource::{ResourcePath, ResourceProvider};
use crate::texture::{BindableTexture, TextureSamplerView, UV};
use bytemuck::{Pod, Zeroable};
use guillotiere::euclid::Size2D;
use guillotiere::AtlasAllocator;
use image::imageops::overlay;
use image::{GenericImageView, ImageBuffer, Rgba};
use minecraft_assets::schemas;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::num::NonZeroU32;

use std::sync::Arc;

use arc_swap::ArcSwap;
use parking_lot::RwLock;
use wgpu::Extent3d;
use wgpu_biolerless::State;

use crate::render::pipeline::RenderPipelineManager;
use crate::WmRenderer;

pub const ATLAS_DIMENSIONS: u32 = 2048;

///A texture atlas. This is used in many places, most notably terrain and entity rendering.
///
/// # Example
///
/// ```
/// let wgpu_state: State;
/// let pipelines: RenderPipelineManager;
/// let resource_provider: Box<dyn ResourceProvider>;
///
/// let atlas = Atlas::new(&wgpu_state, &pipelines);
///
/// atlas.allocate(
///     [
///         &(
///             &"minecraft:textures/block/cobblestone.json".into(),
///             resource_provider.get_bytes("minecraft:textures/block/cobblestone.json").unwrap()
///         ),
///         &(
///             &"minecraft:textures/block/dirt.json".into(),
///             resource_provider.get_bytes("minecraft:textures/block/dirt.json").unwrap()
///         )
///     ]
/// );
///
/// atlas.upload();
/// ```
pub struct Atlas {
    ///The image allocator which decides where images should go in the atlas texture
    pub allocator: RwLock<AtlasAllocator>,
    ///The atlas image buffer itself. This is what gets uploaded to the GPU
    pub image: RwLock<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    ///The mapping of image [ResourcePath]s to UV coordinates
    pub uv_map: RwLock<HashMap<ResourcePath, UV>>,
    ///The representation of the [Atlas]'s image buffer on the GPU, which can be bound to a draw call
    pub bindable_texture: Arc<BindableTexture>,
    ///Not every [Atlas] is used for block textures, but the ones that are store the information for each animated texture here
    pub animated_textures: RwLock<Vec<schemas::texture::TextureAnimation>>,
    ///
    pub animated_texture_offsets: RwLock<HashMap<ResourcePath, u32>>,
}

impl Debug for Atlas {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Atlas {{ uv_map: {:?} }}", self.uv_map.read())
    }
}

impl Atlas {
    pub fn new(wgpu_state: &State, pipelines: &RenderPipelineManager) -> Self {
        let bindable_texture = BindableTexture::from_tsv(
            wgpu_state,
            pipelines,
            TextureSamplerView::from_rgb_bytes(
                wgpu_state,
                &[0u8; (ATLAS_DIMENSIONS * ATLAS_DIMENSIONS) as usize * 4],
                (ATLAS_DIMENSIONS, ATLAS_DIMENSIONS),
                wgpu::TextureFormat::Rgba8Unorm,
            )
            .unwrap(),
        );

        Self {
            allocator: RwLock::new(AtlasAllocator::new(Size2D {
                width: ATLAS_DIMENSIONS as i32,
                height: ATLAS_DIMENSIONS as i32,
                _unit: Default::default(),
            })),
            image: RwLock::new(ImageBuffer::new(ATLAS_DIMENSIONS, ATLAS_DIMENSIONS)),
            uv_map: Default::default(),
            bindable_texture: Arc::new(bindable_texture),
            animated_textures: RwLock::new(Vec::new()),
            animated_texture_offsets: Default::default(),
        }
    }

    ///Add multiple textures to the atlas. This automatically handles .mcmeta files when dealing with block textures
    pub fn allocate<'a, T>(
        &self,
        images: impl IntoIterator<Item = (&'a ResourcePath, &'a T)>,
        resource_provider: &dyn ResourceProvider,
    ) where
        T: AsRef<[u8]> + 'a,
    {
        let mut allocator = self.allocator.write();
        let mut image_buffer = self.image.write();
        let mut map = self.uv_map.write();

        let mut animated_textures = self.animated_textures.write();
        let mut animated_texture_offsets = self.animated_texture_offsets.write();

        images.into_iter().for_each(|(name, slice)| {
            self.allocate_one(
                &mut *image_buffer,
                &mut *map,
                &mut *allocator,
                &mut animated_textures,
                name,
                slice.as_ref(),
                resource_provider,
            );
        });
    }

    fn allocate_one(
        &self,
        image_buffer: &mut image::ImageBuffer<Rgba<u8>, Vec<u8>>,
        map: &mut HashMap<ResourcePath, UV>,

        allocator: &mut AtlasAllocator,

        animated_textures: &mut Vec<schemas::texture::TextureAnimation>,

        path: &ResourcePath,
        image_bytes: &[u8],
        resource_provider: &dyn ResourceProvider,
    ) {
        let image = image::load_from_memory(image_bytes).unwrap();

        let allocation = allocator
            .allocate(Size2D::new(image.width() as i32, image.height() as i32))
            .unwrap();

        overlay(
            image_buffer,
            &image,
            allocation.rectangle.min.x as i64,
            allocation.rectangle.min.y as i64,
        );

        let mcmeta_path = path.append(".mcmeta");

        let mcmeta = resource_provider
            .get_string(&mcmeta_path)
            .and_then(|string| serde_json::from_str::<schemas::texture::Texture>(&string).ok());

        match mcmeta {
            Some(texture) => match texture.animation {
                Some(animation) => animated_textures.push(animation),
                _ => {}
            },
            _ => {}
        }

        map.insert(
            path.clone(),
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

    ///Upload the atlas texture to the GPU
    pub fn upload(&self, wm: &WmRenderer) {
        wm.wgpu_state.queue().write_texture(
            self.bindable_texture.tsv.texture.as_image_copy(),
            self.image.read().as_raw(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * ATLAS_DIMENSIONS as u32),
                rows_per_image: NonZeroU32::new(ATLAS_DIMENSIONS as u32),
            },
            Extent3d {
                width: ATLAS_DIMENSIONS as u32,
                height: ATLAS_DIMENSIONS as u32,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn clear(&self) {
        self.allocator.write().clear();
        self.animated_texture_offsets.write().clear();
        self.animated_textures.write().clear();
        *self.image.write() = ImageBuffer::new(ATLAS_DIMENSIONS, ATLAS_DIMENSIONS);
    }
}

///Stores uplodaded textures which will be automatically updated whenever necessary
pub struct TextureManager {
    ///Using RwLock<HashMap>> instead of DashMap because when doing a resource pack reload,
    /// we need potentially a lot of textures to be updated and it's better to be able to
    /// have some other thread work on building a new HashMap, and then just blocking any other
    /// readers for a bit to update the whole map
    pub textures: RwLock<HashMap<ResourcePath, Arc<BindableTexture>>>,

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

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
struct AnimatedUV {
    pub uv_1: [f32; 2],
    pub uv_2: [f32; 2],
    pub blend: f32,
    pub padding: f32,
}

// impl AnimatedTexture {
//     pub fn new(width: u32, height: u32, real_width: f32, animation: AnimationData) -> Self {
//         Self {
//             width,
//             height,
//             frame_size: width,
//             real_width,
//             real_frame_size: real_width,
//             animation,
//             frame_count: height / width,
//             subframe: 0,
//         }
//     }

//     pub fn get_frame_size(&self) -> u32 {
//         self.frame_size
//     }

//     pub fn update(&self, subframe: u32) -> [f32; 5] {

//         //Due to padding in the buffer, some of these elements are always left as 0.0
//         let mut out = [0.0; 5];
//         let mut current_frame = (subframe / self.animation.frame_time) % self.frame_count;

//         if self.animation.frames.is_some() { //if custom frame order is present translate to that
//             current_frame = self.animation.frames.as_ref().unwrap()[current_frame as usize];
//         }

//         out[1] = self.real_frame_size * (current_frame as f32);

//         if self.animation.interpolate {
//             let mut next_frame = ((subframe / self.animation.frame_time) + 1) % self.frame_count;

//             if self.animation.frames.is_some() { //if custom frame order is present translate to that
//                 next_frame = self.animation.frames.as_ref().unwrap()[next_frame as usize];
//             }

//             out[3] = self.real_frame_size * (next_frame as f32);
//             out[4] = ((subframe % self.animation.frame_time) as f32) / (self.animation.frame_time as f32);
//         }

//         out
//     }
// }
