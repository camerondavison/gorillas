use crate::constants::TIME_STEP;
use crate::game::{Explosion, InGameplaySet};
use crate::prelude::*;

#[derive(Component, Deref, DerefMut, Debug)]
pub(crate) struct GlobalWorldAcceleration(pub(crate) Vec2);

#[derive(Component, Deref, DerefMut, Debug)]
pub(crate) struct Velocity(pub(crate) Vec2);

#[derive(Component, Deref, DerefMut, Debug)]
pub(crate) struct Rotation(pub(crate) Quat);

#[derive(Component, Debug)]
pub(crate) struct MovementState{
    old_position: Vec2,
    cur_position: Vec2,
}

impl MovementState {
    pub(crate) fn new(current_position: Vec2) -> Self {
        MovementState { old_position: current_position, cur_position: current_position }
    }
}

#[derive(Component)]
pub(crate) struct Gravity;

pub(crate) struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gravity).add_systems(
            FixedUpdate,
            (
                ((apply_acceleration, apply_velocity, lerp_velocity).chain(), apply_rotation)
                    .in_set(InGameplaySet::Movement),
                (animate_explosion).in_set(InGameplaySet::Collisions),
            ),
        );
    }
}

fn animate_explosion(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut Transform), With<Explosion>>,
) {
    for (e, ref mut t) in explosion_query.iter_mut() {
        if t.scale.x > EXPLOSION_SIZE {
            commands.entity(e).despawn_recursive();
        } else {
            t.scale *= 1.0 + EXPLOSION_SPEED * TIME_STEP;
        }
    }
}

fn setup_gravity(mut commands: Commands) {
    commands.spawn((Gravity, GlobalWorldAcceleration(Vec2::new(0.0, GRAVITY_Y_ACCEL))));
}

fn apply_acceleration(
    fixed_time: Res<Time<Fixed>>,
    acceleration_query: Query<&GlobalWorldAcceleration>,
    mut velocity_query: Query<&mut Velocity>,
) {
    for acc in acceleration_query.iter() {
        for mut velocity in velocity_query.iter_mut() {
            velocity.x += acc.x * fixed_time.delta_seconds();
            velocity.y += acc.y * fixed_time.delta_seconds();
        }
    }
}

fn apply_velocity(fixed_time: Res<Time<Fixed>>, mut query: Query<(&Velocity, &mut MovementState)>) {
    for (velocity, mut movement) in query.iter_mut() {
        movement.old_position = movement.cur_position;
        movement.cur_position += velocity.0 * fixed_time.delta_seconds();
    }
}

fn lerp_velocity(fixed_time: Res<Time<Fixed>>, mut query: Query<(&mut Transform, &mut MovementState)>) {
    let a = fixed_time.overstep_fraction();
    for (mut transform, movement) in query.iter_mut() {
        transform.translation = movement.old_position.lerp(movement.cur_position, a).extend(transform.translation.z)
    }
}

fn apply_rotation(mut query: Query<(&mut Transform, &Rotation)>) {
    for (mut transform, rotation) in query.iter_mut() {
        transform.rotation *= rotation.0;
    }
}
