use crate::arrow;
use crate::prelude::*;
use bevy::input::common_conditions::input_just_pressed;
use rand::{thread_rng, RngCore};

// Marker component
#[derive(Component)]
struct Wind;

pub(crate) struct WindPlugin;
impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_wind).add_systems(
            Update,
            wind_changer.run_if(input_just_pressed(KeyCode::KeyW)),
        );
    }
}

fn setup_wind(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_wind_wth_accel(&mut commands, asset_server);
}

fn spawn_wind_wth_accel(commands: &mut Commands, asset_server: Res<AssetServer>) {
    let font_medium = asset_server.load("fonts/FiraMono-Medium.ttf");
    let mut rng = thread_rng();
    let wind = (rng.next_u32() % 40) as i32 - 20;
    info!("new wind of {}", wind);
    let raw_length = wind as i16 * 10i16;
    let top = 50.0;
    let right = 300.0;
    let y = SCREEN_HEIGHT / 2.0 - top;
    let x = SCREEN_WIDTH / 2.0 - right;

    commands.spawn((
        Wind,
        arrow::build_arrow_shape(
            Color::DARK_GRAY,
            Color::GRAY,
            raw_length,
            30,
            x,
            y,
            WIND_Z_INDEX,
        ),
        GlobalWorldAcceleration(Vec2::new(wind as f32, 0.0)),
    ));

    commands.spawn((
        Wind,
        wind_text_bundle(font_medium, top - 30.0, right, "wind".to_string()),
    ));
}

fn wind_text_bundle(font_medium: Handle<Font>, top: f32, right: f32, value: String) -> TextBundle {
    TextBundle {
        text: Text {
            sections: vec![TextSection {
                value,
                style: TextStyle {
                    font: font_medium,
                    font_size: 30.0,
                    color: Color::BLACK,
                },
            }],
            ..default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(top),
            right: Val::Px(right),
            ..default()
        },
        ..default()
    }
}

fn wind_changer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    wind_query: Query<Entity, With<Wind>>,
) {
    for we in wind_query.iter() {
        commands.entity(we).despawn();
    }
    spawn_wind_wth_accel(&mut commands, asset_server);
}
