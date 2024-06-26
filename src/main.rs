use bevy::prelude::*;
mod constants;
mod prelude;
mod game;
mod audio;
mod collision;
mod players;

fn main() {
    App::new()
        .add_plugins(crate::game::GamePlugin)
        .run();
}