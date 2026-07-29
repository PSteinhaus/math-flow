use std::collections::HashMap;

use shared::{
    BubbleData, BubbleEdge, BubbleGraph, CreatureData, PlayerData, SeedData,
};

/// Local copy of the game world.
///
/// The server is authoritative.
/// This world is updated from snapshots and interpolated locally.
/// Rendering never accesses this directly.
#[derive(Debug, Default)]
pub struct ClientWorld {
    pub graph: BubbleGraph,

    pub players: HashMap<u64, PlayerData>,

    pub creatures: HashMap<u64, CreatureData>,

    pub seeds: HashMap<u64, SeedData>,
}

impl ClientWorld {
    pub fn new() -> Self {
        Self::default()
    }

    // ---------------------------------------------------------------------
    // Full state replacement
    // ---------------------------------------------------------------------

    pub fn load_world(
        &mut self,
        graph: BubbleGraph,
        players: Vec<PlayerData>,
        seeds: Vec<SeedData>,
        creatures: Vec<CreatureData>,
    ) {
        self.graph = graph;

        self.players = players
            .into_iter()
            .map(|p| (p.id, p))
            .collect();

        self.seeds = seeds
            .into_iter()
            .map(|s| (s.id, s))
            .collect();

        self.creatures = creatures
            .into_iter()
            .map(|c| (c.id, c))
            .collect();
    }

    // ---------------------------------------------------------------------
    // Bubble graph
    // ---------------------------------------------------------------------

    pub fn update_bubble(&mut self, bubble: BubbleData) {
        self.graph.nodes.insert(bubble.id, bubble);
    }

    pub fn remove_bubble(&mut self, id: u64) {
        self.graph.nodes.remove(&id);

        self.graph.edges.retain(|_, edge| {
            edge.node_a != id && edge.node_b != id
        });
    }

    pub fn update_edge(&mut self, edge: BubbleEdge) {
        self.graph.edges.insert(edge.id, edge);
    }

    pub fn remove_edge(&mut self, id: u64) {
        self.graph.edges.remove(&id);
    }

    // ---------------------------------------------------------------------
    // Players
    // ---------------------------------------------------------------------

    pub fn update_player(&mut self, player: PlayerData) {
        self.players.insert(player.id, player);
    }

    pub fn remove_player(&mut self, id: u64) {
        self.players.remove(&id);
    }

    // ---------------------------------------------------------------------
    // Creatures
    // ---------------------------------------------------------------------

    pub fn update_creature(&mut self, creature: CreatureData) {
        self.creatures.insert(creature.id, creature);
    }

    pub fn remove_creature(&mut self, id: u64) {
        self.creatures.remove(&id);
    }

    // ---------------------------------------------------------------------
    // Seeds
    // ---------------------------------------------------------------------

    pub fn update_seed(&mut self, seed: SeedData) {
        self.seeds.insert(seed.id, seed);
    }

    pub fn remove_seed(&mut self, id: u64) {
        self.seeds.remove(&id);
    }

    // ---------------------------------------------------------------------
    // Queries
    // ---------------------------------------------------------------------

    pub fn bubble(&self, id: u64) -> Option<&BubbleData> {
        self.graph.nodes.get(&id)
    }

    pub fn player(&self, id: u64) -> Option<&PlayerData> {
        self.players.get(&id)
    }

    pub fn creature(&self, id: u64) -> Option<&CreatureData> {
        self.creatures.get(&id)
    }

    pub fn seed(&self, id: u64) -> Option<&SeedData> {
        self.seeds.get(&id)
    }
}