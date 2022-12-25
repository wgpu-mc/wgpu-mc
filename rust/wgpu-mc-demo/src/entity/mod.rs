use std::sync::Arc;

use wgpu_mc::mc::entity::{
    Cuboid, CuboidUV, Entity, EntityInstanceTransforms, EntityInstances, EntityManager, EntityPart,
    PartTransform,
};
use wgpu_mc::mc::resource::ResourcePath;
use wgpu_mc::render::atlas::{Atlas, ATLAS_DIMENSIONS};
use wgpu_mc::WmRenderer;

pub fn describe_entity(wm: &WmRenderer) -> (Arc<Entity>, EntityInstances) {
    // px = pixel(s)
    let _1_px = 1.0 / (ATLAS_DIMENSIONS as f32);
    let _16_px = 16.0 * _1_px;
    let _64_px = 64.0 * _1_px;

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
                    north: ((0.0, _1_px * 56.0), (_1_px * 24.0, _64_px)),
                    east: ((0.0, _1_px * 56.0), (_1_px * 24.0, _64_px)),
                    south: ((0.0, _1_px * 56.0), (_1_px * 24.0, _64_px)),
                    west: ((0.0, _1_px * 56.0), (_1_px * 24.0, _64_px)),
                    up: ((_1_px * 48.0, _1_px * 32.0), (_1_px * 72.0, _1_px * 56.0)),
                    down: ((_1_px * 48.0, _1_px * 32.0), (_1_px * 72.0, _1_px * 56.0)),
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
                            north: ((_64_px, _16_px), (_64_px + _16_px, _16_px + _16_px)),
                            east: ((_64_px, _16_px), (_64_px + _16_px, _16_px + _16_px)),
                            south: ((_64_px, _16_px), (_64_px + _16_px, _16_px + _16_px)),
                            west: ((_64_px, _16_px), (_64_px + _16_px, _16_px + _16_px)),
                            up: ((_64_px, _16_px), (_64_px + _16_px, _16_px + _16_px)),
                            down: ((_64_px, _16_px), (_64_px + _16_px, _16_px + _16_px)),
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
                            north: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
                            east: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
                            south: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
                            west: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
                            up: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
                            down: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
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
                            north: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
                            east: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
                            south: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
                            west: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
                            up: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
                            down: ((_16_px, 0.0), (_16_px + _16_px, _16_px)),
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
    let test_entity_atlas = Atlas::new(
        &wm.wgpu_state,
        &wm.pipelines.load_full(),
        false,
    );

    //Allocate the image with the alex_skin_ns variable as the key
    test_entity_atlas.allocate(
        [(&alex_skin_ns, &alex_skin_resource)],
        &*wm.mc.resource_provider,
    );

    //Uploads the atlas texture to the GPU
    test_entity_atlas.upload(wm);

    let entity_manager =
        EntityManager::new(&wm.wgpu_state, &wm.pipelines.load_full());

    let test_entity = Arc::new(Entity::new(
        entity_root,
        &wm.wgpu_state,
        test_entity_atlas.bindable_texture.load_full(),
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

    (test_entity, instances)
}
