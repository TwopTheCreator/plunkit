use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Component, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn distance_to(&self, other: &Position) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn chunk_position(&self) -> (i32, i32) {
        ((self.x / 16.0).floor() as i32, (self.z / 16.0).floor() as i32)
    }
}

#[derive(Component, Clone, Copy, Serialize, Deserialize)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

impl Rotation {
    pub fn new(yaw: f32, pitch: f32) -> Self {
        Self { yaw, pitch }
    }
}

#[derive(Component, Clone, Copy, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Velocity {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct EntityId {
    pub id: i32,
}

impl EntityId {
    pub fn new(id: i32) -> Self {
        Self { id }
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct EntityUuid {
    pub uuid: Uuid,
}

impl EntityUuid {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self { uuid }
    }
}

impl Default for EntityUuid {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct EntityType {
    pub type_name: String,
}

impl EntityType {
    pub fn new(type_name: String) -> Self {
        Self { type_name }
    }

    pub fn player() -> Self {
        Self {
            type_name: "player".to_string(),
        }
    }

    pub fn item() -> Self {
        Self {
            type_name: "item".to_string(),
        }
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Player {
    pub username: String,
    pub gamemode: GameMode,
    pub health: f32,
    pub food: i32,
    pub level: i32,
    pub experience: f32,
}

impl Player {
    pub fn new(username: String) -> Self {
        Self {
            username,
            gamemode: GameMode::Survival,
            health: 20.0,
            food: 20,
            level: 0,
            experience: 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

#[derive(Component, Clone)]
pub struct OnGround {
    pub on_ground: bool,
}

impl OnGround {
    pub fn new(on_ground: bool) -> Self {
        Self { on_ground }
    }
}

#[derive(Component, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}
