use crate::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioPlugin, AudioSource};
pub(crate) mod prelude {
    pub(crate) use super::GorillasAudioPlugin;
}
#[derive(Resource)]
struct ExplosionSound(Handle<AudioSource>);
pub(crate) struct GorillasAudioPlugin;
impl Plugin for GorillasAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, play_collision_sound);
    }
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let explosion_sounds = asset_server.load("sounds/explosion.mp3");
    commands.insert_resource(ExplosionSound(explosion_sounds));
}
fn play_collision_sound(
    mut collision_events: EventReader<CollisionEvent>,
    audio: Res<Audio>,
    sound: Res<ExplosionSound>,
) {
    if collision_events.read().count() > 0 {
        audio.play(sound.0.clone());
    }
}
