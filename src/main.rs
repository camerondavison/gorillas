use bevy::prelude::*;
mod arrow;
mod audio;
mod banana;
mod collision;
mod constants;
mod game;
mod physics;
mod players;
mod prelude;
mod wind;

fn main() {
    App::new().add_plugins(game::GamePlugin).run();
}
