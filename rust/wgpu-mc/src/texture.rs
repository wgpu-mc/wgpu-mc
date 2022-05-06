use image::GenericImageView;

use crate::WgpuState;
use std::num::NonZeroU32;
use wgpu::Extent3d;

pub type TextureId = u32;
pub type UV = ((f32, f32), (f32, f32));

///Representation of a texture that has been uploaded to wgpu along with the corresponding view
/// and sampler
#[derive(Debug)]
pub struct TextureSamplerView {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl TextureSamplerView {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn from_image_file_bytes(
        wgpu_state: &WgpuState,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, anyhow::Error> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(wgpu_state, &img, Some(label))
    }

    #[must_use]
    pub fn create_depth_texture(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: surface_config.width,
            height: surface_config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // 4.
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn from_image(
        wgpu_state: &WgpuState,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self, anyhow::Error> {
        let rgba8 = img.to_rgba8();

        let dimensions = img.dimensions();

        Self::from_rgb_bytes(
            wgpu_state,
            &rgba8.as_raw()[..],
            Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
            label,
            wgpu::TextureFormat::Rgba8Unorm,
        )
    }

    pub fn from_rgb_bytes(
        wgpu_state: &WgpuState,
        bytes: &[u8],
        size: wgpu::Extent3d,
        label: Option<&str>,
        format: wgpu::TextureFormat,
    ) -> Result<Self, anyhow::Error> {
        let texture = wgpu_state.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        wgpu_state.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(size.width * 4),
                rows_per_image: NonZeroU32::new(size.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = wgpu_state.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
