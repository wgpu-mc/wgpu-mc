pub type Position = (f64, f64);
pub type Rotation = (f32, f32);

pub struct EntityPart {
    position: Position
}

pub struct EntityModel {
    parts: Vec<EntityPart>
}

pub struct Entity {
    model: EntityModel,
}

pub struct EntityInstance {
    entity: &'static Entity,
    position: Position,
    part_rotation: Vec<Rotation>
}