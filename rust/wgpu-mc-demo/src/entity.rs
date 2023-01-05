use std::collections::HashMap;
use std::sync::Arc;
use arc_swap::ArcSwap;
use cgmath::{Matrix4, SquareMatrix};
use serde::Deserialize;
use wgpu_mc::mc::entity::{Entity, EntityInstances, EntityInstanceTransforms, PartTransform};
use wgpu_mc::mc::resource::ResourcePath;
use wgpu_mc::render::atlas::Atlas;
use wgpu_mc::texture::TextureSamplerView;

const ENTITY_JSON: &str = include_str!("../dumped_entities.json");

use wgpu_mc::WmRenderer;
use wgpu_mc_jni::entity::{ModelPartData, tmd_to_wm};

#[derive(Deserialize)]
pub struct Wrapper2 {
    data: ModelPartData
}

#[derive(Deserialize)]
pub struct Wrapper1 {
    data: Wrapper2
}

pub fn describe_entity(wm: &WmRenderer) -> (Arc<Entity>, EntityInstances) {

    let entities: HashMap<String, Wrapper1> = serde_json::from_str(ENTITY_JSON).unwrap();

    let chest_main = &entities.get("minecraft:chest#main").unwrap().data.data;

    let wm_entity = tmd_to_wm(chest_main);

    let entity_atlas_guard = {
        let pipelines = wm.pipelines.load();
        let atlas = Arc::new(ArcSwap::new(Arc::new(Atlas::new(&wm.wgpu_state, &pipelines, false))));
        let mut atlases = (**wm.mc.texture_manager.atlases.load()).clone();
        atlases.insert("entity".into(), atlas.clone());
        wm.mc.texture_manager.atlases.store(Arc::new(atlases));
        atlas.load_full()
    };

    let chest_texture_rp = ResourcePath(
        "minecraft:textures/entity/chest/normal.png".into()
    );

    let chest_texture_bytes = wm.mc.resource_provider.get_bytes(&chest_texture_rp).unwrap();

    entity_atlas_guard.allocate([(&chest_texture_rp, &chest_texture_bytes)], &*wm.mc.resource_provider);

    let entity = Arc::new(Entity::new(
        "minecraft:chest#main".into(),
        wm_entity.unwrap(),
        &wm.wgpu_state,
        entity_atlas_guard.bindable_texture.clone()
    ));

    let one_chest_transform = EntityInstanceTransforms {
        position: (0.0, 0.0, 0.0),
        looking_yaw: 0.0,
        uv_offset: (0.0, 0.0),
        part_transforms: vec![
            PartTransform::identity(),
            PartTransform::identity(),
            PartTransform::identity(),
            PartTransform::identity(),
            PartTransform::identity()
        ],
    };

    let instances = EntityInstances::new(entity.clone(), vec![one_chest_transform]);
    instances.upload(wm);

    (entity, instances)
}
