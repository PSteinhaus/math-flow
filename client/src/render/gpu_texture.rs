use macroquad::prelude::*;
use macroquad::miniquad::{
    TextureAccess,
    TextureFormat,
    TextureParams,
    TextureSource,
};
use half::f16;

use crate::render::gpu_types::INITIAL_BUBBLE_CAPACITY;


/// A GPU texture storing packed float data.
///
/// This is used as a data buffer for shaders rather than
/// as an image.
///
/// Each texel contains four float values:
///
/// R G B A
///
/// The texture dimensions are width x 1.
pub struct FloatTexture {
    texture: Texture2D,
    width: u32,
    capacity: usize,
}


impl FloatTexture {
    /// Creates a new empty float texture.
    ///
    /// The texture is initially sized to the requested capacity.
    pub fn new(capacity: usize) -> Self {
        let width = capacity.max(1) as u32;
        let texture =
            create_empty_float_texture(width, 1);

        texture.set_filter(FilterMode::Nearest);

        Self {
            texture,
            width,
            capacity,
        }
    }

    /// Returns the underlying Macroquad texture.
    pub fn texture(&self) -> Texture2D {
        self.texture.clone()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    /// Number of texels available.
    pub fn capacity(&self) -> usize {
        self.capacity
    }


    /// Ensures that the texture can store at least
    /// `required_capacity` texels.
    ///
    /// Capacity grows exponentially.
    pub fn ensure_capacity(
        &mut self,
        required_capacity: usize,
    ) {
        if required_capacity <= self.capacity {
            return;
        }

        let mut new_capacity = self.capacity.max(INITIAL_BUBBLE_CAPACITY);

        while new_capacity < required_capacity {
            new_capacity *= 2;
        }

        self.resize(new_capacity);
    }



    fn resize(
        &mut self,
        new_capacity: usize,
    ) {
        let width = new_capacity as u32;

        self.texture =
            create_empty_float_texture(
                width,
                1,
            );

        self.texture.set_filter(FilterMode::Nearest);
        self.width = width;
        self.capacity = new_capacity;
    }


    /// Uploads RGBA float data.
    ///
    /// Data layout:
    ///
    /// texel0: r g b a
    /// texel1: r g b a
    /// ...
    ///
    pub fn upload(
        &self,
        data: &[f32],
    ) {

        assert!(
            data.len() % 4 == 0,
            "GPU texture data must be RGBA"
        );

let mut copy = data.to_vec();

copy[3] = 1.0;


        // Full texture upload requires the complete texture size.
        let mut packed =
            vec![
                0u16;
                self.capacity * 4
            ];

for (i, value) in copy.iter().enumerate() {
    packed[i] = f16::from_f32(*value).to_bits();
}


        for (i, value) in data.iter().enumerate() {

            packed[i] =
                f16::from_f32(*value)
                    .to_bits();
        }


        let mut bytes =
            Vec::<u8>::with_capacity(
                packed.len()*2
            );

        for value in packed {
            bytes.extend_from_slice(
                &value.to_le_bytes()
            );
        }

        self.texture.update_from_bytes(
            self.width,
            1,
            &bytes,
        );
    }

    pub fn upload_test(&self)
    {
        let mut data = vec![0u16; self.capacity * 4];

        data[0] = half::f16::from_f32(1.0).to_bits(); // R
        data[3] = half::f16::from_f32(1.0).to_bits(); // A

        let mut bytes = Vec::new();

        for x in data {
            bytes.extend_from_slice(
                &x.to_le_bytes()
            );
        }

        self.texture.update_from_bytes(
            self.width,
            1,
            &bytes,
        );
    }
}



/// Creates an empty RGBA16F texture.
///
/// This allocates GPU memory without uploading data.
fn create_empty_float_texture(
    width: u32,
    height: u32,
) -> Texture2D {

    let params = TextureParams {
        width,
        height,
        format: TextureFormat::RGBA16F,
        ..Default::default()
    };

    let ctx =
        unsafe {
            get_internal_gl()
        };

    let texture_id =
        ctx.quad_context.new_texture(
            TextureAccess::Static,
            TextureSource::Empty,
            params,
        );

    Texture2D::from_miniquad_texture(texture_id)
}