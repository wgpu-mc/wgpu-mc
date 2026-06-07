use std::ffi::{c_char, CStr};
use std::fmt::{Debug, Display, Formatter};
use std::iter::{Map, Zip};
use std::ops::{Deref, Index, Range};
use std::vec::IntoIter;
use crate::device::TextureView_;

#[repr(C)]
pub struct RawArray<T: Sized> {
    contents: *const T,
    size: u64
}

impl<T> RawArray<T> {

    pub(crate) fn iter(&self) -> IntoIter<&T> {
        dbg!(self.size);
        (0..self.size as usize).map(|index| &self[index]).collect::<Vec<&T>>().into_iter()
    }

}

impl<'a, T> IntoIterator for RawArray<T> {

    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        (0..self.size as usize).map(|index| {
            unsafe { std::ptr::read(self.contents.offset(index as isize)) }
        }).collect::<Vec<T>>().into_iter()
    }

}

impl<T> Debug for RawArray<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawArray")
            .field("size", &self.size)
            .field_with("contents", |f| {
                let mut list = f.debug_list();

                for i in 0..self.size as usize {
                    list.entry(&self[i]);
                }

                list.finish()
            }).finish()
    }
}

impl<T> Index<usize> for RawArray<T> {

    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.size as usize);

        unsafe {
            self.contents.offset(index as isize).as_ref_unchecked()
        }
    }

}

#[repr(C)]
#[derive(Debug)]
pub struct BlazeAttachmentDescriptor<'a, ClearVal: Sized + Debug> {
    pub texture_view: &'a TextureView_,
    pub clear_value: Option<&'a ClearVal>
}

#[repr(C)]
#[derive(Debug)]
pub struct BlazeRenderPassDescriptor<'a> {
    pub attachments: &'a RawArray<BlazeAttachmentDescriptor<'a, [f32; 4]>>,
    pub depth_attachment: Option<&'a BlazeAttachmentDescriptor<'a, f64>>
}

#[unsafe(no_mangle)]
pub extern "C" fn dummy(_: BlazeRenderPassDescriptor) {}

#[repr(C)]
#[derive(Debug)]
pub struct VertexFormatElement {
    pub offset: u64,
    pub format: GpuFormat,
}

#[repr(C)]
#[derive(Debug)]
pub struct VertexFormat<'a> {
    pub elements: &'a RawArray<VertexFormatElement>,
    pub vertex_size: u64
}

#[repr(u64)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UniformType {
    TexelBuffer = 0,
    UBO = 1,
    Sampler = 2,
}

#[repr(C)]
#[derive(Debug)]
pub struct BindGroupEntryDescriptor {
    pub type_: UniformType,
    pub name: FfiStr,
    pub texture_format: GpuFormat
}

#[repr(transparent)]
pub struct FfiStr {
    ptr: *const c_char
}

impl Deref for FfiStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe {
            CStr::from_ptr(self.ptr).to_str().unwrap()
        }
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
#[derive(Debug)]
pub struct FragState {}

#[repr(C)]
#[derive(Debug)]
pub struct BlazeBindGroupLayout<'a> {
    pub entries: &'a RawArray<BindGroupEntryDescriptor>
}

#[repr(C)]
#[derive(Debug)]
pub struct BlazeColorTargetState {
    blend_function: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BlazeDepthStencilState {
    compare_function: u64
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
#[derive(Debug)]
pub struct RenderPipeline<'a> {
    pub bind_group_layouts: &'a RawArray<BlazeBindGroupLayout<'a>>,
    pub color_target_states: &'a RawArray<BlazeColorTargetState>,
    pub depth_stencil_state: Option<&'a BlazeDepthStencilState>,
    pub vertex_formats: &'a RawArray<VertexFormat<'a>>,
    pub vertex_shader: FfiStr,
    pub fragment_shader: FfiStr,
    pub defines: &'a RawArray<[FfiStr; 2]>,
    pub frag_state: Option<&'a FragState>,
    pub primitive_topology: PrimitiveTopology
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
    S8_UINT = 56
}

#[unsafe(no_mangle)]
pub extern "C" fn thing(format: GpuFormat) {}