use std::io::{Cursor, Read};
use std::mem::size_of;
use byteorder::{BigEndian, ReadBytesExt};
use jni::JNIEnv;
use jni::objects::{JClass, ReleaseMode};
use jni::sys::{jboolean, jbyteArray, jint, jlong, jlongArray};
use jni_fn::jni_fn;
use mc_varint::{VarInt, VarIntRead};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use slab::Slab;
use tracing_timing::HashMap;
use winit::event::VirtualKeyCode::N;

pub static LIGHT_DATA: Lazy<RwLock<Slab<LightData>>> = Lazy::new(|| {
    RwLock::new(Slab::new())
});

#[derive(Clone, Debug)]
struct BitSet {
    words: Vec<i64>,
    in_use: usize
}

impl BitSet {

    fn try_from_read_be(cursor: &mut Cursor<&[u8]>) -> Option<Self> {
        let length: i32 = cursor.read_var_int().ok()?.into();
        println!("len: {}", length);

        let mut buffer = vec![0i64; length as usize];
        cursor.read_exact(bytemuck::cast_slice_mut(&mut buffer)).ok()?;

        Some(BitSet::from(&buffer[..]))
    }

}

impl From<&[i64]> for BitSet {
    
    fn from(longs: &[i64]) -> Self {
        let mut n = longs.len();
        loop {
            if !(n > 0 && longs[n - 1] == 0) {
                break;
            }
            
            n -= 1;
        }
        
        Self {
            words: Vec::from(&longs[..n]),
            in_use: n,
        }
    }
    
}

fn try_read_packet_list_byte_array_be<const N: usize>(cursor: &mut Cursor<&[u8]>) -> Option<Vec<[u8; N]>> {
    let length: i32 = cursor.read_var_int().ok()?.into();

    let mut out = vec![];

    for _ in 0..length {
        let length: i32 = cursor.read_var_int().ok()?.into();
        assert_eq!(length as usize, N);
        let mut buffer = [0; N];
        cursor.read_exact(&mut buffer).ok()?;
        out.push(buffer);
    }

    Some(out)
}

#[derive(Debug, Clone)]
pub struct LightData {
    non_edge: bool,
    inited_sky: BitSet,
    inited_block: BitSet,
    uninited_sky: BitSet,
    uninited_block: BitSet,
    sky_nibbles: Vec<[u8; 2048]>,
    block_nibbles: Vec<[u8; 2048]>,
}

impl LightData {

    pub fn from_buffer(bytes: &[u8]) -> Option<Self> {
        let mut cursor = Cursor::new(bytes);
        
        Some(Self {
            non_edge: if cursor.read_u8().ok()? == 1 { true } else { false },

            inited_sky: BitSet::try_from_read_be(&mut cursor)?,
            inited_block: BitSet::try_from_read_be(&mut cursor)?,
            uninited_sky: BitSet::try_from_read_be(&mut cursor)?,
            uninited_block: BitSet::try_from_read_be(&mut cursor)?,

            sky_nibbles: try_read_packet_list_byte_array_be(&mut cursor)?,
            block_nibbles: try_read_packet_list_byte_array_be(&mut cursor)?
        })
    }

}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createAndDeserializeLightData(env: JNIEnv, _class: JClass, array: jbyteArray, index: jint) -> jlong {
    let byte_array = env.get_byte_array_elements(array, ReleaseMode::NoCopyBack).unwrap();
    let slice = unsafe {
        std::slice::from_raw_parts(byte_array.as_ptr().offset(index as isize) as *mut u8, byte_array.size().unwrap() as usize - (index as usize))
    };

    let light_data = LightData::from_buffer(slice).unwrap();
    println!("{:?}", light_data);

    LIGHT_DATA.write().insert(light_data) as jlong
}