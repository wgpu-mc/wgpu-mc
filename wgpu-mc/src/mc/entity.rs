use std::sync::Arc;

pub type Position = (f64, f64);
pub type Rotation = (f32, f32);

#[allow(dead_code)] // TODO
pub struct EntityPart {
    position: Position,
}

#[allow(dead_code)] // TODO
pub struct EntityModel {
    parts: Vec<EntityPart>,
}

#[allow(dead_code)] // TODO
pub struct Entity {
    model: EntityModel,
}

#[allow(dead_code)] // TODO
pub struct EntityInstance {
    entity: Arc<Entity>,
    position: Position,
    part_rotation: Vec<Rotation>,
}
