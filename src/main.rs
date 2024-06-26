use bevy::prelude::*;
mod audio;
mod collision;
mod constants;
mod game;
mod physics;
mod players;
mod prelude;

fn main() {
    App::new().add_plugins(game::GamePlugin).run();
}
