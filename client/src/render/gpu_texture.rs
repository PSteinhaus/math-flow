use macroquad::prelude::*;
use macroquad::miniquad::{
    TextureAccess, TextureFormat, TextureId, TextureParams, TextureSource, raw_gl::*,
};

use crate::render::gpu_types::INITIAL_BUBBLE_CAPACITY;

const GL_HALF_FLOAT: u32 = 0x140B;

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
    gl_id: GLuint,
    width: u32,
    capacity: usize,
}


impl FloatTexture {
    /// Creates a new empty float texture.
    ///
    /// The texture is initially sized to the requested capacity.
    pub fn new(capacity: usize) -> Self {
        let width = capacity.max(1) as u32;
        let (texture, gl_id) =
            create_empty_float_texture(width, 1);

        texture.set_filter(FilterMode::Nearest);

        Self {
            texture,
            gl_id,
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

        (self.texture, self.gl_id) =
            create_empty_float_texture(
                width,
                1,
            );

        self.texture.set_filter(FilterMode::Nearest);
        self.width = width;
        self.capacity = new_capacity;
    }


    pub fn upload_gl(
        &self,
        data: &[f32],
    ) {
        assert!(
            data.len() % 4 == 0
        );

        let mut bytes =
            Vec::<u8>::with_capacity(
                data.len() * 2
            );

        for value in data {
            let half =
                half::f16::from_f32(*value);

            bytes.extend_from_slice(
                &half.to_bits().to_le_bytes()
            );
        }

        let texture_id =
            self.texture.raw_miniquad_id();

        let gl_texture = extract_gl_texture_id(texture_id);

        unsafe {
            upload_rgba16f(
                gl_texture,
                self.width,
                1,
                &bytes,
            );
        }
    }

    // pub fn upload_test(&self)
    // {
    //     let mut data = vec![0u16; self.capacity * 4];

    //     data[0] = half::f16::from_f32(1.0).to_bits(); // R
    //     data[3] = half::f16::from_f32(1.0).to_bits(); // A

    //     let mut bytes = Vec::new();

    //     for x in data {
    //         bytes.extend_from_slice(
    //             &x.to_le_bytes()
    //         );
    //     }

    //     self.texture.update_from_bytes(
    //         self.width,
    //         1,
    //         &bytes,
    //     );
    // }
}



/// Creates an empty RGBA16F texture.
///
/// This allocates GPU memory without uploading data.
fn create_empty_float_texture(
    width: u32,
    height: u32,
) -> (Texture2D, GLuint) {

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
    
    let gl_id = extract_gl_texture_id(texture_id);
    let texture_2d = Texture2D::from_miniquad_texture(texture_id);
    (texture_2d, gl_id)
}

unsafe fn upload_rgba16f(
    texture: GLuint,
    width: u32,
    height: u32,
    bytes: &[u8],
) {
    unsafe {
        glBindTexture(
            GL_TEXTURE_2D,
            texture
        );

        glTexSubImage2D(
            GL_TEXTURE_2D,
            0,
            0,
            0,
            width as i32,
            height as i32,
            GL_RGBA,
            GL_HALF_FLOAT,
            bytes.as_ptr() as *const _
        );

        glBindTexture(
            GL_TEXTURE_2D,
            0
        );
    }
}

fn extract_gl_texture_id(texture_id: TextureId) -> GLuint
{
    let ctx = unsafe { get_internal_gl() };

    let raw = unsafe {
        ctx.quad_context.texture_raw_id(texture_id)
    };

    let gl_texture = match raw {
        miniquad::RawId::OpenGl(id) => id,
        _ => panic!("RGBA16F upload requires OpenGL"),
    };
    assert_ne!(gl_texture, 0);

    gl_texture
}