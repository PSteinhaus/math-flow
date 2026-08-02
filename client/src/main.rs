use macroquad::prelude::*;

use shared::BubbleData;

mod render;
mod world;

use render::{
    gpu_buffers::BubbleBuffers,
    gpu_scene::GpuScene,
    gpu_types::GpuBubble,
};

const VERTEX_SHADER: &str = r#"#version 300 es
precision highp float;

in vec3 position;
in vec2 texcoord;

out vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position =
        Projection *
        Model *
        vec4(position, 1.0);

    uv = texcoord;
}
"#;

const FRAGMENT_SHADER: &str =
    include_str!("render/shaders/shader.glsl");

#[macroquad::main("Microscopic Bubble World")]
async fn main() {

    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },

        MaterialParams {
            uniforms: vec![
                UniformDesc::new(
                    "u_resolution",
                    UniformType::Float2
                ),
                UniformDesc::new(
                    "u_camera_pos",
                    UniformType::Float2
                ),
                UniformDesc::new(
                    "u_zoom",
                    UniformType::Float1
                ),
                UniformDesc::new(
                    "u_bubble_count",
                    UniformType::Int1
                ),
                UniformDesc::new(
                    "u_texture_width",
                    UniformType::Float1
                ),
            ],
            textures: vec![
                "u_position_radius_texture".to_string(),
                "u_color_pressure_texture".to_string(),
            ],

            ..Default::default()
        },
    )
    .unwrap();

    // -----------------------------------------------------
    // TEST WORLD
    // -----------------------------------------------------

    let mut bubbles: Vec<BubbleData> = (1..=100).map(|i| {
        if i == 100 {
            // Last bubble follows the mouse cursor (green)
            BubbleData::new(
                i as u64,
                0.0,
                0.0,
                90.0,
                0.7,
                [0.0, 1.0, 0.0],
            )
        } else {
            // Random bubbles
            let x = rand::gen_range(-1500.0, 1500.0);
            let y = rand::gen_range(-1500.0, 1500.0);
            let radius = rand::gen_range(20.0, 190.0);
            let light = rand::gen_range(0.1, 1.0);
            let color = [
                rand::gen_range(0.0, 1.0),
                rand::gen_range(0.0, 1.0),
                rand::gen_range(0.0, 1.0),
            ];
            BubbleData::new(
                i as u64,
                x,
                y,
                radius,
                light,
                color,
            )
        }
    }).collect();

    let mut gpu_scene =
        GpuScene::new();

    let mut bubble_buffers =
        BubbleBuffers::new();

    let mut camera_pos =
        vec2(0.0,0.0);

    let zoom = 0.5;
    
    // Initialize the offscreen Render Target (Half scale)
    let render_scale = 0.5; // 0.5 = half resolution, 0.75 = 3/4 resolution
    
    let mut render_t = render_target(
        (screen_width() * render_scale) as u32,
        (screen_height() * render_scale) as u32,
    );
    
    // Linear filtering ensures the upscale is smooth, not blocky
    render_t.texture.set_filter(FilterMode::Linear);

    let mut last_screen_size = (screen_width(), screen_height());

    loop {

        // -------------------------------------------------
        // TEST INPUT
        // -------------------------------------------------

        let mouse_screen = vec2(mouse_position().0, -mouse_position().1);
        let screen_center = vec2(screen_width() * 0.5, -screen_height() * 0.5);
        // Mouse picking works purely on screen space & true zoom
        let mouse_world = (mouse_screen - screen_center) / zoom + camera_pos;

        bubbles[99].x = mouse_world.x;
        bubbles[99].y = mouse_world.y;

        if is_key_down(KeyCode::Left) {
            camera_pos.x -= 5.0;
        }
        if is_key_down(KeyCode::Right) {
            camera_pos.x += 5.0;
        }
        if is_key_down(KeyCode::Up) {
            camera_pos.y += 5.0;
        }
        if is_key_down(KeyCode::Down) {
            camera_pos.y -= 5.0;
        }

        // --- 1. HANDLE WINDOW RESIZING ---
        let current_screen_size = (screen_width(), screen_height());
        if current_screen_size != last_screen_size {
            render_t = render_target(
                (current_screen_size.0 * render_scale) as u32,
                (current_screen_size.1 * render_scale) as u32,
            );
            render_t.texture.set_filter(FilterMode::Linear);
            last_screen_size = current_screen_size;
        }

        let fbo_width = render_t.texture.width();
        let fbo_height = render_t.texture.height();

        // --- 2. SETUP OFFSCREEN CAMERA ---
        // A camera that perfectly maps 1:1 to the render target's pixels
        let mut offscreen_camera = Camera2D::from_display_rect(Rect::new(
            0.0,
            0.0,
            fbo_width,
            fbo_height,
        ));
        offscreen_camera.render_target = Some(render_t.clone());
        
        // --- 3. PREPARE OFFSCREEN BUFFER ---
        set_camera(&offscreen_camera);
        clear_background(BLACK);

        // -------------------------------------------------
        // CPU -> GPU EXTRACTION
        // -------------------------------------------------

        gpu_scene.clear();

        for bubble in &bubbles {
            gpu_scene.push_bubble(
                GpuBubble {
                    x: bubble.x,

                    y: bubble.y,

                    radius: bubble.radius,

                    light: bubble.light,

                    color: Color::new(
                        bubble.color[0],
                        bubble.color[1],
                        bubble.color[2],
                        1.0,
                    ),

                    pressure: bubble.pressure,
                }
            );
        }

        bubble_buffers.upload(
            &gpu_scene
        );

        // -------------------------------------------------
        // SHADER UNIFORMS
        // -------------------------------------------------

        // Feed the HALF resolution to the shader, not the screen resolution!
        material.set_uniform("u_resolution", vec2(fbo_width, fbo_height));
        material.set_uniform("u_camera_pos", camera_pos);
        // Scale zoom by render_scale so 1 FBO pixel spans the correct world distance
        material.set_uniform("u_zoom", zoom * render_scale);

        material.set_uniform(
            "u_bubble_count",
            bubble_buffers.bubble_count()
                as i32
        );
        material.set_uniform(
            "u_texture_width",
            bubble_buffers.texture_width() as f32
        );

        material.set_texture(
            "u_position_radius_texture",
            bubble_buffers.position_radius_texture()
        );

        material.set_texture(
            "u_color_pressure_texture",
            bubble_buffers.color_pressure_texture()
        );

        // -------------------------------------------------
        // RENDER
        // -------------------------------------------------

        clear_background(BLACK);

        gl_use_material(
            &material
        );
        // Draw a full-frame rectangle to trigger the fragment shader for every pixel
        draw_rectangle(0.0, 0.0, fbo_width, fbo_height, WHITE);
        gl_use_default_material();

        // --- 4. DRAW UPSCALED RESULT TO MAIN SCREEN ---
        set_default_camera();
        clear_background(BLACK);

        // Draw the offscreen texture stretched across the entire physical window
        draw_texture_ex(
            &render_t.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                // RenderTargets in OpenGL are upside down. Macroquad requires flipping 
                // it vertically when drawing it back to the screen.
                flip_y: true, 
                ..Default::default()
            },
        );

        draw_text(
            "Move mouse/touch to move the green bubble",
            20.0,
            30.0,
            20.0,
            WHITE,
        );

        next_frame().await;
    }
}