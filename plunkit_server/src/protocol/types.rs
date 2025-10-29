use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::{self, Cursor};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid packet")]
    InvalidPacket,
    #[error("Unsupported protocol version")]
    UnsupportedVersion,
    #[error("Invalid state transition")]
    InvalidState,
    #[error("String too long")]
    StringTooLong,
}

pub type ProtocolResult<T> = Result<T, ProtocolError>;

pub trait MCReadExt: Buf {
    fn read_varint(&mut self) -> ProtocolResult<i32> {
        let mut num_read = 0;
        let mut result = 0i32;

        loop {
            if !self.has_remaining() {
                return Err(ProtocolError::InvalidPacket);
            }

            let read = self.get_u8();
            let value = (read & 0b01111111) as i32;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return Err(ProtocolError::InvalidPacket);
            }

            if (read & 0b10000000) == 0 {
                break;
            }
        }

        Ok(result)
    }

    fn read_varlong(&mut self) -> ProtocolResult<i64> {
        let mut num_read = 0;
        let mut result = 0i64;

        loop {
            if !self.has_remaining() {
                return Err(ProtocolError::InvalidPacket);
            }

            let read = self.get_u8();
            let value = (read & 0b01111111) as i64;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 10 {
                return Err(ProtocolError::InvalidPacket);
            }

            if (read & 0b10000000) == 0 {
                break;
            }
        }

        Ok(result)
    }

    fn read_string(&mut self, max_len: usize) -> ProtocolResult<String> {
        let len = self.read_varint()? as usize;

        if len > max_len * 4 {
            return Err(ProtocolError::StringTooLong);
        }

        if self.remaining() < len {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut bytes = vec![0u8; len];
        self.copy_to_slice(&mut bytes);

        String::from_utf8(bytes).map_err(|_| ProtocolError::InvalidPacket)
    }

    fn read_uuid(&mut self) -> ProtocolResult<uuid::Uuid> {
        if self.remaining() < 16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut bytes = [0u8; 16];
        self.copy_to_slice(&mut bytes);

        Ok(uuid::Uuid::from_bytes(bytes))
    }

    fn read_position(&mut self) -> ProtocolResult<(i32, i32, i32)> {
        let val = self.get_i64();

        let x = (val >> 38) as i32;
        let y = (val << 52 >> 52) as i32;
        let z = (val << 26 >> 38) as i32;

        Ok((x, y, z))
    }
}

impl<T: Buf> MCReadExt for T {}

pub trait MCWriteExt: BufMut {
    fn write_varint(&mut self, mut value: i32) {
        loop {
            let mut temp = (value & 0b01111111) as u8;
            value >>= 7;

            if value != 0 {
                temp |= 0b10000000;
            }

            self.put_u8(temp);

            if value == 0 {
                break;
            }
        }
    }

    fn write_varlong(&mut self, mut value: i64) {
        loop {
            let mut temp = (value & 0b01111111) as u8;
            value >>= 7;

            if value != 0 {
                temp |= 0b10000000;
            }

            self.put_u8(temp);

            if value == 0 {
                break;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.put_slice(bytes);
    }

    fn write_uuid(&mut self, uuid: &uuid::Uuid) {
        self.put_slice(uuid.as_bytes());
    }

    fn write_position(&mut self, x: i32, y: i32, z: i32) {
        let val = ((x as i64 & 0x3FFFFFF) << 38)
                | ((z as i64 & 0x3FFFFFF) << 12)
                | (y as i64 & 0xFFF);
        self.put_i64(val);
    }
}

impl<T: BufMut> MCWriteExt for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Play,
}

#[derive(Debug, Clone)]
pub struct PacketFrame {
    pub id: i32,
    pub data: Bytes,
}

impl PacketFrame {
    pub fn new(id: i32, data: Bytes) -> Self {
        Self { id, data }
    }

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();

        let data_len = varint_len(self.id) + self.data.len();
        buf.write_varint(data_len as i32);
        buf.write_varint(self.id);
        buf.put_slice(&self.data);

        buf
    }

    pub fn decode(buf: &mut BytesMut) -> ProtocolResult<Option<Self>> {
        let mut cursor = Cursor::new(&buf[..]);

        let packet_len = match cursor.read_varint() {
            Ok(len) => len as usize,
            Err(_) => return Ok(None),
        };

        let header_len = cursor.position() as usize;
        let total_len = header_len + packet_len;

        if buf.len() < total_len {
            return Ok(None);
        }

        cursor.set_position(header_len as u64);
        let packet_id = cursor.read_varint()?;

        let data_start = cursor.position() as usize;
        let data_end = total_len;
        let data = buf.split_to(total_len).freeze().slice(data_start..data_end);

        Ok(Some(PacketFrame {
            id: packet_id,
            data,
        }))
    }
}

fn varint_len(mut value: i32) -> usize {
    let mut len = 0;
    loop {
        len += 1;
        value >>= 7;
        if value == 0 {
            break;
        }
    }
    len
}
