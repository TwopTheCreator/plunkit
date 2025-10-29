use super::{chunk::*, entity::*};
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::Result;

pub struct World {
    pub id: String,
    pub name: String,
    pub ecs: Arc<RwLock<EcsWorld>>,
    pub chunks: Arc<RwLock<HashMap<(i32, i32), Chunk>>>,
    pub entity_counter: Arc<RwLock<i32>>,
}

pub struct EcsWorld {
    pub world: bevy_ecs::world::World,
}

impl EcsWorld {
    pub fn new() -> Self {
        Self {
            world: bevy_ecs::world::World::new(),
        }
    }
}

impl World {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            ecs: Arc::new(RwLock::new(EcsWorld::new())),
            chunks: Arc::new(RwLock::new(HashMap::new())),
            entity_counter: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn get_chunk(&self, x: i32, z: i32) -> Option<Chunk> {
        let chunks = self.chunks.read().await;
        chunks.get(&(x, z)).cloned()
    }

    pub async fn set_chunk(&self, chunk: Chunk) {
        let mut chunks = self.chunks.write().await;
        chunks.insert((chunk.position.x, chunk.position.z), chunk);
    }

    pub async fn generate_chunk(&self, x: i32, z: i32) -> Chunk {
        let position = ChunkPosition::new(x, z);
        let mut chunk = Chunk::new(position);

        chunk.generate_flat(4);

        self.set_chunk(chunk.clone()).await;
        chunk
    }

    pub async fn get_or_generate_chunk(&self, x: i32, z: i32) -> Chunk {
        if let Some(chunk) = self.get_chunk(x, z).await {
            return chunk;
        }

        self.generate_chunk(x, z).await
    }

    pub async fn get_block(&self, x: i32, y: i32, z: i32) -> u16 {
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;

        if let Some(chunk) = self.get_chunk(chunk_x, chunk_z).await {
            let local_x = (x & 15) as usize;
            let local_z = (z & 15) as usize;

            if y >= 0 && y < CHUNK_HEIGHT as i32 {
                chunk.get_block(local_x, y as usize, local_z)
            } else {
                0
            }
        } else {
            0
        }
    }

    pub async fn set_block(&self, x: i32, y: i32, z: i32, block_id: u16) {
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;

        let mut chunks = self.chunks.write().await;

        if let Some(chunk) = chunks.get_mut(&(chunk_x, chunk_z)) {
            let local_x = (x & 15) as usize;
            let local_z = (z & 15) as usize;

            if y >= 0 && y < CHUNK_HEIGHT as i32 {
                chunk.set_block(local_x, y as usize, local_z, block_id);
            }
        } else {
            drop(chunks);
            let mut chunk = self.generate_chunk(chunk_x, chunk_z).await;

            let local_x = (x & 15) as usize;
            let local_z = (z & 15) as usize;

            if y >= 0 && y < CHUNK_HEIGHT as i32 {
                chunk.set_block(local_x, y as usize, local_z, block_id);
            }

            self.set_chunk(chunk).await;
        }
    }

    pub async fn spawn_player(&self, username: String, uuid: Uuid) -> Entity {
        let mut counter = self.entity_counter.write().await;
        let entity_id = *counter;
        *counter += 1;

        let mut ecs = self.ecs.write().await;

        ecs.world.spawn((
            EntityId::new(entity_id),
            EntityUuid::from_uuid(uuid),
            EntityType::player(),
            Player::new(username),
            Position::new(0.0, 64.0, 0.0),
            Rotation::new(0.0, 0.0),
            Velocity::zero(),
            OnGround::new(false),
            Health::new(20.0),
        ))
    }

    pub async fn get_player_entity(&self, uuid: &Uuid) -> Option<Entity> {
        let ecs = self.ecs.read().await;

        let mut query = ecs.world.query::<(Entity, &EntityUuid, &Player)>();

        for (entity, entity_uuid, _) in query.iter(&ecs.world) {
            if entity_uuid.uuid == *uuid {
                return Some(entity);
            }
        }

        None
    }

    pub async fn get_player_position(&self, entity: Entity) -> Option<Position> {
        let ecs = self.ecs.read().await;

        if let Ok(pos) = ecs.world.get::<Position>(entity) {
            Some(*pos)
        } else {
            None
        }
    }

    pub async fn set_player_position(&self, entity: Entity, position: Position) -> Result<()> {
        let mut ecs = self.ecs.write().await;

        if let Ok(mut pos) = ecs.world.get_mut::<Position>(entity) {
            *pos = position;
        }

        Ok(())
    }

    pub async fn update_player_rotation(&self, entity: Entity, rotation: Rotation) -> Result<()> {
        let mut ecs = self.ecs.write().await;

        if let Ok(mut rot) = ecs.world.get_mut::<Rotation>(entity) {
            *rot = rotation;
        }

        Ok(())
    }

    pub async fn get_nearby_chunks(&self, center_x: i32, center_z: i32, distance: i32) -> Vec<(i32, i32)> {
        let mut chunk_positions = Vec::new();

        for x in (center_x - distance)..=(center_x + distance) {
            for z in (center_z - distance)..=(center_z + distance) {
                chunk_positions.push((x, z));
            }
        }

        chunk_positions
    }

    pub async fn get_all_players(&self) -> Vec<(Entity, String, Position)> {
        let ecs = self.ecs.read().await;

        let mut query = ecs.world.query::<(Entity, &Player, &Position)>();

        query
            .iter(&ecs.world)
            .map(|(entity, player, position)| (entity, player.username.clone(), *position))
            .collect()
    }

    pub async fn tick(&self) -> Result<()> {
        Ok(())
    }
}

pub struct WorldManager {
    worlds: Arc<RwLock<HashMap<String, Arc<World>>>>,
}

impl WorldManager {
    pub fn new() -> Self {
        Self {
            worlds: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_world(&self, id: String, name: String) -> Arc<World> {
        let world = Arc::new(World::new(id.clone(), name));

        let mut worlds = self.worlds.write().await;
        worlds.insert(id, world.clone());

        world
    }

    pub async fn get_world(&self, id: &str) -> Option<Arc<World>> {
        let worlds = self.worlds.read().await;
        worlds.get(id).cloned()
    }

    pub async fn remove_world(&self, id: &str) {
        let mut worlds = self.worlds.write().await;
        worlds.remove(id);
    }

    pub async fn list_worlds(&self) -> Vec<String> {
        let worlds = self.worlds.read().await;
        worlds.keys().cloned().collect()
    }

    pub async fn tick_all(&self) -> Result<()> {
        let worlds = self.worlds.read().await;

        for world in worlds.values() {
            world.tick().await?;
        }

        Ok(())
    }
}

impl Default for WorldManager {
    fn default() -> Self {
        Self::new()
    }
}
