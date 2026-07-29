pub mod extraction;
pub mod gpu_buffers;
pub mod gpu_scene;
pub mod gpu_texture;
pub mod gpu_types;
pub mod gpu_upload;

pub struct RendererLimits {

    pub max_visible_bubbles: usize,

    pub max_creatures: usize,

    pub max_particles: usize,
}