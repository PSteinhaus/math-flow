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

    let mut bubbles = vec![
        BubbleData::new(
            1,
            0.0,
            0.0,
            120.0,
            0.4,
            [0.0, 0.0, 1.0],
        ),
        BubbleData::new(
            2,
            -160.0,
            40.0,
            70.0,
            0.7,
            [1.0, 0.0, 0.0],
        ),
        BubbleData::new(
            3,
            150.0,
            -80.0,
            90.0,
            0.7,
            [0.0, 1.0, 0.0],
        ),
    ];

    let mut gpu_scene =
        GpuScene::new();

    let mut bubble_buffers =
        BubbleBuffers::new();

    let mut camera_pos =
        vec2(0.0,0.0);

    let zoom =
        1.0;

    loop {

        // -------------------------------------------------
        // TEST INPUT
        // -------------------------------------------------

        let mouse_world = (vec2(mouse_position().0, -mouse_position().1)
                -               vec2(screen_width()*0.5, -screen_height()*0.5))
            / zoom + camera_pos;

        bubbles[2].x = mouse_world.x;
        bubbles[2].y = mouse_world.y;

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

        material.set_uniform(
            "u_resolution",
            (
                screen_width(),
                screen_height()
            )
        );
        material.set_uniform(
            "u_camera_pos",
            camera_pos
        );
        material.set_uniform(
            "u_zoom",
            zoom
        );
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
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            WHITE,
        );
        gl_use_default_material();

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