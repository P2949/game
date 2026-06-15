use crate::PropertyBag;

#[derive(Clone, Debug)]
pub struct MapRegion {
    pub id: String,
    pub shape: RegionShape,
    pub tags: Tags,
    pub properties: PropertyBag,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RegionShape {
    Rect { min: glam::Vec2, max: glam::Vec2 },
    Circle { center: glam::Vec2, radius: f32 },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tags {
    values: Vec<String>,
}

impl Tags {
    pub fn new(values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            values: values.into_iter().map(Into::into).collect(),
        }
    }

    pub fn contains(&self, tag: &str) -> bool {
        self.values.iter().any(|value| value == tag)
    }
}
