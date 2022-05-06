use jni::objects::{JClass, JObject, JString};
use jni::JNIEnv;
use wgpu_mc::mc::chunk::Chunk;

use jni::sys::jarray;

pub fn xyz_to_index(x: i32, y: i32, z: i32) -> i32 {
    y << 8 | z << 4 | x
}

pub fn chunk_from_java_world_chunk(env: &JNIEnv, world_chunk: &JObject) {
    let chunk_pos = env
        .get_field(*world_chunk, "pos", "Lnet/minecraft/util/math/ChunkPos;")
        .unwrap()
        .l()
        .unwrap();

    let _x = env.get_field(chunk_pos, "x", "I").unwrap().i().unwrap() * 16;
    let _z = env.get_field(chunk_pos, "z", "I").unwrap().i().unwrap() * 16;

    let sections = env
        .get_field(
            *world_chunk,
            "sections",
            "[Lnet/minecraft/world/chunk/ChunkSection;",
        )
        .unwrap()
        .l()
        .unwrap()
        .into_inner();

    let section_count = env.get_array_length(sections).unwrap();

    let sections: Vec<JObject> = (0..section_count)
        .map(|index| env.get_object_array_element(sections, index).unwrap())
        .filter(|section| !section.is_null())
        .collect();

    sections.iter().for_each(|section| {
        let _section_y_offset = env
            .get_field(*section, "yOffset", "I")
            .unwrap()
            .i()
            .unwrap();

        let container = env
            .get_field(
                *section,
                "container",
                "Lnet/minecraft/world/chunk/PalettedContainer;",
            )
            .unwrap()
            .l()
            .unwrap();

        let _data = env
            .get_field(
                *container,
                "data",
                "Lnet/minecraft/util/collection/PackedIntegerArray;",
            )
            .unwrap()
            .l()
            .unwrap();

        assert!(!container.is_null());

        let palette = env
            .get_field(
                *container,
                "palette",
                //Technically it's an ArrayPalette but that's not the signature
                "Lnet/minecraft/world/chunk/Palette;",
            )
            .unwrap()
            .l()
            .unwrap();

        assert!(!palette.is_null());

        let _palette_arr = env
            .get_field(palette, "array", "[Ljava/lang/Object;")
            .unwrap()
            .l()
            .unwrap()
            .into_inner();
    });
}

pub fn register_sprite(_env: JNIEnv, _class: JClass, _identifier: JString, _array: jarray) {}

struct ChunkInterface<'a> {
    world_chunk: JObject<'a>,
    chunk: Chunk,
}
