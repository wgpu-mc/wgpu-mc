use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use arc_swap::ArcSwap;
use serde::Deserialize;

use wgpu_mc::mc::entity::{BundledEntityInstances, Entity, EntityInstance, PartTransform};
use wgpu_mc::mc::resource::ResourcePath;
use wgpu_mc::render::atlas::Atlas;
use wgpu_mc::WmRenderer;
use wgpu_mc_jni::entity::{tmd_to_wm, ModelPartData};

const ENTITY_JSON: &str = include_str!("../dumped_entities.json");

#[derive(Deserialize)]
pub struct Wrapper2 {
    data: ModelPartData,
}

#[derive(Deserialize)]
pub struct Wrapper1 {
    data: Wrapper2,
}

#[allow(unused)]
pub const ENTITY_NAME: &str = "minecraft:creeper#main";
#[allow(unused)]
const TEXTURE_LOCATION: &str = "minecraft:textures/entity/creeper/creeper.png";

#[allow(unused)]
pub fn describe_entity(wm: &WmRenderer) -> (Arc<Entity>, BundledEntityInstances) {
    let instant = Instant::now();
    let entities: HashMap<String, Wrapper1> = serde_json::from_str(ENTITY_JSON).unwrap();

    println!("{}ms", Instant::now().duration_since(instant).as_millis());

    let model_part_data = &entities.get(ENTITY_NAME).unwrap().data.data;

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

    let wm_entity = tmd_to_wm("root".into(), model_part_data, [0, 0]);

    let texture_rp = ResourcePath(TEXTURE_LOCATION.into());

    let texture_bytes = wm.mc.resource_provider.get_bytes(&texture_rp).unwrap();

    entity_atlas_guard.allocate([(&texture_rp, &texture_bytes)], &*wm.mc.resource_provider);
    entity_atlas_guard.upload(wm);

    let entity = Arc::new(Entity::new(
        ENTITY_NAME.into(),
        wm_entity.unwrap(),
        &wm.wgpu_state,
    ));

    let one_transform = EntityInstance {
        position: (0.0, 0.0, 0.0),
        looking_yaw: 0.0,
        uv_offset: [0, 0],
        part_transforms: vec![PartTransform::identity(); entity.parts.len()],
        overlays: vec![0; entity.parts.len()],
    };

    let mut instances = BundledEntityInstances::new(
        entity.clone(),
        1,
        entity_atlas_guard.bindable_texture.load_full(),
    );
    instances.upload(wm, &[one_transform]);

    (entity, instances)
}
