use macroquad::prelude::*;

use crate::world::ClientWorld;

/// GPU-independent representation of a bubble.
#[derive(Debug, Clone)]
pub struct RenderBubble {
    pub id: u64,

    pub position: Vec2,

    pub radius: f32,

    pub light: f32,

    pub color: Color,
}

/// GPU-independent representation of a player.
#[derive(Debug, Clone)]
pub struct RenderPlayer {
    pub id: u64,

    pub position: Vec2,

    pub rotation: f32,
}

/// GPU-independent representation of a creature.
#[derive(Debug, Clone)]
pub struct RenderCreature {
    pub id: u64,

    pub position: Vec2,

    pub rotation: f32,

    pub color: Color,
}

/// Everything that the renderer needs this frame.
///
/// No gameplay information should appear here.
#[derive(Debug, Default)]
pub struct RenderWorld {
    pub bubbles: Vec<RenderBubble>,

    pub players: Vec<RenderPlayer>,

    pub creatures: Vec<RenderCreature>,
}

impl RenderWorld {
    pub fn clear(&mut self) {
        self.bubbles.clear();
        self.players.clear();
        self.creatures.clear();
    }
}

/// Converts gameplay state into render state.
///
/// This should contain **no rendering logic**.
/// It merely copies the data needed by the renderer.
pub fn extract_render_world(world: &ClientWorld) -> RenderWorld {
    let mut render = RenderWorld::default();

    // ---------------------------------------------------------------------
    // Bubbles
    // ---------------------------------------------------------------------

    render.bubbles.reserve(world.graph.nodes.len());

    for bubble in world.graph.nodes.values() {
        render.bubbles.push(RenderBubble {
            id: bubble.id,

            position: vec2(bubble.x, bubble.y),

            radius: bubble.radius,

            light: bubble.light,

            color: Color::new(
                bubble.color[0],
                bubble.color[1],
                bubble.color[2],
                1.0,
            ),
        });
    }

    // ---------------------------------------------------------------------
    // Players
    // ---------------------------------------------------------------------

    render.players.reserve(world.players.len());

    for player in world.players.values() {
        render.players.push(RenderPlayer {
            id: player.id,

            position: vec2(player.x, player.y),

            rotation: player.rotation,
        });
    }

    // ---------------------------------------------------------------------
    // Creatures
    // ---------------------------------------------------------------------

    render.creatures.reserve(world.creatures.len());

    for creature in world.creatures.values() {

        let rotation = creature.vy.atan2(creature.vx);

        render.creatures.push(RenderCreature {
            id: creature.id,

            position: vec2(creature.x, creature.y),

            rotation,

            color: Color::new(
                creature.color_bias[0],
                creature.color_bias[1],
                creature.color_bias[2],
                1.0,
            ),
        });
    }

    render
}