use macroquad::prelude::*;

use super::gpu_types::{
    COLOR_PRESSURE_FLOATS,
    GpuBubble,
    POSITION_RADIUS_FLOATS,
};

/// CPU-side representation of all GPU upload buffers.
///
/// The layout here mirrors the GPU textures exactly.
///
/// position_radius:
///
/// x,y,radius,light
///
/// color_pressure:
///
/// r,g,b,pressure
#[derive(Default)]
pub struct GpuScene {

    position_radius: Vec<f32>,

    color_pressure: Vec<f32>,

    bubble_count: usize,
}

impl GpuScene {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {

        self.position_radius.clear();

        self.color_pressure.clear();

        self.bubble_count = 0;
    }

    pub fn reserve(&mut self, bubbles: usize) {

        self.position_radius
            .reserve(bubbles * POSITION_RADIUS_FLOATS);

        self.color_pressure
            .reserve(bubbles * COLOR_PRESSURE_FLOATS);
    }

    pub fn push_bubble(&mut self, bubble: GpuBubble) {

        self.position_radius.extend_from_slice(&[
            bubble.x,
            bubble.y,
            bubble.radius,
            bubble.light,
        ]);

        self.color_pressure.extend_from_slice(&[
            bubble.color.r,
            bubble.color.g,
            bubble.color.b,
            bubble.pressure,
        ]);

        self.bubble_count += 1;
    }

    pub fn bubble_count(&self) -> usize {
        self.bubble_count
    }

    pub fn position_radius_data(&self) -> &[f32] {
        &self.position_radius
    }

    pub fn color_pressure_data(&self) -> &[f32] {
        &self.color_pressure
    }
}