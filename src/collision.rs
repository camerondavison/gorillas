use crate::prelude::*;
#[derive(Default, Resource, Event)]
pub(crate) struct CollisionEvent;
pub(crate) struct CollisionPlugin;
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionEvent>();
    }
}
