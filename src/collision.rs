use crate::game::{Action, BuildingBrick, Explosion, InGameplaySet};
use crate::prelude::*;
use bevy::math::bounding::{Aabb2d, IntersectsVolume};

#[derive(Resource, Event)]
pub(crate) struct GorillaCollisionEvent {
    pub(crate) player: Player,
}

#[derive(Resource, Event, Debug)]
pub(crate) struct BananaCollisionEvent {
    pub(crate) banana_entity: Entity,
}

#[derive(Component)]
pub(crate) struct Collider;

pub(crate) struct CollisionPlugin;
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GorillaCollisionEvent>()
            .add_event::<BananaCollisionEvent>()
            .add_systems(
                Update,
                (
                    // remove the banana before we check for more collisions
                    spawn_explosion,
                    // check for collisions
                    check_for_collisions_explosion_gorilla.run_if(not(in_state(Action::Winner))),
                    check_for_collisions_banana.run_if(in_state(Action::Watching)),
                )
                    .chain()
                    .in_set(InGameplaySet::Collisions),
            );
    }
}

fn check_for_collisions_explosion_gorilla(
    mut commands: Commands,
    explosion_query: Query<&Transform, With<Explosion>>,
    collider_query: Query<
        (Entity, &Transform, Option<&BuildingBrick>, Option<&Gorilla>),
        With<Collider>,
    >,
    mut collision_event: EventWriter<GorillaCollisionEvent>,
) {
    // look up if explosion has hit something
    for explosion_transform in explosion_query.iter() {
        let mut t = explosion_transform.clone();
        t.scale *= EXPLOSION_START_RADIUS * 2.0; // todo: what is this doing?
        let (_, did_collide_with_gorilla) =
            check_if_did_collide(&mut commands, &collider_query, &t);

        if let Some(player) = did_collide_with_gorilla {
            info!("Collision with {:?}", player);
            collision_event.send(GorillaCollisionEvent { player });
        }
    }
}

fn check_for_collisions_banana(
    mut commands: Commands,
    banana_query: Query<(Entity, &Transform), With<Banana>>,
    collider_query: Query<
        (Entity, &Transform, Option<&BuildingBrick>, Option<&Gorilla>),
        With<Collider>,
    >,
    mut collision_event: EventWriter<BananaCollisionEvent>,
) {
    // look up if our banana has hit something
    for (banana_entity, banana_transform) in banana_query.iter() {
        let (did_collide, _) =
            check_if_did_collide(&mut commands, &collider_query, banana_transform);
        if did_collide {
            info!("banana collided with something");
            collision_event.send(BananaCollisionEvent { banana_entity });
        }
    }
}

fn check_if_did_collide(
    commands: &mut Commands,
    collider_query: &Query<
        (Entity, &Transform, Option<&BuildingBrick>, Option<&Gorilla>),
        With<Collider>,
    >,
    moving_transform: &Transform,
) -> (bool, Option<Player>) {
    let mut did_collide = false;
    let mut did_collide_with_gorilla = None;

    for (e, transform, maybe_building, maybe_gorilla) in collider_query.iter() {
        let collision = Aabb2d::new(
            moving_transform.translation.truncate(),
            moving_transform.scale.truncate() / 2.0,
        )
        .intersects(&Aabb2d::new(
            transform.translation.truncate(),
            transform.scale.truncate() / 2.0,
        ));
        if collision {
            did_collide = true;
            if let Some(g) = maybe_gorilla {
                did_collide_with_gorilla = Some(g.player.clone());
            }
            if maybe_building.is_some() {
                commands.entity(e).despawn_recursive();
            }
        }
    }

    (did_collide, did_collide_with_gorilla)
}

fn spawn_explosion(
    mut commands: Commands,
    query: Query<&Transform, With<Banana>>,
    mut banana_collision_event: EventReader<BananaCollisionEvent>,
) {
    for event in banana_collision_event.read() {
        if let Ok(transform) = query.get(event.banana_entity) {
            let banana_pos = transform.translation.truncate();
            let shape = shapes::RegularPolygon {
                sides: 10,
                feature: shapes::RegularPolygonFeature::Radius(EXPLOSION_START_RADIUS),
                ..shapes::RegularPolygon::default()
            };
            info!("remove banana and spawn explosion");
            // remove the banana
            commands.entity(event.banana_entity).despawn_recursive();
            // add the explosion
            commands.spawn((
                Explosion,
                (
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shape),
                        spatial: SpatialBundle::from_transform(Transform::from_translation(
                            banana_pos.extend(EXPLOSION_Z_INDEX),
                        )),
                        ..default()
                    },
                    Fill::color(Color::ORANGE_RED),
                    Stroke::new(Color::BLACK, 2.0),
                ),
            ));
        }
    }
}
