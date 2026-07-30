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
/// x,y,radius,light
///
/// color_pressure:
/// l,a,b,pressure  <-- (Now storing Oklab instead of RGB!)
#[derive(Default)]
pub struct GpuScene {
    position_radius: Vec<f32>,
    color_pressure: Vec<f32>,
    bubble_count: usize,
}

/// Converts sRGB directly to Oklab for the GPU texture.
fn srgb_to_oklab(color: Color) -> [f32; 3] {
    // 1. sRGB to Linear
    let r = color.r.powf(2.2);
    let g = color.g.powf(2.2);
    let b = color.b.powf(2.2);

    // 2. Linear to LMS
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    // 3. Non-linear step
    let l_ = f32::max(0.0, l).powf(1.0 / 3.0);
    let m_ = f32::max(0.0, m).powf(1.0 / 3.0);
    let s_ = f32::max(0.0, s).powf(1.0 / 3.0);

    // 4. LMS to Oklab
    [
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    ]
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

        // Pre-bake the Oklab conversion here!
        let oklab = srgb_to_oklab(bubble.color);

        self.color_pressure.extend_from_slice(&[
            oklab[0],
            oklab[1],
            oklab[2],
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