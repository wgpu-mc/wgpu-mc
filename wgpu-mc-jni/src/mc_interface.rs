use jni::objects::JObject;
use wgpu_mc::mc::chunk::Chunk;
use jni::JNIEnv;

use jni::errors::Result;

pub fn xyz_to_index(x: i32, y: i32, z: i32) -> i32 {
    y << 8 | z << 4 | x
}

pub fn chunk_from_java_world_chunk(env: &JNIEnv, object: &JObject) -> Result<Chunk> {
    
    let chunk_sections = env.get_field(
        object,
        "sections",
        "Lnet/minecraft/world/chunk/ChunkSection;")?
        .l()?;


    Chunk {
        pos: (0, 0),
        sections: Box::new([]),
        vertices: None,
        vertex_buffer: None,
        vertex_count: 0
    }
}

struct ChunkInterface<'a> {
    world_chunk: JObject<'a>,
    chunk: Chunk
}