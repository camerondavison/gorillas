use crate::prelude::*;

#[derive(Debug, States, Clone, Hash, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum Player {
    #[default]
    One,
    Two,
}
#[derive(Component)]
pub(crate) struct Gorilla(pub Player);

pub(crate) struct PlayersPlugin;
impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Player>();
    }
}