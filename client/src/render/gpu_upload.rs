use super::{
    extraction::RenderWorld,
    gpu_scene::GpuScene,
    gpu_types::GpuBubble,
};

/// Converts the renderer-independent RenderWorld into
/// GPU upload buffers.
///
/// This function contains **no rendering code**.
/// It merely packs CPU-side data into GPU-friendly layouts.
pub fn build_gpu_scene(
    render_world: &RenderWorld,
    gpu_scene: &mut GpuScene,
) {

    gpu_scene.clear();

    gpu_scene.reserve(render_world.bubbles.len());

    for bubble in &render_world.bubbles {

        gpu_scene.push_bubble(GpuBubble {

            x: bubble.position.x,

            y: bubble.position.y,

            radius: bubble.radius,

            light: bubble.light,

            color: bubble.color,

            // pressure will later come from the
            // BubbleAppearance / physics system.
            pressure: 1.0,
        });
    }
}