use crate::prelude::*;

#[derive(Debug, States, Clone, Hash, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum Player {
    #[default]
    One,
    Two,
}

#[derive(Component)]
pub(crate) struct Gorilla {
    pub(crate) player: Player,
    pub(crate) name: String,
}

impl Gorilla {
    pub(crate) fn one(name: String) -> Gorilla {
        Gorilla { player: Player::One, name }
    }
    pub(crate) fn two(name: String) -> Gorilla {
        Gorilla { player: Player::Two, name }
    }
}

pub(crate) struct PlayersPlugin;
impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Player>();
    }
}