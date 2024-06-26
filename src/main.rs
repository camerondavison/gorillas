use bevy::prelude::*;
mod audio;
mod collision;
mod constants;
mod game;
mod players;
mod prelude;

fn main() {
    App::new().add_plugins(crate::game::GamePlugin).run();
}
