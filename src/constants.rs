// Defines the amount of time that should elapse between each physics step.
pub const TIME_STEP: f32 = 1.0 / 60.0;
pub const PIXEL_STEP_SIZE: f32 = 10.0;

// Define sizes
pub const BUILDING_WIDTH: f32 = 160.0;
pub const BUILDING_BRICK_WIDTH: f32 = 32.0; //20.0
pub const BUILDING_BRICK_HEIGHT: f32 = 8.0; //5.0
pub const SCREEN_WIDTH: f32 = 1280.0;
pub const SCREEN_HEIGHT: f32 = 720.0;
pub const BANANA_WIDTH: f32 = 32.0;
pub const BANANA_HEIGHT: f32 = 32.0;
pub const GORILLA_HEIGHT: f32 = 64.0;
pub const GORILLA_WIDTH: f32 = 32.0;
pub const EXPLOSION_START_RADIUS: f32 = BANANA_WIDTH / 2.0;
pub const EXPLOSION_SIZE: f32 = 3.0;

// Speeds
pub const GRAVITY_Y_ACCEL: f32 = -9.8 * PIXEL_STEP_SIZE;

// Z index
pub const BUILDING_Z_INDEX: f32 = 1.0;
pub const BANANA_Z_INDEX: f32 = 4.0;
pub const GORILLA_Z_INDEX: f32 = 10.0;
pub const THROW_IND_Z_INDEX: f32 = 12.0;
pub const EXPLOSION_Z_INDEX: f32 = 15.0;
pub const WIND_Z_INDEX: f32 = 20.0;
