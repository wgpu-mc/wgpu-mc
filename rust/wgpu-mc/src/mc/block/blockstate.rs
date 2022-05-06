use crate::mc::datapack::NamespacedResource;

pub struct BlockstateVariantModelDefinitionRotations {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

//Blocks are defined in-game like minecraft:cobblestone
//All `Block`s have blockstate variant definitions, (`BlockstateVariantDefinitionModel`)
//which define how to render the block in various configurations
//Those various configurations are called variants, which are baked into a BlockstateVariantMesh

pub struct BlockstateVariantDefinitionModel {
    pub id: NamespacedResource,
    pub rotations: BlockstateVariantModelDefinitionRotations,
    pub model: NamespacedResource,
}
