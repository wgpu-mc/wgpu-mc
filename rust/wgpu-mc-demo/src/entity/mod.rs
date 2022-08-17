use std::sync::Arc;
use wgpu_mc::mc::chunk::BlockStateProvider;
use wgpu_mc::mc::entity::{
    Cuboid, CuboidUV, Entity, EntityInstanceTransforms, EntityInstances, EntityManager, EntityPart,
    PartTransform,
};
use wgpu_mc::mc::resource::ResourcePath;
use wgpu_mc::render::atlas::{Atlas, ATLAS_DIMENSIONS};
use wgpu_mc::WmRenderer;

pub fn describe_entity(wm: &WmRenderer) -> (Arc<Entity>, EntityInstances) {
    let _1 = 1.0 / (ATLAS_DIMENSIONS as f32);
    let _16 = 16.0 / (ATLAS_DIMENSIONS as f32);
    let _64 = 64.0 / (ATLAS_DIMENSIONS as f32);

    let entity_root = {
        EntityPart {
            name: Arc::new("cube".into()),
            transform: PartTransform {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                pivot_x: 0.0,
                pivot_y: 0.0,
                pivot_z: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                roll: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                scale_z: 1.0,
            },
            cuboids: vec![Cuboid {
                x: 0.0,
                y: 0.0,
                z: 0.0,

                width: 32.0,
                height: 8.0,
                length: 32.0,

                textures: CuboidUV {
                    north: ((0.0, _1 * 56.0), (_1 * 24.0, _64)),
                    east: ((0.0, _1 * 56.0), (_1 * 24.0, _64)),
                    south: ((0.0, _1 * 56.0), (_1 * 24.0, _64)),
                    west: ((0.0, _1 * 56.0), (_1 * 24.0, _64)),
                    up: ((_1 * 48.0, _1 * 32.0), (_1 * 72.0, _1 * 56.0)),
                    down: ((_1 * 48.0, _1 * 32.0), (_1 * 72.0, _1 * 56.0)),
                },
            }],
            children: vec![
                EntityPart {
                    name: Arc::new("pink cube".to_string()),
                    transform: PartTransform {
                        x: 0.5,
                        y: 1.6,
                        z: 0.5,
                        pivot_x: 0.0,
                        pivot_y: 0.0,
                        pivot_z: 0.0,
                        yaw: 0.0,
                        pitch: 0.0,
                        roll: 0.0,
                        scale_x: 1.0,
                        scale_y: 1.0,
                        scale_z: 1.0,
                    },
                    cuboids: vec![Cuboid {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        width: 16.0,
                        height: 16.0,
                        length: 16.0,
                        textures: CuboidUV {
                            north: ((_64, _16), (_64 + _16, _16 + _16)),
                            east: ((_64, _16), (_64 + _16, _16 + _16)),
                            south: ((_64, _16), (_64 + _16, _16 + _16)),
                            west: ((_64, _16), (_64 + _16, _16 + _16)),
                            up: ((_64, _16), (_64 + _16, _16 + _16)),
                            down: ((_64, _16), (_64 + _16, _16 + _16)),
                        },
                    }],
                    children: vec![],
                },
                EntityPart {
                    name: Arc::new("glass thing".to_string()),
                    transform: PartTransform {
                        x: 0.4,
                        y: 1.5,
                        z: 0.4,
                        pivot_x: 0.0,
                        pivot_y: 0.0,
                        pivot_z: 0.0,
                        yaw: 0.0,
                        pitch: 0.0,
                        roll: 0.0,
                        scale_x: 1.0,
                        scale_y: 1.0,
                        scale_z: 1.0,
                    },
                    cuboids: vec![Cuboid {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        width: 19.2,
                        height: 19.2,
                        length: 19.2,
                        textures: CuboidUV {
                            north: ((_16, 0.0), (_16 + _16, _16)),
                            east: ((_16, 0.0), (_16 + _16, _16)),
                            south: ((_16, 0.0), (_16 + _16, _16)),
                            west: ((_16, 0.0), (_16 + _16, _16)),
                            up: ((_16, 0.0), (_16 + _16, _16)),
                            down: ((_16, 0.0), (_16 + _16, _16)),
                        },
                    }],
                    children: vec![],
                },
                EntityPart {
                    name: Arc::new("glass thing 2".to_string()),
                    transform: PartTransform {
                        x: 0.4,
                        y: 1.5,
                        z: 0.4,
                        pivot_x: 0.0,
                        pivot_y: 0.0,
                        pivot_z: 0.0,
                        yaw: 0.0,
                        pitch: 0.0,
                        roll: 0.0,
                        scale_x: 1.0,
                        scale_y: 1.0,
                        scale_z: 1.0,
                    },
                    cuboids: vec![Cuboid {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        width: 19.2,
                        height: 19.2,
                        length: 19.2,
                        textures: CuboidUV {
                            north: ((_16, 0.0), (_16 + _16, _16)),
                            east: ((_16, 0.0), (_16 + _16, _16)),
                            south: ((_16, 0.0), (_16 + _16, _16)),
                            west: ((_16, 0.0), (_16 + _16, _16)),
                            up: ((_16, 0.0), (_16 + _16, _16)),
                            down: ((_16, 0.0), (_16 + _16, _16)),
                        },
                    }],
                    children: vec![],
                },
            ],
        }
    };

    let alex_skin_ns: ResourcePath = "minecraft:textures/entity/end_crystal/end_crystal.png".into();
    let alex_skin_resource = wm.mc.resource_provider.get_bytes(&alex_skin_ns).unwrap();

    //Create a new texture atlas
    let test_entity_atlas = Atlas::new(&*wm.wgpu_state, &*wm.render_pipeline_manager.load_full());

    //Allocate the image with the alex_skin_ns variable as the key
    test_entity_atlas.allocate(
        [(&alex_skin_ns, &alex_skin_resource)],
        &*wm.mc.resource_provider,
    );

    //Uploads the atlas texture to the GPU
    test_entity_atlas.upload(wm);

    let entity_manager =
        EntityManager::new(&*wm.wgpu_state, &wm.render_pipeline_manager.load_full());

    let test_entity = Arc::new(Entity::new(
        entity_root,
        &wm.wgpu_state,
        test_entity_atlas.bindable_texture.clone(),
    ));

    {
        *entity_manager.player_texture_atlas.write() = test_entity_atlas;
    }

    entity_manager
        .entity_types
        .write()
        .push(test_entity.clone());

    let instances = EntityInstances::new(
        test_entity.clone(),
        vec![EntityInstanceTransforms {
            position: (0.0, 0.0, 0.0),
            looking_yaw: 0.0,
            uv_offset: (0.0, 0.0),
            part_transforms: vec![
                PartTransform::identity(),
                PartTransform::identity(),
                PartTransform::identity(),
                PartTransform::identity(),
            ],
        }],
    );

    instances.upload(wm);

    (test_entity.clone(), instances)
}
