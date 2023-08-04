use std::sync::Arc;

use wgpu_mc::mc::entity::{
    Cuboid, CuboidUV, Entity, EntityInstanceTransforms, EntityInstances, EntityManager, EntityPart,
    PartTransform,
};
use wgpu_mc::mc::resource::ResourcePath;
use wgpu_mc::render::atlas::Atlas;
use wgpu_mc::WmRenderer;

pub fn describe_entity(wm: &WmRenderer) -> (Arc<Entity>, EntityInstances) {
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
                    north: ((0, 56), (24, 64)),
                    east: ((0, 56), (24, 64)),
                    south: ((0, 56), (24, 64)),
                    west: ((0, 56), (24, 64)),
                    up: ((48, 32), (72, 56)),
                    down: ((48, 32), (72, 56)),
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
                            north: ((64, 16), (80, 32)),
                            east: ((64, 16), (80, 32)),
                            south: ((64, 16), (80, 32)),
                            west: ((64, 16), (80, 32)),
                            up: ((64, 16), (80, 32)),
                            down: ((64, 16), (80, 32)),
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
                            north: ((16, 0), (32, 16)),
                            east: ((16, 0), (32, 16)),
                            south: ((16, 0), (32, 16)),
                            west: ((16, 0), (32, 16)),
                            up: ((16, 0), (32, 16)),
                            down: ((16, 0), (32, 16)),
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
                            north: ((16, 0), (32, 16)),
                            east: ((16, 0), (32, 16)),
                            south: ((16, 0), (32, 16)),
                            west: ((16, 0), (32, 16)),
                            up: ((16, 0), (32, 16)),
                            down: ((16, 0), (32, 16)),
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
    let test_entity_atlas = Atlas::new(&wm.wgpu_state, &wm.pipelines.load_full(), false);

    //Allocate the image with the alex_skin_ns variable as the key
    test_entity_atlas.allocate(
        [(&alex_skin_ns, &alex_skin_resource)],
        &*wm.mc.resource_provider,
    );

    //Uploads the atlas texture to the GPU
    test_entity_atlas.upload(wm);

    let entity_manager = EntityManager::new(&wm.wgpu_state, &wm.pipelines.load_full());

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
