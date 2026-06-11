use std::collections::HashMap;
use crate::device::{BlazePipeline};
use std::ffi::{CStr, c_char, CString};
use std::fmt::{Debug, Display, Formatter, Write};
use std::iter::{Map, Zip};
use std::mem;
use std::ops::{Deref, Index, Range};
use std::vec::IntoIter;
use glsl::syntax::TypeSpecifierNonArray;
use wgpu_mc::{wgpu, WmRenderer};
use wgpu_mc::wgpu::{BlendState, BufferAddress, BufferSize};

#[repr(C)]
pub struct RawArray<T: Sized> {
    contents: *const T,
    size: u64,
}

impl<T> Clone for RawArray<T> where T: Clone {
    fn clone(&self) -> Self {
        let cloned_contents: Vec<T> = self.iter().cloned().collect();

        assert_eq!(cloned_contents.len() as u64, self.size);

        Self {
            contents: Box::into_raw(cloned_contents.into_boxed_slice()).to_raw_parts().0 as *const _,
            size: self.size,
        }
    }
}

impl<T> RawArray<T> {
    pub(crate) fn iter(&self) -> IntoIter<&T> {
        (0..self.size as usize)
            .map(|index| &self[index])
            .collect::<Vec<&T>>()
            .into_iter()
    }
}

impl<'a, T> IntoIterator for RawArray<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        (0..self.size as usize)
            .map(|index| unsafe { std::ptr::read(self.contents.offset(index as isize)) })
            .collect::<Vec<T>>()
            .into_iter()
    }
}

impl<T> Debug for RawArray<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawArray")
            .field("size", &self.size)
            .field_with("contents", |f| {
                let mut list = f.debug_list();

                for i in 0..self.size as usize {
                    list.entry(&self[i]);
                }

                list.finish()
            })
            .finish()
    }
}

impl<T> Index<usize> for RawArray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.size as usize);

        unsafe { self.contents.offset(index as isize).as_ref_unchecked() }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct BlazeAttachmentDescriptor<'a, ClearVal: Sized + Debug> {
    pub texture_view: &'a wgpu::TextureView,
    pub clear_value: Option<&'a ClearVal>,
}

#[repr(C)]
#[derive(Debug)]
pub struct BlazeRenderPassDescriptor<'a> {
    pub attachments: &'a RawArray<BlazeAttachmentDescriptor<'a, [f32; 4]>>,
    pub depth_attachment: Option<&'a BlazeAttachmentDescriptor<'a, f64>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn dummy(_: BlazeRenderPassDescriptor, _: BlazeCompareFunction) {}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct VertexFormatElement {
    pub offset: u64,
    pub format: GpuFormat,
    pub name: FfiStr,
}

#[derive(Debug)]
pub enum BlazeBindingResource {
    TextureView(wgpu::TextureView),
    Sampler(wgpu::Sampler),
    Buffer(wgpu::Buffer, Range<BufferAddress>)
}

#[derive(Debug)]
pub struct BindingBuilder {
    pub bindings: HashMap<String, BlazeBindingResource>
}

pub struct BindGroups_(pub Vec<wgpu::BindGroup>);

#[unsafe(no_mangle)]
pub extern "C" fn finalize_binding_builder(wm: &WmRenderer, binding_builder: &mut BindingBuilder, blaze_pipeline: &BlazePipeline) -> Box<BindGroups_> {
    let bindings = &binding_builder.bindings;

    let bind_groups = blaze_pipeline.blaze_descriptor
        .bind_group_layouts
        .iter()
        .zip(&blaze_pipeline.bind_group_layouts)
        .map(|(blaze_bgl, wgpu_bgl)| {
            let entries = blaze_bgl
                .entries
                .iter()
                .scan(0, |index, entry| {
                    Some(match entry.type_ {
                        UniformType::TexelBuffer | UniformType::UBO => {
                            *index += 1;

                            let (ssbo_backer, range) = if let Some(BlazeBindingResource::Buffer(buffer, slice)) = bindings.get(&*entry.name) {
                                (buffer, slice)
                            } else {
                                panic!("Couldn't find buffer {entry:?} in {bindings:?}");
                            };

                            vec![
                                wgpu::BindGroupEntry {
                                    binding: *index - 1,
                                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                        buffer: ssbo_backer,
                                        offset: range.start,
                                        size: Some(BufferSize::new(range.end - range.start).unwrap()),
                                    })
                                }
                            ]
                        }
                        UniformType::Sampler => {
                            *index += 2;

                            let mut texshim = String::with_capacity(entry.name.len() + "_wm_texshim".len());
                            texshim.write_str(&entry.name).unwrap();
                            texshim.write_str("_wm_texshim").unwrap();

                            let mut sampler = String::with_capacity(entry.name.len() + "_wm_sampler".len());
                            sampler.write_str(&entry.name).unwrap();
                            sampler.write_str("_wm_sampler").unwrap();

                            let view = if let BlazeBindingResource::TextureView (texture) = bindings.get(&texshim).unwrap() {
                                texture
                            } else {
                                panic!("Type mismatch");
                            };

                            let sampler = if let BlazeBindingResource::Sampler (sampler) = bindings.get(&sampler).unwrap() {
                                sampler
                            } else {
                                panic!("Type mismatch");
                            };


                            vec![
                                wgpu::BindGroupEntry {
                                    binding: *index - 2,
                                    resource: wgpu::BindingResource::TextureView(view)
                                },
                                wgpu::BindGroupEntry {
                                    binding: *index - 1,
                                    resource: wgpu::BindingResource::Sampler(sampler)
                                },
                            ]
                        }
                    })
                })
                .flatten()
                .collect::<Vec<_>>();

            wm.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &wgpu_bgl,
                entries: &entries,
            })
        })
        .collect();

    Box::new(BindGroups_(bind_groups))
}

#[unsafe(no_mangle)]
pub extern "C" fn create_binding_builder() -> Box<BindingBuilder> {
    Box::new(BindingBuilder {
        bindings: HashMap::new(),
    })
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct VertexFormat {
    pub elements: Box<RawArray<VertexFormatElement>>,
    pub vertex_size: u64,
}

#[repr(u64)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UniformType {
    TexelBuffer = 0,
    UBO = 1,
    Sampler = 2,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct BindGroupEntryDescriptor {
    pub type_: UniformType,
    pub name: FfiStr,
    pub texture_format: GpuFormat,
}

#[repr(transparent)]
pub struct FfiStr {
    ptr: *const c_char,
}

impl Clone for FfiStr {
    fn clone(&self) -> Self {
        Self {
            ptr: CString::new(self.to_string()).unwrap().into_raw()
        }
    }
}

impl Deref for FfiStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.ptr).to_str().unwrap() }
    }
}

impl Display for FfiStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*self)
    }
}

impl Debug for FfiStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*self)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct FragState {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct BlazeBindGroupLayout {
    pub entries: Box<RawArray<BindGroupEntryDescriptor>>,
}

#[repr(C)]
#[derive(Copy, Debug, Clone)]
pub enum BlazeBlendOp {
    Add = 1,
    Subtract = 2,
    ReverseSubtract = 3,
    Min = 4,
    Max = 5
}

impl BlazeBlendOp {

    pub fn to_wgpu(&self) -> wgpu::BlendOperation {
        match self {
            BlazeBlendOp::Add => wgpu::BlendOperation::Add,
            BlazeBlendOp::Subtract => wgpu::BlendOperation::Subtract,
            BlazeBlendOp::ReverseSubtract => wgpu::BlendOperation::ReverseSubtract,
            BlazeBlendOp::Min => wgpu::BlendOperation::Min,
            BlazeBlendOp::Max => wgpu::BlendOperation::Max
        }
    }

}

#[repr(C)]
#[derive(Copy, Debug, Clone)]
pub enum BlazeBlendFactor {
    ConstantAlpha = 1,
    ConstantColor = 2,
    DstAlpha = 3,
    DstColor = 4,
    One = 5,
    OneMinusConstantAlpha = 6,
    OneMinusConstantColor = 7,
    OneMinusDstAlpha = 8,
    OneMinusDstColor = 9,
    OneMinusSrcAlpha = 10,
    OneMinusSrcColor = 11,
    SrcAlpha = 12 ,
    SrcAlphaSaturate = 13,
    SrcColor = 14,
    Zero = 15,
}

impl BlazeBlendFactor {

    pub fn to_wgpu(&self) -> wgpu::BlendFactor {
        match self {
            BlazeBlendFactor::ConstantAlpha | BlazeBlendFactor::ConstantColor => wgpu::BlendFactor::Constant,
            BlazeBlendFactor::DstAlpha => wgpu::BlendFactor::DstAlpha,
            BlazeBlendFactor::DstColor => wgpu::BlendFactor::Dst,
            BlazeBlendFactor::One => wgpu::BlendFactor::One,
            BlazeBlendFactor::OneMinusConstantAlpha => wgpu::BlendFactor::OneMinusConstant,
            BlazeBlendFactor::OneMinusConstantColor => wgpu::BlendFactor::OneMinusConstant,
            BlazeBlendFactor::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
            BlazeBlendFactor::OneMinusDstColor => wgpu::BlendFactor::OneMinusDst,
            BlazeBlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
            BlazeBlendFactor::OneMinusSrcColor => wgpu::BlendFactor::OneMinusSrc,
            BlazeBlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
            BlazeBlendFactor::SrcAlphaSaturate => wgpu::BlendFactor::SrcAlphaSaturated,
            BlazeBlendFactor::SrcColor => wgpu::BlendFactor::Src,
            BlazeBlendFactor::Zero => wgpu::BlendFactor::Zero
        }
    }

}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct BlazeBlendState {
    pub src_color_factor: BlazeBlendFactor,
    pub dst_color_factor: BlazeBlendFactor,
    pub color_op: BlazeBlendOp,
    pub src_alpha_factor: BlazeBlendFactor,
    pub dst_alpha_factor: BlazeBlendFactor,
    pub alpha_op: BlazeBlendOp
}

impl BlazeBlendState {

    pub fn to_wgpu(&self) -> wgpu::BlendState {
        wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: self.src_color_factor.to_wgpu(),
                dst_factor: self.dst_color_factor.to_wgpu(),
                operation: self.color_op.to_wgpu(),
            },
            alpha: wgpu::BlendComponent {
                src_factor: self.src_alpha_factor.to_wgpu(),
                dst_factor: self.dst_alpha_factor.to_wgpu(),
                operation: self.alpha_op.to_wgpu(),
            },
        }
    }

}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct BlazeColorTargetState {
    pub format: GpuFormat,
    pub write_mask: u64,
    pub blend_function: Option<Box<BlazeBlendState>>
}

#[repr(u64)]
#[derive(Copy, Clone, Debug)]
pub enum BlazeCompareFunction {
    AlwaysPass = 1,
    LessThan = 2,
    LessThanOrEqual = 3,
    Equal = 4,
    NotEqual = 5,
    GreaterThanOrEqual = 6,
    GreaterThan = 7,
    NeverPass = 8
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct BlazeDepthStencilState {
    pub compare_function: BlazeCompareFunction,
    pub active: u64
}

#[repr(u64)]
#[derive(Copy, Clone, Debug)]
pub enum PrimitiveTopology {
    Lines = 1,
    DebugLineStrip = 2,
    Points = 3,
    Tris = 4,
    TriangleStrip = 5,
    TriangleFan = 6,
    Quads = 7,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct RenderPipeline {
    pub name: FfiStr,
    pub bind_group_layouts: Box<RawArray<BlazeBindGroupLayout>>,
    pub color_target_states: Box<RawArray<BlazeColorTargetState>>,
    pub depth_stencil_state: Option<Box<BlazeDepthStencilState>>,
    pub vertex_formats: Box<RawArray<VertexFormat>>,
    pub vertex_shader: FfiStr,
    pub fragment_shader: FfiStr,
    pub directives: FfiStr,
    pub frag_state: Option<Box<FragState>>,
    pub primitive_topology: PrimitiveTopology,
}

#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum GpuFormat {
    None = 0,
    R8_UNORM = 1,
    R8_SNORM = 2,
    RG8_UNORM = 3,
    RG8_SNORM = 4,
    RGB8_UNORM = 5,
    RGB8_SNORM = 6,
    RGBA8_UNORM = 7,
    RGBA8_SNORM = 8,
    R16_UNORM = 9,
    R16_SNORM = 10,
    RG16_UNORM = 11,
    RG16_SNORM = 12,
    RGB16_UNORM = 13,
    RGB16_SNORM = 14,
    RGBA16_UNORM = 15,
    RGBA16_SNORM = 16,
    R8_UINT = 17,
    R8_SINT = 18,
    RG8_UINT = 19,
    RG8_SINT = 20,
    RGB8_UINT = 21,
    RGB8_SINT = 22,
    RGBA8_UINT = 23,
    RGBA8_SINT = 24,
    R16_UINT = 25,
    R16_SINT = 26,
    RG16_UINT = 27,
    RG16_SINT = 28,
    RGB16_UINT = 29,
    RGB16_SINT = 30,
    RGBA16_UINT = 31,
    RGBA16_SINT = 32,
    R32_UINT = 33,
    R32_SINT = 34,
    RG32_UINT = 35,
    RG32_SINT = 36,
    RGB32_UINT = 37,
    RGB32_SINT = 38,
    RGBA32_UINT = 39,
    RGBA32_SINT = 40,
    R16_FLOAT = 41,
    RG16_FLOAT = 42,
    RGB16_FLOAT = 43,
    RGBA16_FLOAT = 44,
    R32_FLOAT = 45,
    RG32_FLOAT = 46,
    RGB32_FLOAT = 47,
    RGBA32_FLOAT = 48,
    RGB10A2_UNORM = 49,
    RGB10A2_UINT = 50,
    RG11B10_FLOAT = 51,
    D32_FLOAT = 52,
    D32_FLOAT_S8_UINT = 53,
    D24_UNORM_S8_UINT = 54,
    D16_UNORM = 55,
    S8_UINT = 56,
}

impl GpuFormat {

    pub fn to_wgpu_texture_format(&self) -> wgpu::TextureFormat {
        match self {
            GpuFormat::RGBA8_UNORM => wgpu::TextureFormat::Rgba8Unorm,
            GpuFormat::R8_UNORM => wgpu::TextureFormat::R8Unorm,
            GpuFormat::D32_FLOAT => wgpu::TextureFormat::Depth32Float,
            _ => unimplemented!("{self:?}")
        }
    }

    pub fn to_wgpu_vertex_format(&self) -> wgpu::VertexFormat {
        match self {
            GpuFormat::RGB32_FLOAT => wgpu::VertexFormat::Float32x3,
            GpuFormat::RG16_SINT => wgpu::VertexFormat::Sint16x2,
            GpuFormat::RG16_UINT => wgpu::VertexFormat::Uint16x2,
            GpuFormat::RG32_SINT => wgpu::VertexFormat::Sint32x2,
            GpuFormat::RGB32_SINT => wgpu::VertexFormat::Sint32x3,
            GpuFormat::RG16_SNORM => wgpu::VertexFormat::Snorm16x2,
            GpuFormat::RG32_FLOAT => wgpu::VertexFormat::Float32x2,
            GpuFormat::RGBA8_UNORM => wgpu::VertexFormat::Unorm8x4,
            GpuFormat::RGBA8_SNORM => wgpu::VertexFormat::Snorm8x4,
            GpuFormat::R32_FLOAT => wgpu::VertexFormat::Float32,
            _ => unimplemented!("{self:?}")
        }
    }

}

#[unsafe(no_mangle)]
pub extern "C" fn thing(format: GpuFormat) {}
