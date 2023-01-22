use byteorder::{BigEndian, NetworkEndian, ReadBytesExt};
use jni::objects::{JClass, ReleaseMode};
use jni::sys::{jboolean, jbyteArray, jint, jlong, jlongArray};
use jni::JNIEnv;
use jni_fn::jni_fn;
use mc_varint::{VarInt, VarIntRead};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use slab::Slab;
use std::io::{Cursor, Read};
use std::mem::size_of;
use tracing_timing::HashMap;
use winit::event::VirtualKeyCode::N;
use wgpu_mc::mc::chunk::CHUNK_SECTIONS_PER;
use crate::{ChunkHolder, CHUNKS};

pub static LIGHT_DATA: Lazy<RwLock<Slab<LightData>>> = Lazy::new(|| RwLock::new(Slab::new()));

#[derive(Debug, Clone)]
struct BitSet {
    longs: Vec<i64>,
}

impl BitSet {

    fn from_reader<R: Read>(mut r: R) -> Option<Self> {
        let len = r.read_var_int().ok()?;
        let as_i32: i32 = len.into();
        let longs = (0..as_i32)
            .map(|_| r.read_i64::<NetworkEndian>().ok())
            .collect::<Option<Vec<_>>>()?;

        Some(Self { longs })
    }

    pub fn is_set(&self, bit_index: usize) -> bool {
        (bit_index / 64 < self.longs.len()) && (self.longs[bit_index / 64] & (1i64 << (bit_index % 64))) != 0
    }

}

fn read_nibble_arrays<const N: usize, R: Read>(mut r: R) -> Option<Vec<[u8; N]>> {
    let len = r.read_var_int().ok()?;
    let as_i32: i32 = len.into();

    (0..as_i32)
        .map(|_| {
            let mut v = [0; N];

            let len: i32 = r.read_var_int().ok()?.into();
            assert_eq!(len as usize, N);

            r.read(&mut v).ok()?;
            Some(v)
        })
        .collect::<Option<Vec<_>>>()
}

#[derive(Copy, Clone, Debug)]
pub struct DeserializedLightData {
    pub sky_light: [Option<[u8; 2048]>; 24],
    pub block_light: [Option<[u8; 2048]>; 24]
}

impl DeserializedLightData {

    pub fn new(light_data: LightData) -> Self {
        let mut sky_light = [None; 24];
        let mut block_light = [None; 24];

        let mut sky_index = 0;
        let mut block_index = 0;

        for offset in 0..CHUNK_SECTIONS_PER {
            let inited_sky = light_data.inited_sky.is_set(offset);
            let inited_block = light_data.inited_block.is_set(offset);
            // let empty_sky = light_data.uninited_sky.is_set(offset);
            // let empty_block = light_data.uninited_block.is_set(offset);

            if inited_sky {
                sky_light[offset] = Some(light_data.sky_nibbles[sky_index]);
                sky_index += 1;
            }

            if inited_block {
                block_light[offset] = Some(light_data.block_nibbles[block_index]);
                block_index += 1;
            }
        }

        Self {
            sky_light,
            block_light
        }
    }

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
            inited_sky: BitSet::from_reader(&mut cursor)?,
            inited_block: BitSet::from_reader(&mut cursor)?,
            uninited_sky: BitSet::from_reader(&mut cursor)?,
            uninited_block: BitSet::from_reader(&mut cursor)?,
            sky_nibbles: read_nibble_arrays(&mut cursor)?,
            block_nibbles: read_nibble_arrays(&mut cursor)?,
        })
    }


}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn createAndDeserializeLightData(
    env: JNIEnv,
    _class: JClass,
    array: jbyteArray,
    index: jint,
) -> jlong {
    let byte_array = env
        .get_byte_array_elements(array, ReleaseMode::NoCopyBack)
        .unwrap();
    let slice = unsafe {
        std::slice::from_raw_parts(
            byte_array.as_ptr().offset(index as isize) as *mut u8,
            byte_array.size().unwrap() as usize - (index as usize),
        )
    };

    let light_data = LightData::from_buffer(slice).unwrap();
    LIGHT_DATA.write().insert(light_data) as jlong
}

#[jni_fn("dev.birb.wgpu.rust.WgpuNative")]
pub fn bindLightData(
    env: JNIEnv,
    _class: JClass,
    data_offset: jlong,
    x: jint,
    z: jint
) {
    let mut chunks = CHUNKS.write();
    println!("{}", data_offset);

    let light = Some(DeserializedLightData::new(LIGHT_DATA.write().remove(data_offset as usize)));

    match chunks.get_mut(&[x, z]) {
        None => { chunks.insert([x, z], ChunkHolder {
            sections: [(); 24].map(|_| None),
            light_data: light,
        }); },
        Some(holder) => holder.light_data = light
    };
}