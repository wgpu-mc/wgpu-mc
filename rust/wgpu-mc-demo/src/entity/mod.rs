use std::collections::HashMap;
use std::sync::Arc;
use wgpu_mc::mc::datapack::NamespacedResource;
use wgpu_mc::mc::entity::{
    Cuboid, CuboidUV, DescribedEntityInstances, EntityInstance, EntityManager, EntityModel,
    EntityPart, PartTransform, UploadedEntityInstanceBuffer,
};
use wgpu_mc::render::atlas::{Atlas, ATLAS_DIMENSIONS};
use wgpu_mc::WmRenderer;

pub fn describe_entity(wm: &WmRenderer) -> (UploadedEntityInstanceBuffer, Arc<EntityModel>) {
    let _atlas_1px = 1.0 / (ATLAS_DIMENSIONS as f32);
    let atlas_16px = 16.0 / (ATLAS_DIMENSIONS as f32);

    let _one = 1.0 / 16.0;

    let player_root = {
        EntityPart {
            name: Arc::new("cube".into()),
            transform: PartTransform {
                pivot_x: 0.5,
                pivot_y: 0.5,
                pivot_z: 0.5,
                yaw: 0.0,
                pitch: 0.0,
                roll: 0.0,
            },
            cuboids: vec![Cuboid {
                x: 0.0,
                y: 0.0,
                z: 0.0,

                width: 1.0,
                height: 1.0,
                length: 1.0,

                textures: CuboidUV {
                    north: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                    east: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                    south: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                    west: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                    up: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                    down: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                },
            }],
            children: vec![],
        }
    };

    let alex_skin_ns: NamespacedResource = "minecraft:textures/entity/alex.png".try_into().unwrap();
    let alex_skin_resource = wm.mc.resource_provider.get_resource(&alex_skin_ns);

    //Create a new texture atlas. It's immediately present on the GPU, but it's just a blank texture
    let player_atlas = Atlas::new(&*wm.wgpu_state, &*wm.render_pipeline_manager.load_full());

    //Allocate the image with the alex_skin_ns variable as the key
    player_atlas.allocate(&[(&alex_skin_ns, &alex_skin_resource)]);

    //Uploads the atlas texture to the GPU
    player_atlas.upload(&wm);

    let entity_manager =
        EntityManager::new(&*wm.wgpu_state, &wm.render_pipeline_manager.load_full());

    {
        *entity_manager.player_texture_atlas.write() = player_atlas;
    }

    let player_model = Arc::new(EntityModel::new(player_root));

    entity_manager
        .entity_types
        .write()
        .push(player_model.clone());

    let entity_instance = EntityInstance {
        entity_model: 0,
        position: (0.0, 0.0, 0.0),
        looking_yaw: 0.0,
        uv_offset: (0.0, 0.0),
        hurt: false,
        part_transforms: vec![PartTransform {
            pivot_x: 0.0,
            pivot_y: 0.0,
            pivot_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
        }],
    };

    let described_instance = entity_instance.describe_instance(&entity_manager);

    let (entity_instance_buffer, entity_instance_bind_group) = DescribedEntityInstances {
        matrices: vec![described_instance],
    }
    .upload(&wm);

    (
        (entity_instance_buffer, entity_instance_bind_group),
        player_model,
    )
}
