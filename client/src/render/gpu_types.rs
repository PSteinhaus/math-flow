use macroquad::prelude::Color;

/// Number of floats stored for each bubble in the position texture.
///
/// Layout:
/// x | y | radius | light
pub const POSITION_RADIUS_FLOATS: usize = 4;

/// Number of floats stored for each bubble in the color texture.
///
/// Layout:
/// r | g | b | pressure
pub const COLOR_PRESSURE_FLOATS: usize = 4;

/// Initial texture capacity (number of bubbles).
///
/// Capacity grows exponentially when needed.
pub const INITIAL_BUBBLE_CAPACITY: usize = 256;

/// Texture unit assignments.
pub const POSITION_TEXTURE_UNIT: i32 = 0;
pub const COLOR_TEXTURE_UNIT: i32 = 1;

/// GPU representation of a bubble before upload.
#[derive(Clone, Copy, Debug)]
pub struct GpuBubble {

    pub x: f32,
    pub y: f32,

    pub radius: f32,

    pub light: f32,

    pub color: Color,

    pub pressure: f32,
}