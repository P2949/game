use game_core::input::Input;
use game_core::query::{DeltaTime, ParamSystem, Query, ResMut, With};
use game_core::world::{Entity, World};

#[derive(Clone, Copy)]
struct Position(f32);

#[derive(Clone, Copy)]
struct Speed(f32);

#[derive(Clone, Copy)]
struct Mover;

#[derive(Clone, Copy)]
struct Counter(f32);

fn move_entities(
    mut positions: Query<&mut Position, With<Mover>>,
    mut speeds: Query<&Speed, With<Mover>>,
) {
    let speed = speeds
        .iter()
        .next()
        .map(|(_, speed)| speed.0)
        .unwrap_or_default();
    for (_, position) in &mut positions {
        position.0 += speed;
    }
}

fn conflicting_entities(_write: Query<&mut Position>, _read: Query<&Position>) {}

fn conflicting_mutable_entities(_first: Query<&mut Position>, _second: Query<&mut Position>) {}

fn conflicting_filter_access(
    _writers: Query<&mut Mover>,
    _filtered: Query<&Position, With<Mover>>,
) {
}

fn advance_counter(mut counter: ResMut<Counter>, dt: DeltaTime) {
    counter.0 += dt.0;
}

fn run_disjoint<S>(system: &mut S, world: &mut World)
where
    S: ParamSystem<
        fn(
            Query<'static, &'static mut Position, With<Mover>>,
            Query<'static, &'static Speed, With<Mover>>,
        ),
    >,
{
    system.validate_params().unwrap();
    system.run_params(world, &Input::default(), 0.0);
}

fn registration_error<S>(system: &S) -> String
where
    S: ParamSystem<fn(Query<'static, &'static mut Position>, Query<'static, &'static Position>)>,
{
    system.validate_params().unwrap_err().to_string()
}

fn mutable_registration_error<S>(system: &S) -> String
where
    S: ParamSystem<
        fn(Query<'static, &'static mut Position>, Query<'static, &'static mut Position>),
    >,
{
    system.validate_params().unwrap_err().to_string()
}

fn filter_registration_error<S>(system: &S) -> String
where
    S: ParamSystem<
        fn(Query<'static, &'static mut Mover>, Query<'static, &'static Position, With<Mover>>),
    >,
{
    system.validate_params().unwrap_err().to_string()
}

fn run_resource<S>(system: &mut S, world: &mut World)
where
    S: ParamSystem<fn(ResMut<'static, Counter>, DeltaTime)>,
{
    system.validate_params().unwrap();
    system.run_params(world, &Input::default(), 0.5);
}

#[test]
fn two_non_overlapping_queries_run_through_the_parameter_adapter() {
    let mut world = World::new();
    let entity = world.spawn(
        Entity::new(glam::Vec2::ZERO)
            .with(Position(3.0))
            .with(Speed(2.0))
            .with(Mover),
    );
    let mut system = move_entities;

    run_disjoint(&mut system, &mut world);

    assert_eq!(world.get::<Position>(entity).unwrap().0, 5.0);
}

#[test]
fn read_and_write_to_the_same_component_are_rejected_at_registration() {
    let error = registration_error(&conflicting_entities);
    assert!(error.contains("conflicting component access"));
    assert!(error.contains("Position"));
}

#[test]
fn two_mutable_queries_for_the_same_component_are_rejected_at_registration() {
    let error = mutable_registration_error(&conflicting_mutable_entities);
    assert!(error.contains("conflicting component access"));
    assert!(error.contains("Position"));
}

#[test]
fn a_filter_read_conflicts_with_a_mutable_query_for_that_component() {
    let error = filter_registration_error(&conflicting_filter_access);
    assert!(error.contains("conflicting component access"));
    assert!(error.contains("Mover"));
}

#[test]
fn resource_parameters_are_extracted_and_mutated_for_one_execution() {
    let mut world = World::new();
    world.insert_resource(Counter(2.0));
    let mut system = advance_counter;

    run_resource(&mut system, &mut world);

    assert_eq!(world.get_resource::<Counter>().unwrap().0, 2.5);
}
