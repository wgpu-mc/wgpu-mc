use crate::mc::datapack::NamespacedResource;

pub struct BlockstateVariantModelDefinitionRotations {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

//Blocks are defined in-game like minecraft:cobblestone
//Blocks can either be multipart or have simple variants defined.
//If it has variants, those definitions are serialized into this struct

pub struct BlockstateVariantModelDefinition {
    pub id: NamespacedResource,
    pub rotations: BlockstateVariantModelDefinitionRotations,
    pub model: NamespacedResource,
}
