use arc_swap::ArcSwap;
use cgmath::{Matrix4, SquareMatrix};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use wgpu_mc::mc::entity::{Entity, EntityInstanceTransforms, EntityInstances, PartTransform};
use wgpu_mc::mc::resource::ResourcePath;
use wgpu_mc::render::atlas::{Atlas, ATLAS_DIMENSIONS};
use wgpu_mc::texture::TextureSamplerView;

const ENTITY_JSON: &str = include_str!("../dumped_entities.json");

use wgpu_mc::WmRenderer;
use wgpu_mc_jni::entity::{tmd_to_wm, AtlasPosition, ModelPartData};

#[derive(Deserialize)]
pub struct Wrapper2 {
    data: ModelPartData,
}

#[derive(Deserialize)]
pub struct Wrapper1 {
    data: Wrapper2,
}

pub const ENTITY_NAME: &str = "minecraft:chest#main";
const TEXTURE_LOCATION: &str = "minecraft:textures/entity/chest/normal.png";

pub fn describe_entity(wm: &WmRenderer) -> (Arc<Entity>, EntityInstances) {
    let instant = Instant::now();
    let entities: HashMap<String, Wrapper1> = serde_json::from_str(ENTITY_JSON).unwrap();

    println!("{}ms", Instant::now().duration_since(instant).as_millis());

    let model_part_data = &entities.get(ENTITY_NAME).unwrap().data.data;

    let atlas_pos = AtlasPosition {
        width: ATLAS_DIMENSIONS,
        height: ATLAS_DIMENSIONS,
        x: 0.0,
        y: 0.0,
    };

    let entity_atlas_guard = {
        let pipelines = wm.pipelines.load();
        let atlas = Arc::new(ArcSwap::new(Arc::new(Atlas::new(
            &wm.wgpu_state,
            &pipelines,
            false,
        ))));
        let mut atlases = (**wm.mc.texture_manager.atlases.load()).clone();
        atlases.insert("entity".into(), atlas.clone());
        wm.mc.texture_manager.atlases.store(Arc::new(atlases));
        atlas.load_full()
    };

    let wm_entity = tmd_to_wm("root".into(), model_part_data, &atlas_pos);

    let texture_rp = ResourcePath(TEXTURE_LOCATION.into());

    let texture_bytes = wm.mc.resource_provider.get_bytes(&texture_rp).unwrap();

    entity_atlas_guard.allocate([(&texture_rp, &texture_bytes)], &*wm.mc.resource_provider);
    entity_atlas_guard.upload(&wm);

    let entity = Arc::new(Entity::new(
        ENTITY_NAME.into(),
        wm_entity.unwrap(),
        &wm.wgpu_state,
        entity_atlas_guard.bindable_texture.clone(),
    ));

    println!("{:?}", entity.parts);

    let one_transform = EntityInstanceTransforms {
        position: (0.0, 0.0, 0.0),
        looking_yaw: 0.0,
        uv_offset: (0.0, 0.0),
        part_transforms: vec![PartTransform::identity(); entity.parts.len()],
    };

    let instances = EntityInstances::new(entity.clone(), vec![one_transform]);
    instances.upload(wm);

    (entity, instances)
}
