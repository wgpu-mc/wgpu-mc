use image::GenericImageView;

use crate::render::pipeline::RenderPipelineManager;
use std::num::NonZeroU32;
use wgpu::Extent3d;
use wgpu_biolerless::{State, TextureBuilder};

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

    pub fn from_image_file_bytes(wgpu_state: &State, bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(wgpu_state, &img)
    }

    #[must_use]
    pub fn create_depth_texture(state: &State) -> Self {
        let texture = state.create_depth_texture(Self::DEPTH_FORMAT);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = state.device().create_sampler(&wgpu::SamplerDescriptor {
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
        wgpu_state: &State,
        img: &image::DynamicImage,
    ) -> Result<Self, anyhow::Error> {
        let rgba8 = img.to_rgba8();

        let dimensions = img.dimensions();

        Self::from_rgb_bytes(
            wgpu_state,
            &rgba8.as_raw()[..],
            dimensions,
            wgpu::TextureFormat::Rgba8Unorm,
        )
    }

    pub fn from_rgb_bytes(
        wgpu_state: &State,
        bytes: &[u8],
        size: (u32, u32),
        format: wgpu::TextureFormat,
    ) -> Result<Self, anyhow::Error> {
        let texture = wgpu_state.create_texture(
            TextureBuilder::new()
                .format(format)
                .dimensions(size)
                .data(bytes),
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = wgpu_state
            .device()
            .create_sampler(&wgpu::SamplerDescriptor {
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

///Represents a texture that has been uploaded to GPU and has an associated `BindGroup`
#[derive(Debug)]
pub struct BindableTexture {
    pub tsv: TextureSamplerView,
    pub bind_group: wgpu::BindGroup,
}

impl BindableTexture {
    #[must_use]
    pub fn from_tsv(
        wgpu_state: &State,
        pipelines: &RenderPipelineManager,
        texture: TextureSamplerView,
    ) -> Self {
        let bind_group = wgpu_state.create_bind_group(
            pipelines.bind_group_layouts.read().get("texture").unwrap(),
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        );

        Self {
            tsv: texture,
            bind_group,
        }
    }
}

// impl TryFrom<(&WmRenderer, &NamespacedResource)> for BindableTexture {
//     type Error = anyhow::Error;
//
//     fn try_from(value: (&WmRenderer, &NamespacedResource)) -> Result<Self, Self::Error> {
//         Ok(Self::from_tsv(
//             &value.0.wgpu_state,
//             &value.0.render_pipeline_manager.load(),
//             TextureSamplerView::from_image_file_bytes(
//                 &value.0.wgpu_state, value.0.mc.resource_provider.get_bytes(value.1).ok_or(), ""
//             )?
//         ))
//     }
// }
