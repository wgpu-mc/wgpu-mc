use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use guillotiere::euclid::Size2D;
use guillotiere::AtlasAllocator;
use image::imageops::overlay;
use image::{ImageBuffer, Rgba};
use minecraft_assets::schemas;
use parking_lot::RwLock;
use wgpu::Extent3d;

use crate::mc::resource::{ResourcePath, ResourceProvider};
use crate::texture::{TextureAndView, UV};
use crate::{Display, WmRenderer};

/// The width and height of an [atlas](Atlas];
pub const ATLAS_DIMENSIONS: u32 = 2048;

/// A texture atlas. This is used in many places, most notably terrain and entity rendering.
/// Combines multiple small textures into a single big one, which can help improve performance.
///
/// # Example
///
///```ignore
/// # use wgpu_mc::mc::resource::{ResourcePath, ResourceProvider};
/// # use wgpu_mc::render::atlas::Atlas;
/// # use wgpu_mc::{Display, WmRenderer};
/// # use wgpu_mc::render::pipeline::RenderPipelineManager;
///
/// # let wgpu_state: Display;
/// # let wm_renderer: WmRenderer;
/// # let pipelines: RenderPipelineManager;
/// # let resource_provider: Box<dyn ResourceProvider>;
///
/// let atlas = Atlas::new(&wgpu_state, &pipelines, false);
///
/// let cobble = ResourcePath("minecraft:textures/block/cobblestone.json".into());
/// let dirt = ResourcePath("minecraft:textures/block/dirt.json".into());
///
/// atlas.allocate(
///     [
///         (
///             &cobble,
///             &resource_provider.get_bytes(&cobble).unwrap()
///         ),
///         (
///             &dirt,
///             &resource_provider.get_bytes(&dirt).unwrap()
///         )
///     ], &*resource_provider
/// );
///
/// atlas.upload(&wm_renderer);
/// ```
pub struct Atlas {
    /// The image allocator which decides where images should go in the atlas texture
    pub allocator: RwLock<AtlasAllocator>,
    /// The atlas image buffer itself. This is what gets uploaded to the GPU
    pub image: RwLock<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    /// The mapping of image [ResourcePath]s to UV coordinates
    pub uv_map: RwLock<HashMap<ResourcePath, UV>>,
    /// The representation of the [Atlas]'s image buffer on the GPU, which can be bound to a draw call
    pub texture: Arc<TextureAndView>,
    /// Not every [Atlas] is used for block textures, but the ones that are store the information for each animated texture here
    pub animated_textures: RwLock<Vec<schemas::texture::TextureAnimation>>,
    pub animated_texture_offsets: RwLock<HashMap<ResourcePath, u32>>,
    size: u32,
}

impl Debug for Atlas {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Atlas {{ uv_map: {:?} }}", self.uv_map.read())
    }
}

impl Atlas {
    pub fn new(display: &Display, _resizes: bool) -> Self {
        let tv = TextureAndView::from_rgb_bytes(
            display,
            &vec![0u8; (ATLAS_DIMENSIONS * ATLAS_DIMENSIONS) as usize * 4],
            Extent3d {
                width: ATLAS_DIMENSIONS,
                height: ATLAS_DIMENSIONS,
                depth_or_array_layers: 1,
            },
            None,
            wgpu::TextureFormat::Rgba8Unorm,
        )
        .unwrap();

        Self {
            allocator: RwLock::new(AtlasAllocator::new(Size2D::new(
                ATLAS_DIMENSIONS as i32,
                ATLAS_DIMENSIONS as i32,
            ))),
            image: RwLock::new(ImageBuffer::new(ATLAS_DIMENSIONS, ATLAS_DIMENSIONS)),
            uv_map: Default::default(),
            texture: Arc::new(tv),
            animated_textures: RwLock::new(Vec::new()),
            animated_texture_offsets: Default::default(),
            size: ATLAS_DIMENSIONS,
        }
    }

    /// Add multiple textures to the atlas. This automatically handles .mcmeta files when dealing with block textures
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
        // let mut animated_texture_offsets = self.animated_texture_offsets.write();

        images.into_iter().for_each(|(name, slice)| {
            self.allocate_one(
                &mut image_buffer,
                &mut map,
                &mut allocator,
                &mut animated_textures,
                name,
                slice.as_ref(),
                resource_provider,
            );
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn allocate_one(
        &self,
        image_buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
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

        if let Some(texture) = mcmeta {
            if let Some(animation) = texture.animation {
                animated_textures.push(animation)
            }
        }

        map.insert(
            path.clone(),
            (
                (
                    allocation.rectangle.min.x as u16,
                    allocation.rectangle.min.y as u16,
                ),
                (
                    allocation.rectangle.max.x as u16,
                    allocation.rectangle.max.y as u16,
                ),
            ),
        );
    }

    /// Upload the atlas texture to the GPU. If the Atlas has to resize the texture on the GPU, then the bindable_texture that this struct provides may
    /// become obsolete if you .load() the BindableTexture before calling upload(), so you should get the BindableTexture after calling this function and not before-hand.
    /// Returns true if the atlas was resized.
    pub fn upload(&self, wm: &WmRenderer) -> bool {
        wm.display.queue.write_texture(
            self.texture.texture.as_image_copy(),
            self.image.read().as_raw(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.size),
                rows_per_image: Some(self.size),
            },
            Extent3d {
                width: self.size,
                height: self.size,
                depth_or_array_layers: 1,
            },
        );

        false
    }

    pub fn clear(&self) {
        self.allocator.write().clear();
        self.animated_texture_offsets.write().clear();
        self.animated_textures.write().clear();
        *self.image.write() = ImageBuffer::new(self.size, self.size);
    }
}

/// Stores uploaded textures which will be automatically updated whenever necessary
#[derive(Debug)]
pub struct TextureManager {
    pub default_sampler: Arc<wgpu::Sampler>,

    pub atlases: RwLock<HashMap<String, Atlas>>,
}

impl TextureManager {
    #[must_use]
    pub fn new(wgpu_state: &Display) -> Self {
        let sampler = wgpu_state.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            default_sampler: Arc::new(sampler),
            atlases: RwLock::new(HashMap::new()),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
#[allow(unused)]
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
