use super::types::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Clone)]
pub enum ServerBoundPacket {
    Handshake {
        protocol_version: i32,
        server_address: String,
        server_port: u16,
        next_state: i32,
    },
    StatusRequest,
    StatusPing {
        payload: i64,
    },
    LoginStart {
        name: String,
        uuid: Option<uuid::Uuid>,
    },
    LoginEncryptionResponse {
        shared_secret: Vec<u8>,
        verify_token: Vec<u8>,
    },
    LoginPluginResponse {
        message_id: i32,
        data: Option<Bytes>,
    },
    KeepAlive {
        id: i64,
    },
    ChatMessage {
        message: String,
    },
    PlayerPosition {
        x: f64,
        y: f64,
        z: f64,
        on_ground: bool,
    },
    PlayerRotation {
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
    PlayerPositionAndRotation {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
    PlayerDigging {
        status: i32,
        location: (i32, i32, i32),
        face: i8,
    },
    PlayerBlockPlacement {
        location: (i32, i32, i32),
        face: i32,
        hand: i32,
        cursor_x: f32,
        cursor_y: f32,
        cursor_z: f32,
        inside_block: bool,
    },
    PluginMessage {
        channel: String,
        data: Bytes,
    },
}

#[derive(Debug, Clone)]
pub enum ClientBoundPacket {
    StatusResponse {
        json: String,
    },
    StatusPong {
        payload: i64,
    },
    LoginDisconnect {
        reason: String,
    },
    LoginEncryptionRequest {
        server_id: String,
        public_key: Vec<u8>,
        verify_token: Vec<u8>,
    },
    LoginSuccess {
        uuid: uuid::Uuid,
        username: String,
    },
    LoginSetCompression {
        threshold: i32,
    },
    LoginPluginRequest {
        message_id: i32,
        channel: String,
        data: Bytes,
    },
    PlayDisconnect {
        reason: String,
    },
    KeepAlive {
        id: i64,
    },
    JoinGame {
        entity_id: i32,
        is_hardcore: bool,
        gamemode: u8,
        previous_gamemode: i8,
        dimension_names: Vec<String>,
        registry_codec: nbt::Blob,
        dimension_type: String,
        dimension_name: String,
        hashed_seed: i64,
        max_players: i32,
        view_distance: i32,
        simulation_distance: i32,
        reduced_debug_info: bool,
        enable_respawn_screen: bool,
        is_debug: bool,
        is_flat: bool,
    },
    ChunkData {
        x: i32,
        z: i32,
        data: Bytes,
    },
    SpawnPlayer {
        entity_id: i32,
        uuid: uuid::Uuid,
        x: f64,
        y: f64,
        z: f64,
        yaw: u8,
        pitch: u8,
    },
    EntityPosition {
        entity_id: i32,
        delta_x: i16,
        delta_y: i16,
        delta_z: i16,
        on_ground: bool,
    },
    EntityRotation {
        entity_id: i32,
        yaw: u8,
        pitch: u8,
        on_ground: bool,
    },
    EntityPositionAndRotation {
        entity_id: i32,
        delta_x: i16,
        delta_y: i16,
        delta_z: i16,
        yaw: u8,
        pitch: u8,
        on_ground: bool,
    },
    PlayerPositionAndLook {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        flags: u8,
        teleport_id: i32,
        dismount_vehicle: bool,
    },
    PluginMessage {
        channel: String,
        data: Bytes,
    },
    ChatMessage {
        message: String,
        position: i8,
        sender: uuid::Uuid,
    },
}

impl ServerBoundPacket {
    pub fn decode(state: ConnectionState, id: i32, mut data: Bytes) -> ProtocolResult<Self> {
        let mut cursor = Cursor::new(data.clone());

        match state {
            ConnectionState::Handshaking => match id {
                0x00 => {
                    let protocol_version = cursor.read_varint()?;
                    let server_address = cursor.read_string(255)?;
                    let server_port = cursor.get_u16();
                    let next_state = cursor.read_varint()?;

                    Ok(Self::Handshake {
                        protocol_version,
                        server_address,
                        server_port,
                        next_state,
                    })
                }
                _ => Err(ProtocolError::InvalidPacket),
            },
            ConnectionState::Status => match id {
                0x00 => Ok(Self::StatusRequest),
                0x01 => {
                    let payload = cursor.get_i64();
                    Ok(Self::StatusPing { payload })
                }
                _ => Err(ProtocolError::InvalidPacket),
            },
            ConnectionState::Login => match id {
                0x00 => {
                    let name = cursor.read_string(16)?;
                    let has_uuid = cursor.remaining() >= 16;
                    let uuid = if has_uuid {
                        Some(cursor.read_uuid()?)
                    } else {
                        None
                    };

                    Ok(Self::LoginStart { name, uuid })
                }
                0x01 => {
                    let secret_len = cursor.read_varint()? as usize;
                    let mut shared_secret = vec![0u8; secret_len];
                    cursor.copy_to_slice(&mut shared_secret);

                    let token_len = cursor.read_varint()? as usize;
                    let mut verify_token = vec![0u8; token_len];
                    cursor.copy_to_slice(&mut verify_token);

                    Ok(Self::LoginEncryptionResponse {
                        shared_secret,
                        verify_token,
                    })
                }
                0x02 => {
                    let message_id = cursor.read_varint()?;
                    let successful = cursor.get_u8() != 0;
                    let data = if successful {
                        let remaining = cursor.remaining();
                        let mut bytes = vec![0u8; remaining];
                        cursor.copy_to_slice(&mut bytes);
                        Some(Bytes::from(bytes))
                    } else {
                        None
                    };

                    Ok(Self::LoginPluginResponse { message_id, data })
                }
                _ => Err(ProtocolError::InvalidPacket),
            },
            ConnectionState::Play => match id {
                0x0F => {
                    let id = cursor.get_i64();
                    Ok(Self::KeepAlive { id })
                }
                0x05 => {
                    let message = cursor.read_string(256)?;
                    Ok(Self::ChatMessage { message })
                }
                0x14 => {
                    let x = cursor.get_f64();
                    let y = cursor.get_f64();
                    let z = cursor.get_f64();
                    let on_ground = cursor.get_u8() != 0;

                    Ok(Self::PlayerPosition { x, y, z, on_ground })
                }
                0x15 => {
                    let yaw = cursor.get_f32();
                    let pitch = cursor.get_f32();
                    let on_ground = cursor.get_u8() != 0;

                    Ok(Self::PlayerRotation { yaw, pitch, on_ground })
                }
                0x16 => {
                    let x = cursor.get_f64();
                    let y = cursor.get_f64();
                    let z = cursor.get_f64();
                    let yaw = cursor.get_f32();
                    let pitch = cursor.get_f32();
                    let on_ground = cursor.get_u8() != 0;

                    Ok(Self::PlayerPositionAndRotation {
                        x, y, z, yaw, pitch, on_ground,
                    })
                }
                0x1A => {
                    let status = cursor.read_varint()?;
                    let location = cursor.read_position()?;
                    let face = cursor.get_i8();

                    Ok(Self::PlayerDigging { status, location, face })
                }
                0x2E => {
                    let hand = cursor.read_varint()?;
                    let location = cursor.read_position()?;
                    let face = cursor.read_varint()?;
                    let cursor_x = cursor.get_f32();
                    let cursor_y = cursor.get_f32();
                    let cursor_z = cursor.get_f32();
                    let inside_block = cursor.get_u8() != 0;

                    Ok(Self::PlayerBlockPlacement {
                        location,
                        face,
                        hand,
                        cursor_x,
                        cursor_y,
                        cursor_z,
                        inside_block,
                    })
                }
                0x0A => {
                    let channel = cursor.read_string(256)?;
                    let remaining = cursor.remaining();
                    let mut bytes = vec![0u8; remaining];
                    cursor.copy_to_slice(&mut bytes);

                    Ok(Self::PluginMessage {
                        channel,
                        data: Bytes::from(bytes),
                    })
                }
                _ => Err(ProtocolError::InvalidPacket),
            },
        }
    }
}

impl ClientBoundPacket {
    pub fn encode(&self) -> ProtocolResult<PacketFrame> {
        let mut buf = BytesMut::new();

        let id = match self {
            Self::StatusResponse { json } => {
                buf.write_string(json);
                0x00
            }
            Self::StatusPong { payload } => {
                buf.put_i64(*payload);
                0x01
            }
            Self::LoginDisconnect { reason } => {
                buf.write_string(reason);
                0x00
            }
            Self::LoginEncryptionRequest {
                server_id,
                public_key,
                verify_token,
            } => {
                buf.write_string(server_id);
                buf.write_varint(public_key.len() as i32);
                buf.put_slice(public_key);
                buf.write_varint(verify_token.len() as i32);
                buf.put_slice(verify_token);
                0x01
            }
            Self::LoginSuccess { uuid, username } => {
                buf.write_uuid(uuid);
                buf.write_string(username);
                buf.write_varint(0);
                0x02
            }
            Self::LoginSetCompression { threshold } => {
                buf.write_varint(*threshold);
                0x03
            }
            Self::LoginPluginRequest {
                message_id,
                channel,
                data,
            } => {
                buf.write_varint(*message_id);
                buf.write_string(channel);
                buf.put_slice(data);
                0x04
            }
            Self::PlayDisconnect { reason } => {
                buf.write_string(reason);
                0x1A
            }
            Self::KeepAlive { id } => {
                buf.put_i64(*id);
                0x21
            }
            Self::JoinGame { .. } => {
                0x26
            }
            Self::ChunkData { x, z, data } => {
                buf.put_i32(*x);
                buf.put_i32(*z);
                buf.put_slice(data);
                0x22
            }
            Self::SpawnPlayer {
                entity_id,
                uuid,
                x,
                y,
                z,
                yaw,
                pitch,
            } => {
                buf.write_varint(*entity_id);
                buf.write_uuid(uuid);
                buf.put_f64(*x);
                buf.put_f64(*y);
                buf.put_f64(*z);
                buf.put_u8(*yaw);
                buf.put_u8(*pitch);
                0x02
            }
            Self::EntityPosition {
                entity_id,
                delta_x,
                delta_y,
                delta_z,
                on_ground,
            } => {
                buf.write_varint(*entity_id);
                buf.put_i16(*delta_x);
                buf.put_i16(*delta_y);
                buf.put_i16(*delta_z);
                buf.put_u8(if *on_ground { 1 } else { 0 });
                0x29
            }
            Self::EntityRotation {
                entity_id,
                yaw,
                pitch,
                on_ground,
            } => {
                buf.write_varint(*entity_id);
                buf.put_u8(*yaw);
                buf.put_u8(*pitch);
                buf.put_u8(if *on_ground { 1 } else { 0 });
                0x2B
            }
            Self::EntityPositionAndRotation {
                entity_id,
                delta_x,
                delta_y,
                delta_z,
                yaw,
                pitch,
                on_ground,
            } => {
                buf.write_varint(*entity_id);
                buf.put_i16(*delta_x);
                buf.put_i16(*delta_y);
                buf.put_i16(*delta_z);
                buf.put_u8(*yaw);
                buf.put_u8(*pitch);
                buf.put_u8(if *on_ground { 1 } else { 0 });
                0x2A
            }
            Self::PlayerPositionAndLook {
                x,
                y,
                z,
                yaw,
                pitch,
                flags,
                teleport_id,
                dismount_vehicle,
            } => {
                buf.put_f64(*x);
                buf.put_f64(*y);
                buf.put_f64(*z);
                buf.put_f32(*yaw);
                buf.put_f32(*pitch);
                buf.put_u8(*flags);
                buf.write_varint(*teleport_id);
                buf.put_u8(if *dismount_vehicle { 1 } else { 0 });
                0x38
            }
            Self::PluginMessage { channel, data } => {
                buf.write_string(channel);
                buf.put_slice(data);
                0x18
            }
            Self::ChatMessage {
                message,
                position,
                sender,
            } => {
                buf.write_string(message);
                buf.put_i8(*position);
                buf.write_uuid(sender);
                0x0F
            }
        };

        Ok(PacketFrame::new(id, buf.freeze()))
    }
}

mod nbt {
    use bytes::Bytes;

    #[derive(Debug, Clone)]
    pub struct Blob;
}
