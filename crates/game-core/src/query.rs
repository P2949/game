//! Typed component queries for signature-driven advanced systems.

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use anyhow::{Result, bail};

use crate::input::Input;
use crate::world::{Component, EntityId, World};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AccessMode {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug)]
struct AccessRecord {
    type_id: TypeId,
    type_name: &'static str,
    mode: AccessMode,
}

/// Accesses declared by one parameter system.
#[doc(hidden)]
#[derive(Default)]
pub struct ParamAccess {
    components: Vec<AccessRecord>,
    resources: Vec<AccessRecord>,
}

impl ParamAccess {
    fn read_component<T: Component>(&mut self) {
        self.components.push(AccessRecord {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            mode: AccessMode::Read,
        });
    }

    fn write_component<T: Component>(&mut self) {
        self.components.push(AccessRecord {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            mode: AccessMode::Write,
        });
    }

    fn read_resource<T: 'static>(&mut self) {
        self.resources.push(AccessRecord {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            mode: AccessMode::Read,
        });
    }

    fn write_resource<T: 'static>(&mut self) {
        self.resources.push(AccessRecord {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            mode: AccessMode::Write,
        });
    }

    /// Rejects every shared/exclusive or duplicate exclusive component alias.
    pub fn validate(&self) -> Result<()> {
        validate_access_group("component", &self.components)?;
        validate_access_group("resource", &self.resources)?;
        Ok(())
    }
}

fn validate_access_group(kind: &str, records: &[AccessRecord]) -> Result<()> {
    let mut grouped = HashMap::<TypeId, Vec<AccessRecord>>::new();
    for record in records {
        grouped.entry(record.type_id).or_default().push(*record);
    }

    for records in grouped.into_values() {
        let writes = records
            .iter()
            .filter(|record| record.mode == AccessMode::Write)
            .count();
        if writes > 0 && records.len() > 1 {
            bail!(
                "parameter-system registration rejected conflicting {kind} access to '{}': an exclusive mutable borrow cannot be combined with another shared or exclusive borrow in one system",
                records[0].type_name
            );
        }
    }
    Ok(())
}

mod sealed {
    pub trait Sealed {}
}

/// A value that can be extracted for one signature-driven system execution.
///
/// The trait is sealed while the scheduler owns the raw pointer extraction
/// boundary. The supported values are Query, Res, ResMut, and DeltaTime.
pub trait SystemParam: sealed::Sealed {
    /// The frame-bound value passed to a system.
    type Item<'w>;

    #[doc(hidden)]
    fn register_access(access: &mut ParamAccess);

    /// # Safety
    ///
    /// The complete parameter list must have passed ParamAccess validation, and
    /// the returned value must not escape the current frame.
    #[doc(hidden)]
    unsafe fn extract<'w>(context: ParamContext<'w>) -> Self::Item<'w>;
}

/// Frame data used internally while extracting a SystemParam.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct ParamContext<'w> {
    world: *mut World,
    input: &'w Input,
    delta_seconds: f32,
}

impl<'w> ParamContext<'w> {
    fn new(world: *mut World, input: &'w Input, delta_seconds: f32) -> Self {
        Self {
            world,
            input,
            delta_seconds,
        }
    }
}

/// A plain function that can run from the scheduler with extracted parameters.
pub trait ParamSystem<Marker>: 'static {
    /// Validates this system's full parameter access set at registration time.
    fn validate_params(&self) -> Result<()>;

    /// Runs the system once using the current frame's world, input, and delta.
    fn run_params(&mut self, world: &mut World, input: &Input, delta_seconds: f32);
}

impl<Func> ParamSystem<fn()> for Func
where
    Func: 'static + FnMut(),
{
    fn validate_params(&self) -> Result<()> {
        Ok(())
    }

    fn run_params(&mut self, _world: &mut World, _input: &Input, _delta_seconds: f32) {
        self()
    }
}

macro_rules! impl_query_param_system {
    ($($data:ident => $filter:ident),+ $(,)?) => {
        impl<Func, $($data, $filter),+>
            ParamSystem<fn($(Query<'static, $data, $filter>),+)> for Func
        where
            Func: 'static + for<'w> FnMut($(Query<'w, $data, $filter>),+),
            $($data: QueryData, $filter: QueryFilter),+
        {
            fn validate_params(&self) -> Result<()> {
                let mut access = ParamAccess::default();
                $(
                    $data::register_access(&mut access);
                    $filter::register_access(&mut access);
                )+
                access.validate()
            }

            fn run_params(
                &mut self,
                world: &mut World,
                input: &Input,
                delta_seconds: f32,
            ) {
                let context = ParamContext::new(world as *mut World, input, delta_seconds);
                // SAFETY: validate_params ran before this system was scheduled.
                unsafe {
                    self($(Query::<'_, $data, $filter>::new(context.world)),+);
                }
            }
        }
    };
}

impl_query_param_system!(D1 => F1);
impl_query_param_system!(D1 => F1, D2 => F2);
impl_query_param_system!(D1 => F1, D2 => F2, D3 => F3);
impl_query_param_system!(D1 => F1, D2 => F2, D3 => F3, D4 => F4);
impl_query_param_system!(D1 => F1, D2 => F2, D3 => F3, D4 => F4, D5 => F5);
impl_query_param_system!(D1 => F1, D2 => F2, D3 => F3, D4 => F4, D5 => F5, D6 => F6);
impl_query_param_system!(
    D1 => F1,
    D2 => F2,
    D3 => F3,
    D4 => F4,
    D5 => F5,
    D6 => F6,
    D7 => F7
);
impl_query_param_system!(
    D1 => F1,
    D2 => F2,
    D3 => F3,
    D4 => F4,
    D5 => F5,
    D6 => F6,
    D7 => F7,
    D8 => F8
);

macro_rules! impl_query_res_system {
    ($resource:ident) => {
        impl<Func, D, F, T> ParamSystem<fn(Query<'static, D, F>, $resource<'static, T>)> for Func
        where
            Func: 'static + for<'w> FnMut(Query<'w, D, F>, $resource<'w, T>),
            D: QueryData,
            F: QueryFilter,
            T: 'static,
        {
            fn validate_params(&self) -> Result<()> {
                let mut access = ParamAccess::default();
                D::register_access(&mut access);
                F::register_access(&mut access);
                <$resource<'static, T> as SystemParam>::register_access(&mut access);
                access.validate()
            }

            fn run_params(&mut self, world: &mut World, input: &Input, delta_seconds: f32) {
                let context = ParamContext::new(world as *mut World, input, delta_seconds);
                // SAFETY: validate_params ran before this system was scheduled.
                unsafe {
                    self(
                        Query::<'_, D, F>::new(context.world),
                        <$resource<'static, T> as SystemParam>::extract(context),
                    );
                }
            }
        }
    };
}

impl_query_res_system!(Res);
impl_query_res_system!(ResMut);

macro_rules! impl_resource_param_system {
    ($resource:ident) => {
        impl<Func, T> ParamSystem<fn($resource<'static, T>)> for Func
        where
            Func: 'static + for<'w> FnMut($resource<'w, T>),
            T: 'static,
        {
            fn validate_params(&self) -> Result<()> {
                let mut access = ParamAccess::default();
                <$resource<'static, T> as SystemParam>::register_access(&mut access);
                access.validate()
            }

            fn run_params(&mut self, world: &mut World, input: &Input, delta_seconds: f32) {
                let context = ParamContext::new(world as *mut World, input, delta_seconds);
                // SAFETY: validate_params ran before this system was scheduled.
                unsafe {
                    self(<$resource<'static, T> as SystemParam>::extract(context));
                }
            }
        }

        impl<Func, T> ParamSystem<fn($resource<'static, T>, DeltaTime)> for Func
        where
            Func: 'static + for<'w> FnMut($resource<'w, T>, DeltaTime),
            T: 'static,
        {
            fn validate_params(&self) -> Result<()> {
                let mut access = ParamAccess::default();
                <$resource<'static, T> as SystemParam>::register_access(&mut access);
                access.validate()
            }

            fn run_params(&mut self, world: &mut World, input: &Input, delta_seconds: f32) {
                let context = ParamContext::new(world as *mut World, input, delta_seconds);
                // SAFETY: validate_params ran before this system was scheduled.
                unsafe {
                    self(
                        <$resource<'static, T> as SystemParam>::extract(context),
                        DeltaTime(delta_seconds),
                    );
                }
            }
        }
    };
}

impl_resource_param_system!(Res);
impl_resource_param_system!(ResMut);

impl<Func> ParamSystem<fn(DeltaTime)> for Func
where
    Func: 'static + FnMut(DeltaTime),
{
    fn validate_params(&self) -> Result<()> {
        Ok(())
    }

    fn run_params(&mut self, _world: &mut World, _input: &Input, delta_seconds: f32) {
        self(DeltaTime(delta_seconds))
    }
}

impl<Func, D, F> ParamSystem<fn(Query<'static, D, F>, DeltaTime)> for Func
where
    Func: 'static + for<'w> FnMut(Query<'w, D, F>, DeltaTime),
    D: QueryData,
    F: QueryFilter,
{
    fn validate_params(&self) -> Result<()> {
        let mut access = ParamAccess::default();
        D::register_access(&mut access);
        F::register_access(&mut access);
        access.validate()
    }

    fn run_params(&mut self, world: &mut World, input: &Input, delta_seconds: f32) {
        let context = ParamContext::new(world as *mut World, input, delta_seconds);
        // SAFETY: validate_params ran before this system was scheduled.
        unsafe {
            self(
                Query::<'_, D, F>::new(context.world),
                DeltaTime(delta_seconds),
            );
        }
    }
}

impl<Func, D, F, T> ParamSystem<fn(Query<'static, D, F>, Res<'static, T>, DeltaTime)> for Func
where
    Func: 'static + for<'w> FnMut(Query<'w, D, F>, Res<'w, T>, DeltaTime),
    D: QueryData,
    F: QueryFilter,
    T: 'static,
{
    fn validate_params(&self) -> Result<()> {
        let mut access = ParamAccess::default();
        D::register_access(&mut access);
        F::register_access(&mut access);
        <Res<'static, T> as SystemParam>::register_access(&mut access);
        access.validate()
    }

    fn run_params(&mut self, world: &mut World, input: &Input, delta_seconds: f32) {
        let context = ParamContext::new(world as *mut World, input, delta_seconds);
        // SAFETY: validate_params ran before this system was scheduled.
        unsafe {
            self(
                Query::<'_, D, F>::new(context.world),
                <Res<'static, T> as SystemParam>::extract(context),
                DeltaTime(delta_seconds),
            );
        }
    }
}

/// Selects only entities that contain T.
#[derive(Clone, Copy, Debug, Default)]
pub struct With<T: Component>(PhantomData<fn() -> T>);

/// Selects only entities that do not contain T.
#[derive(Clone, Copy, Debug, Default)]
pub struct Without<T: Component>(PhantomData<fn() -> T>);

/// Predicate applied to every candidate entity in a Query.
pub trait QueryFilter {
    #[doc(hidden)]
    fn matches(world: &World, id: EntityId) -> bool;

    #[doc(hidden)]
    fn register_access(_access: &mut ParamAccess) {}
}

impl QueryFilter for () {
    fn matches(_world: &World, _id: EntityId) -> bool {
        true
    }
}

impl<T: Component> QueryFilter for With<T> {
    fn matches(world: &World, id: EntityId) -> bool {
        world.has::<T>(id)
    }

    fn register_access(access: &mut ParamAccess) {
        access.read_component::<T>();
    }
}

impl<T: Component> QueryFilter for Without<T> {
    fn matches(world: &World, id: EntityId) -> bool {
        !world.has::<T>(id)
    }

    fn register_access(access: &mut ParamAccess) {
        access.read_component::<T>();
    }
}

/// The component references fetched for each Query entity.
pub trait QueryData {
    /// The references yielded for one matching entity.
    type Item<'w>;

    #[doc(hidden)]
    fn register_access(access: &mut ParamAccess);

    #[doc(hidden)]
    fn candidate_ids(world: &World) -> Vec<EntityId>;

    /// # Safety
    ///
    /// The caller must prove that the returned references do not alias another
    /// query access for their whole lifetime.
    #[doc(hidden)]
    unsafe fn fetch<'w>(world: *mut World, id: EntityId) -> Option<Self::Item<'w>>;
}

impl<T: Component> QueryData for &T {
    type Item<'w> = &'w T;

    fn register_access(access: &mut ParamAccess) {
        access.read_component::<T>();
    }

    fn candidate_ids(world: &World) -> Vec<EntityId> {
        world.ids_with::<T>()
    }

    unsafe fn fetch<'w>(world: *mut World, id: EntityId) -> Option<Self::Item<'w>> {
        // SAFETY: the caller validated all extracted query accesses.
        let component = unsafe { (*world).component_ptr::<T>(id)? };
        Some(unsafe { &*component })
    }
}

impl<T: Component> QueryData for &mut T {
    type Item<'w> = &'w mut T;

    fn register_access(access: &mut ParamAccess) {
        access.write_component::<T>();
    }

    fn candidate_ids(world: &World) -> Vec<EntityId> {
        world.ids_with::<T>()
    }

    unsafe fn fetch<'w>(world: *mut World, id: EntityId) -> Option<Self::Item<'w>> {
        // SAFETY: registration rejects every alias of this mutable component.
        let component = unsafe { (*world).component_ptr::<T>(id)? };
        Some(unsafe { &mut *component })
    }
}

impl<A: QueryData> QueryData for (A,) {
    type Item<'w> = (A::Item<'w>,);

    fn register_access(access: &mut ParamAccess) {
        A::register_access(access);
    }

    fn candidate_ids(world: &World) -> Vec<EntityId> {
        A::candidate_ids(world)
    }

    unsafe fn fetch<'w>(world: *mut World, id: EntityId) -> Option<Self::Item<'w>> {
        Some((unsafe { A::fetch(world, id)? },))
    }
}

macro_rules! impl_query_data_tuple {
    ($first:ident, $($rest:ident),+ $(,)?) => {
        impl<$first, $($rest),+> QueryData for ($first, $($rest,)+)
        where
            $first: QueryData,
            $($rest: QueryData,)+
        {
            type Item<'w> = ($first::Item<'w>, $($rest::Item<'w>,)+);

            fn register_access(access: &mut ParamAccess) {
                $first::register_access(access);
                $($rest::register_access(access);)+
            }

            fn candidate_ids(world: &World) -> Vec<EntityId> {
                $first::candidate_ids(world)
            }

            unsafe fn fetch<'w>(
                world: *mut World,
                id: EntityId,
            ) -> Option<Self::Item<'w>> {
                Some((
                    unsafe { $first::fetch(world, id)? },
                    $(unsafe { $rest::fetch(world, id)? },)+
                ))
            }
        }
    };
}

impl_query_data_tuple!(A, B);
impl_query_data_tuple!(A, B, C);
impl_query_data_tuple!(A, B, C, D);
impl_query_data_tuple!(A, B, C, D, E);
impl_query_data_tuple!(A, B, C, D, E, F);
impl_query_data_tuple!(A, B, C, D, E, F, G);
impl_query_data_tuple!(A, B, C, D, E, F, G, H);

/// A typed component view selected by a signature-driven system.
pub struct Query<'w, D: QueryData, F: QueryFilter = ()> {
    world: NonNull<World>,
    marker: PhantomData<(&'w mut World, D, F)>,
}

impl<'w, D: QueryData, F: QueryFilter> Query<'w, D, F> {
    pub(crate) unsafe fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("query extraction requires a world"),
            marker: PhantomData,
        }
    }

    /// Iterates matching entities in stable entity-id order.
    pub fn iter(&mut self) -> QueryIter<'_, D, F> {
        // SAFETY: the query pointer comes from a live scheduler frame.
        let ids = unsafe { D::candidate_ids(self.world.as_ref()) };
        QueryIter {
            world: self.world,
            ids,
            next: 0,
            marker: PhantomData,
        }
    }

    /// Fetches components for one matching entity.
    pub fn get(&mut self, id: EntityId) -> Option<D::Item<'_>> {
        // SAFETY: the query pointer comes from a live scheduler frame.
        let matches = unsafe { F::matches(self.world.as_ref(), id) };
        if !matches {
            return None;
        }
        // SAFETY: registration validated the query access list.
        unsafe { D::fetch(self.world.as_ptr(), id) }
    }
}

impl<'query, 'world, D: QueryData, F: QueryFilter> IntoIterator
    for &'query mut Query<'world, D, F>
{
    type Item = (EntityId, D::Item<'query>);
    type IntoIter = QueryIter<'query, D, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator produced by Query::iter.
pub struct QueryIter<'q, D: QueryData, F: QueryFilter> {
    world: NonNull<World>,
    ids: Vec<EntityId>,
    next: usize,
    marker: PhantomData<(&'q mut (), D, F)>,
}

impl<'q, D: QueryData, F: QueryFilter> Iterator for QueryIter<'q, D, F> {
    type Item = (EntityId, D::Item<'q>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&id) = self.ids.get(self.next) {
            self.next += 1;
            // SAFETY: the filter's immutable borrow ends before fetch.
            let matches = unsafe { F::matches(self.world.as_ref(), id) };
            if !matches {
                continue;
            }
            // SAFETY: registration validated the query access list.
            if let Some(data) = unsafe { D::fetch(self.world.as_ptr(), id) } {
                return Some((id, data));
            }
        }
        None
    }
}

impl<'a, D: QueryData, F: QueryFilter> sealed::Sealed for Query<'a, D, F> {}

impl<'a, D: QueryData, F: QueryFilter> SystemParam for Query<'a, D, F> {
    type Item<'w> = Query<'w, D, F>;

    fn register_access(access: &mut ParamAccess) {
        D::register_access(access);
        F::register_access(access);
    }

    unsafe fn extract<'w>(context: ParamContext<'w>) -> Self::Item<'w> {
        // SAFETY: the parameter context is created from the active frame.
        unsafe { Query::new(context.world) }
    }
}

/// Shared access to a resource during one system execution.
pub struct Res<'w, T: 'static> {
    value: &'w T,
}

impl<'w, T: 'static> Deref for Res<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: 'static> sealed::Sealed for Res<'a, T> {}

impl<'a, T: 'static> SystemParam for Res<'a, T> {
    type Item<'w> = Res<'w, T>;

    fn register_access(access: &mut ParamAccess) {
        access.read_resource::<T>();
    }

    unsafe fn extract<'w>(context: ParamContext<'w>) -> Self::Item<'w> {
        let pointer = if TypeId::of::<T>() == TypeId::of::<Input>() {
            context.input as *const Input as *const T
        } else {
            // SAFETY: ParamContext carries a live world pointer for this frame.
            unsafe { (&*context.world).get_resource::<T>() }
                .unwrap_or_else(|| {
                    panic!(
                        "parameter-system resource '{}' is unavailable; insert it before running the system",
                        std::any::type_name::<T>()
                    )
                }) as *const T
        };

        Res {
            // SAFETY: registration rejects an aliasing ResMut parameter.
            value: unsafe { &*pointer },
        }
    }
}

/// Exclusive access to a resource during one system execution.
pub struct ResMut<'w, T: 'static> {
    value: &'w mut T,
}

impl<'w, T: 'static> Deref for ResMut<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T: 'static> DerefMut for ResMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'a, T: 'static> sealed::Sealed for ResMut<'a, T> {}

impl<'a, T: 'static> SystemParam for ResMut<'a, T> {
    type Item<'w> = ResMut<'w, T>;

    fn register_access(access: &mut ParamAccess) {
        access.write_resource::<T>();
    }

    unsafe fn extract<'w>(context: ParamContext<'w>) -> Self::Item<'w> {
        assert_ne!(
            TypeId::of::<T>(),
            TypeId::of::<Input>(),
            "Input is frame-scoped and cannot be requested as ResMut<Input>"
        );
        // SAFETY: ParamContext carries a live world pointer for this frame.
        let pointer = unsafe { (&mut *context.world).get_resource_mut::<T>() }
            .unwrap_or_else(|| {
                panic!(
                    "parameter-system resource '{}' is unavailable; insert it before running the system",
                    std::any::type_name::<T>()
                )
            }) as *mut T;

        ResMut {
            // SAFETY: registration rejects every alias of this resource.
            value: unsafe { &mut *pointer },
        }
    }
}

/// Duration of the current frame or fixed timestep.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeltaTime(pub f32);

impl sealed::Sealed for DeltaTime {}

impl SystemParam for DeltaTime {
    type Item<'w> = DeltaTime;

    fn register_access(_access: &mut ParamAccess) {}

    unsafe fn extract<'w>(context: ParamContext<'w>) -> Self::Item<'w> {
        DeltaTime(context.delta_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::{ParamAccess, Query, QueryData, With};
    use crate::world::{Entity, Transform, Velocity, World};
    use glam::Vec2;

    #[derive(Clone, Copy)]
    struct Marker;

    #[test]
    fn query_iterates_filtered_read_write_components() {
        let mut world = World::new();
        let kept = world.spawn(
            Entity::new(Vec2::ZERO)
                .with(Transform::at(Vec2::new(3.0, 4.0)))
                .with(Velocity(Vec2::new(2.0, 0.0)))
                .with(Marker),
        );
        world.spawn(
            Entity::new(Vec2::ZERO)
                .with(Transform::at(Vec2::new(9.0, 9.0)))
                .with(Velocity(Vec2::new(1.0, 0.0))),
        );

        {
            let mut query =
                unsafe { Query::<(&mut Transform, &Velocity), With<Marker>>::new(&mut world) };
            for (id, (transform, velocity)) in &mut query {
                assert_eq!(id, kept);
                transform.pos += velocity.0;
            }
        }

        assert_eq!(
            world.get::<Transform>(kept).unwrap().pos,
            Vec2::new(5.0, 4.0)
        );
    }

    #[test]
    fn access_validation_rejects_read_write_aliases() {
        let mut access = ParamAccess::default();
        <(&Transform, &mut Transform) as QueryData>::register_access(&mut access);
        let error = access.validate().unwrap_err().to_string();
        assert!(error.contains("conflicting component access"));
        assert!(error.contains("Transform"));
    }
}
