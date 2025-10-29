use crate::protocol::*;
use crate::world::{World, Position, Rotation};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct PlayerSession {
    pub uuid: Uuid,
    pub username: String,
    pub world_id: String,
    pub entity: bevy_ecs::entity::Entity,
}

pub struct MinecraftServer {
    pub world: Arc<World>,
    pub sessions: Arc<RwLock<HashMap<Uuid, PlayerSession>>>,
}

impl MinecraftServer {
    pub fn new(world: Arc<World>) -> Self {
        Self {
            world,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        tracing::info!("Minecraft server listening on port {}", port);

        loop {
            let (stream, addr) = listener.accept().await?;
            tracing::info!("New connection from {}", addr);

            let server = self.clone();
            tokio::spawn(async move {
                if let Err(e) = server.handle_connection(stream).await {
                    tracing::error!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        let mut conn = MinecraftConnection::new(stream);

        loop {
            match conn.read_packet().await? {
                Some(packet) => {
                    if !self.handle_packet(&mut conn, packet).await? {
                        break;
                    }
                }
                None => break,
            }
        }

        Ok(())
    }

    async fn handle_packet(
        &self,
        conn: &mut MinecraftConnection,
        packet: ServerBoundPacket,
    ) -> Result<bool> {
        match packet {
            ServerBoundPacket::Handshake {
                protocol_version,
                server_address,
                server_port,
                next_state,
            } => {
                tracing::info!(
                    "Handshake: proto={}, addr={}, port={}, next={}",
                    protocol_version,
                    server_address,
                    server_port,
                    next_state
                );

                if next_state == 1 {
                    conn.set_state(ConnectionState::Status);
                } else if next_state == 2 {
                    conn.set_state(ConnectionState::Login);
                }
            }
            ServerBoundPacket::StatusRequest => {
                let response = serde_json::json!({
                    "version": {
                        "name": "1.19.4",
                        "protocol": 762
                    },
                    "players": {
                        "max": 20,
                        "online": 0,
                        "sample": []
                    },
                    "description": {
                        "text": format!("Plunkit Server - {}", self.world.name)
                    }
                });

                conn.write_packet(ClientBoundPacket::StatusResponse {
                    json: response.to_string(),
                })
                .await?;
            }
            ServerBoundPacket::StatusPing { payload } => {
                conn.write_packet(ClientBoundPacket::StatusPong { payload })
                    .await?;
                return Ok(false);
            }
            ServerBoundPacket::LoginStart { name, uuid } => {
                tracing::info!("Login start: {}", name);

                let player_uuid = uuid.unwrap_or_else(|| Uuid::new_v4());

                conn.write_packet(ClientBoundPacket::LoginSuccess {
                    uuid: player_uuid,
                    username: name.clone(),
                })
                .await?;

                conn.set_state(ConnectionState::Play);

                let entity = self.world.spawn_player(name.clone(), player_uuid).await;

                let mut sessions = self.sessions.write().await;
                sessions.insert(
                    player_uuid,
                    PlayerSession {
                        uuid: player_uuid,
                        username: name,
                        world_id: self.world.id.clone(),
                        entity,
                    },
                );

                conn.write_packet(ClientBoundPacket::JoinGame {
                    entity_id: 1,
                    is_hardcore: false,
                    gamemode: 1,
                    previous_gamemode: -1,
                    dimension_names: vec!["minecraft:overworld".to_string()],
                    registry_codec: Default::default(),
                    dimension_type: "minecraft:overworld".to_string(),
                    dimension_name: "minecraft:overworld".to_string(),
                    hashed_seed: 0,
                    max_players: 20,
                    view_distance: 10,
                    simulation_distance: 10,
                    reduced_debug_info: false,
                    enable_respawn_screen: true,
                    is_debug: false,
                    is_flat: true,
                })
                .await?;

                conn.write_packet(ClientBoundPacket::PlayerPositionAndLook {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0,
                    yaw: 0.0,
                    pitch: 0.0,
                    flags: 0,
                    teleport_id: 0,
                    dismount_vehicle: false,
                })
                .await?;

                self.send_spawn_chunks(conn, 0, 0).await?;
            }
            ServerBoundPacket::KeepAlive { id } => {
                conn.write_packet(ClientBoundPacket::KeepAlive { id }).await?;
            }
            ServerBoundPacket::PlayerPosition { x, y, z, on_ground } => {
            }
            ServerBoundPacket::PlayerRotation { yaw, pitch, on_ground } => {
            }
            ServerBoundPacket::PlayerPositionAndRotation {
                x,
                y,
                z,
                yaw,
                pitch,
                on_ground,
            } => {
            }
            ServerBoundPacket::ChatMessage { message } => {
                tracing::info!("Chat: {}", message);
            }
            ServerBoundPacket::PlayerDigging { status, location, face } => {
                tracing::info!("Digging: {:?} at {:?}", status, location);
            }
            ServerBoundPacket::PlayerBlockPlacement { location, face, hand, .. } => {
                tracing::info!("Block placement at {:?}", location);
            }
            _ => {}
        }

        Ok(true)
    }

    async fn send_spawn_chunks(&self, conn: &mut MinecraftConnection, center_x: i32, center_z: i32) -> Result<()> {
        let view_distance = 2;

        for x in (center_x - view_distance)..=(center_x + view_distance) {
            for z in (center_z - view_distance)..=(center_z + view_distance) {
                let chunk = self.world.get_or_generate_chunk(x, z).await;
                let chunk_data = chunk.encode();

                conn.write_packet(ClientBoundPacket::ChunkData {
                    x,
                    z,
                    data: chunk_data,
                })
                .await?;
            }
        }

        Ok(())
    }

    fn clone(&self) -> Self {
        Self {
            world: self.world.clone(),
            sessions: self.sessions.clone(),
        }
    }
}
