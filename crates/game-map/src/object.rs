use crate::{PrefabId, PropertyBag};

#[derive(Clone, Debug)]
pub struct MapObject {
    pub id: String,
    pub prefab: PrefabId,
    pub position: glam::Vec2,
    pub properties: PropertyBag,
}
