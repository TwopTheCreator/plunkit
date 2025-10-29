use super::{packets::*, types::*};
use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MinecraftConnection {
    stream: Arc<Mutex<TcpStream>>,
    state: ConnectionState,
    read_buffer: BytesMut,
    compression_threshold: Option<i32>,
}

impl MinecraftConnection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
            state: ConnectionState::Handshaking,
            read_buffer: BytesMut::with_capacity(8192),
            compression_threshold: None,
        }
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }

    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }

    pub fn set_compression(&mut self, threshold: i32) {
        self.compression_threshold = Some(threshold);
    }

    pub async fn read_packet(&mut self) -> ProtocolResult<Option<ServerBoundPacket>> {
        loop {
            if let Some(frame) = PacketFrame::decode(&mut self.read_buffer)? {
                let packet = ServerBoundPacket::decode(self.state, frame.id, frame.data)?;
                return Ok(Some(packet));
            }

            let mut stream = self.stream.lock().await;
            let n = stream.read_buf(&mut self.read_buffer).await?;

            if n == 0 {
                if self.read_buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(ProtocolError::InvalidPacket);
                }
            }
        }
    }

    pub async fn write_packet(&mut self, packet: ClientBoundPacket) -> ProtocolResult<()> {
        let frame = packet.encode()?;
        let data = frame.encode();

        let mut stream = self.stream.lock().await;
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    pub async fn disconnect(&mut self, reason: &str) -> ProtocolResult<()> {
        let packet = match self.state {
            ConnectionState::Login => ClientBoundPacket::LoginDisconnect {
                reason: reason.to_string(),
            },
            ConnectionState::Play => ClientBoundPacket::PlayDisconnect {
                reason: reason.to_string(),
            },
            _ => return Ok(()),
        };

        self.write_packet(packet).await?;
        Ok(())
    }
}
