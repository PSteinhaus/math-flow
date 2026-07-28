use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bubble {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

pub fn get_version() -> &'static str {
    "0.1.0"
}