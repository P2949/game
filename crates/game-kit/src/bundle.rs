//! Tuple component bundles (Phase 4).
//!
//! A [`Bundle`] turns a tuple of components into an [`Entity`], so prefab
//! authoring reads like entity composition instead of chained `.with(..)` calls:
//!
//! ```ignore
//! prefab.spawn(|at| (
//!     Name::new("Player"),
//!     Transform::at(at),
//!     Velocity::default(),
//!     Sprite::new(assets.player, vec2s(20.0)).layer(10),
//!     Collider::box_of(vec2s(20.0)),
//! ));
//! ```

use game_core::world::{Component, Entity};
use glam::Vec2;

/// `Vec2::splat(value)` — a square size/extent. Reads well in content where a
/// sprite and its collider share one dimension: `vec2s(20.0)`.
pub fn vec2s(value: f32) -> Vec2 {
    Vec2::splat(value)
}

/// A group of components that can be inserted into a freshly spawned entity.
///
/// Implemented only for tuples (not for arbitrary components) because the blanket
/// `impl<T: 'static> Component for T` would otherwise make every value — including
/// each tuple — ambiguously both a component and a bundle.
pub trait Bundle {
    /// Builds an [`Entity`] carrying exactly this bundle's components. Unlike
    /// [`Entity::new`], no `Transform`/`Velocity` is inserted implicitly: bundles
    /// declare their components explicitly (e.g. `Transform::at(at)`).
    fn build(self) -> Entity;
}

macro_rules! impl_bundle_for_tuple {
    ($($ty:ident),+) => {
        impl<$($ty: Component),+> Bundle for ($($ty,)+) {
            fn build(self) -> Entity {
                #[allow(non_snake_case)]
                let ($($ty,)+) = self;
                Entity::empty()$(.with($ty))+
            }
        }
    };
}

impl_bundle_for_tuple!(A);
impl_bundle_for_tuple!(A, B);
impl_bundle_for_tuple!(A, B, C);
impl_bundle_for_tuple!(A, B, C, D);
impl_bundle_for_tuple!(A, B, C, D, E);
impl_bundle_for_tuple!(A, B, C, D, E, F);
impl_bundle_for_tuple!(A, B, C, D, E, F, G);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);

#[cfg(test)]
mod tests {
    use super::{Bundle, vec2s};
    use game_core::world::{Transform, Velocity, World};

    #[derive(Clone, Copy)]
    struct Tag;

    #[test]
    fn tuple_bundle_inserts_each_component_without_implicit_transform() {
        let mut world = World::new();
        let id = world.spawn((Transform::at(glam::vec2(1.0, 2.0)), Tag).build());

        assert_eq!(
            world.get::<Transform>(id).unwrap().pos,
            glam::vec2(1.0, 2.0)
        );
        assert!(world.get::<Tag>(id).is_some());
        // No implicit Velocity (unlike `Entity::new`).
        assert!(world.get::<Velocity>(id).is_none());
    }

    #[test]
    fn vec2s_splats() {
        assert_eq!(vec2s(3.0), glam::vec2(3.0, 3.0));
    }
}
