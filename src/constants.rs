use std::ops::Range;

// Defines the amount of time that should elapse between each physics step.
pub const FIXED_HZ: f64 = 20.0;
pub const TIME_STEP: f32 = (1.0 / 50.0) as f32;
pub const PIXEL_STEP_SIZE: f32 = 20.0;

// Define sizes
pub const BUILDING_WIDTH: f32 = 160.0;
pub const BUILDING_BRICK_WIDTH: f32 = 32.0;
pub const BUILDING_BRICK_HEIGHT: f32 = 8.0;
pub const SCREEN_WIDTH: f32 = 1280.0;
pub const SCREEN_HEIGHT: f32 = 720.0;
pub const BANANA_WIDTH: f32 = 32.0;
pub const BANANA_HEIGHT: f32 = 32.0;
pub const GORILLA_HEIGHT: f32 = 64.0;
pub const GORILLA_WIDTH: f32 = 32.0;
pub const EXPLOSION_START_RADIUS: f32 = BANANA_WIDTH / 2.0;
pub const EXPLOSION_START_DIAMETER: f32 = EXPLOSION_START_RADIUS * 2.0;
pub const EXPLOSION_SIZE: f32 = 3.0;
pub const BRICK_A_STEP_RANGE: Range<f32> = 0.002..0.008;

// Speeds
pub const EXPLOSION_SPEED: f32 = 4.0 * (64.0 / FIXED_HZ as f32); // the more hz the slower
pub const GRAVITY_Y_ACCEL: f32 = -9.8 * PIXEL_STEP_SIZE;
pub const BRICK_EXPLODE_STARTING_VELOCITY_RANGE_X: Range<f32> = -100.0..100.0;
pub const BRICK_EXPLODE_STARTING_VELOCITY_RANGE_Y: Range<f32> = 100.0..400.0;

// Z index
pub const BUILDING_Z_INDEX: f32 = 1.0;
pub const BANANA_Z_INDEX: f32 = 4.0;
pub const GORILLA_Z_INDEX: f32 = 10.0;
pub const THROW_IND_Z_INDEX: f32 = 12.0;
pub const EXPLOSION_Z_INDEX: f32 = 15.0;
pub const WIND_Z_INDEX: f32 = 20.0;
