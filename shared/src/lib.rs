use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =========================================================================
// 1. MATH OPERATIONS & TASKS
// =========================================================================

/// The 4 basic arithmetic operations, each linked to a distinct visual/topological mechanic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MathOp {
    /// Addition: Awakens seeds, grows new bubbles (Default: Warm White/Gold)
    Addition,
    /// Subtraction: Dissolves wall membranes to merge bubbles into tubes (Default: Soft Cyan/Blue)
    Subtraction,
    /// Multiplication: Fills bubbles with vibrant light (Default: Green)
    Multiplication,
    /// Division: Inserts dividing wall membranes to split bubble structures (Default: Crimson Red)
    Division,
}

impl MathOp {
    /// Associated light color normalized to [0.0..1.0] RGB
    pub fn default_color(&self) -> [f32; 3] {
        match self {
            MathOp::Addition => [0.95, 0.90, 0.80],      // Soft warm white/gold
            MathOp::Subtraction => [0.05, 0.10, 1.],   // Deep blue
            MathOp::Multiplication => [0., 1., 0.],      // Green
            MathOp::Division => [1.00, 0.01, 0.05],      // Red
        }
    }
}

/// A math problem sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathTask {
    pub id: u64,
    pub op: MathOp,
    pub question: String,
    pub options: Vec<i32>,
    pub correct_answer: i32,
}

// =========================================================================
// 2. BUBBLE TOPOLOGY & GRAPH ENGINE
// =========================================================================

/// Defines the physical/visual interface between two connected bubbles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MembraneType {
    /// Held together by surface tension (standard SDF smooth union)
    Normal,
    /// Membrane wall dissolved via Subtraction (amorphous tube)
    Open,
    /// Internal dividing wall inserted via Division
    Divided,
}

/// Represents a single bubble node in the world.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BubbleData {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    /// Internal expansion pressure driving spring physics
    pub pressure: f32,
    /// Light intensity inside the bubble (0.0 to 1.0+)
    pub light: f32,
    /// RGB color normalized [0.0..1.0]
    pub color: [f32; 3],
}

impl BubbleData {
    pub fn new(id: u64, x: f32, y: f32, radius: f32, light: f32, color: [f32; 3]) -> Self {
        Self {
            id,
            x,
            y,
            radius,
            pressure: 1.0,
            light,
            color,
        }
    }

    pub fn position(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    pub fn distance_to(&self, point: [f32; 2]) -> f32 {
        let dx = self.x - point[0];
        let dy = self.y - point[1];
        (dx * dx + dy * dy).sqrt()
    }

    pub fn is_point_inside(&self, point: [f32; 2]) -> bool {
        self.distance_to(point) <= self.radius
    }
}

/// An edge connecting two bubble nodes in the graph.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BubbleEdge {
    pub id: u64,
    pub node_a: u64,
    pub node_b: u64,
    pub membrane: MembraneType,
    /// Rest length used by the spring simulation
    pub rest_length: f32,
}

/// The entire bubble graph representing the safe lit sanctuary world.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BubbleGraph {
    pub nodes: HashMap<u64, BubbleData>,
    pub edges: HashMap<u64, BubbleEdge>,
    next_id: u64,
}

impl BubbleGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn generate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_node(&mut self, mut node: BubbleData) -> u64 {
        if node.id == 0 {
            node.id = self.generate_id();
        }
        let id = node.id;
        self.nodes.insert(id, node);
        id
    }

    pub fn add_edge(&mut self, node_a: u64, node_b: u64, membrane: MembraneType) -> Option<u64> {
        let edge_id = self.generate_id();
        let n1 = self.nodes.get(&node_a)?;
        let n2 = self.nodes.get(&node_b)?;

        let dx = n1.x - n2.x;
        let dy = n1.y - n2.y;
        let dist = (dx * dx + dy * dy).sqrt();

        let edge = BubbleEdge {
            id: edge_id,
            node_a,
            node_b,
            membrane,
            rest_length: dist.max(n1.radius + n2.radius),
        };
        self.edges.insert(edge_id, edge);
        Some(edge_id)
    }

    /// Queries whether a given position is inside a safe, illuminated bubble.
    pub fn is_in_safe_light(&self, point: [f32; 2], min_light_threshold: f32) -> bool {
        self.nodes
            .values()
            .any(|n| n.light >= min_light_threshold && n.is_point_inside(point))
    }
}

// =========================================================================
// 3. SEEDS, EGGS & ECOSYSTEM CREATURES
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedState {
    Dormant,
    CarriedByPlayer(u64), // Player ID
    Planted,
    Awakening,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedData {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub state: SeedState,
    pub required_op: Option<MathOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureData {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub segments: usize,
    /// Dynamic color blend derived from the light colors it was fed
    pub color_bias: [f32; 3],
    pub attached_to_player: Option<u64>,
    pub stamina: f32,
}

// =========================================================================
// 4. PLAYER STATE
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub id: u64,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub rotation: f32,
    /// Light energy acting as health when venturing into dark space
    pub current_light: f32,
    pub max_light: f32,
    pub carried_seed_id: Option<u64>,
    pub attached_creatures: Vec<u64>,
}

// =========================================================================
// 5. WEBSOCKET PROTOCOL MESSAGES
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { name: String },
    InputMove { x: f32, y: f32, vx: f32, vy: f32, rotation: f32 },
    RequestMathTask { op: MathOp },
    SubmitMathAnswer { task_id: u64, answer: i32 },
    PickUpSeed { seed_id: u64 },
    DropSeed { x: f32, y: f32 },
    AttachCreature { creature_id: u64 },
    DetachCreature { creature_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Full initial state download on joining
    Welcome {
        player_id: u64,
        graph: BubbleGraph,
        players: Vec<PlayerData>,
        seeds: Vec<SeedData>,
        creatures: Vec<CreatureData>,
    },
    /// High-frequency position & velocity update (20 Hz tick)
    WorldSnapshot {
        nodes: Vec<BubbleData>,
        players: Vec<PlayerData>,
        creatures: Vec<CreatureData>,
    },
    BubbleAdded(BubbleData),
    BubbleUpdated(BubbleData),
    EdgeAdded(BubbleEdge),
    EdgeUpdated(BubbleEdge),
    EdgeRemoved(u64),
    SeedStateChanged(SeedData),
    MathTaskAssigned(MathTask),
    LightPulse {
        origin_x: f32,
        origin_y: f32,
        color: [f32; 3],
        intensity: f32,
        op_type: MathOp,
    },
    PlayerJoined(PlayerData),
    PlayerLeft(u64),
}