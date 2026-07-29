use macroquad::texture::Texture2D;

use super::{
    gpu_scene::GpuScene,
    gpu_texture::FloatTexture,
    gpu_types::INITIAL_BUBBLE_CAPACITY,
};


/// GPU resources containing all bubble data.
///
/// The data is stored in two RGBA16F textures:
///
/// position_radius:
///     R = x
///     G = y
///     B = radius
///     A = light
///
/// color_pressure:
///     R = red
///     G = green
///     B = blue
///     A = pressure
///
pub struct BubbleBuffers {
    position_radius: FloatTexture,
    color_pressure: FloatTexture,
    bubble_count: usize,
}


impl BubbleBuffers {

    pub fn new() -> Self {

        Self {
            position_radius:
                FloatTexture::new(
                    INITIAL_BUBBLE_CAPACITY
                ),
            color_pressure:
                FloatTexture::new(
                    INITIAL_BUBBLE_CAPACITY
                ),
            bubble_count: 0,
        }
    }

    /// Uploads the current bubble scene to the GPU.
    pub fn upload(
        &mut self,
        scene: &GpuScene,
    ) {
        self.bubble_count =
            scene.bubble_count();

        self.position_radius
            .ensure_capacity(
                self.bubble_count
            );

        self.color_pressure
            .ensure_capacity(
                self.bubble_count
            );

        self.position_radius
            .upload(
                scene.position_radius_data()
            );

        self.color_pressure
            .upload(
                scene.color_pressure_data()
            );
    }

    pub fn position_radius_texture(
        &self,
    ) -> Texture2D {
        self.position_radius.texture()
    }

    pub fn color_pressure_texture(
        &self,
    ) -> Texture2D {
        self.color_pressure.texture()
    }

    pub fn bubble_count(
        &self,
    ) -> usize {
        self.bubble_count
    }

    pub fn texture_width(&self) -> u32 {
        self.position_radius.width()
    }
}