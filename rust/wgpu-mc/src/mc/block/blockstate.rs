use crate::mc::datapack::NamespacedResource;

#[derive(Debug)]
pub struct BlockModelRotations {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

///Blocks can either be multipart or have simple variants defined.
///If it has variants, those definitions are deserialized into this struct. This isn't a [BlockModelMesh]
#[derive(Debug)]
pub struct BlockstateVariantModelDefinition {
    pub id: NamespacedResource,
    pub rotations: BlockModelRotations,
    pub model: NamespacedResource,
}
