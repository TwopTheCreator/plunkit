use bevy_ecs::prelude::*;
use bytes::{BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 384;
pub const SECTION_HEIGHT: usize = 16;
pub const SECTIONS_PER_CHUNK: usize = CHUNK_HEIGHT / SECTION_HEIGHT;

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

impl ChunkPosition {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn from_block(x: i32, z: i32) -> Self {
        Self {
            x: x >> 4,
            z: z >> 4,
        }
    }
}

#[derive(Clone)]
pub struct ChunkSection {
    blocks: Vec<u16>,
    block_light: Vec<u8>,
    sky_light: Vec<u8>,
}

impl ChunkSection {
    pub fn new() -> Self {
        let block_count = CHUNK_WIDTH * SECTION_HEIGHT * CHUNK_WIDTH;
        let light_count = block_count / 2;

        Self {
            blocks: vec![0; block_count],
            block_light: vec![0; light_count],
            sky_light: vec![0xFF; light_count],
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> u16 {
        let index = (y * CHUNK_WIDTH * CHUNK_WIDTH) + (z * CHUNK_WIDTH) + x;
        self.blocks[index]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_id: u16) {
        let index = (y * CHUNK_WIDTH * CHUNK_WIDTH) + (z * CHUNK_WIDTH) + x;
        self.blocks[index] = block_id;
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|&b| b == 0)
    }

    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();

        buf.put_i16(self.blocks.iter().filter(|&&b| b != 0).count() as i16);

        for &block in &self.blocks {
            buf.put_u16(block);
        }

        buf.put_slice(&self.block_light);
        buf.put_slice(&self.sky_light);

        buf.freeze()
    }
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Clone)]
pub struct Chunk {
    pub position: ChunkPosition,
    pub sections: Vec<ChunkSection>,
    pub heightmap: Vec<i32>,
    pub biomes: Vec<i32>,
}

impl Chunk {
    pub fn new(position: ChunkPosition) -> Self {
        Self {
            position,
            sections: vec![ChunkSection::new(); SECTIONS_PER_CHUNK],
            heightmap: vec![0; CHUNK_WIDTH * CHUNK_WIDTH],
            biomes: vec![1; CHUNK_WIDTH * CHUNK_WIDTH],
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> u16 {
        let section_index = y / SECTION_HEIGHT;
        let section_y = y % SECTION_HEIGHT;

        if section_index >= self.sections.len() {
            return 0;
        }

        self.sections[section_index].get_block(x, section_y, z)
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_id: u16) {
        let section_index = y / SECTION_HEIGHT;
        let section_y = y % SECTION_HEIGHT;

        if section_index >= self.sections.len() {
            return;
        }

        self.sections[section_index].set_block(x, section_y, z, block_id);

        let heightmap_index = z * CHUNK_WIDTH + x;
        if block_id != 0 && y as i32 > self.heightmap[heightmap_index] {
            self.heightmap[heightmap_index] = y as i32;
        }
    }

    pub fn generate_flat(&mut self, height: usize) {
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                self.set_block(x, 0, z, 7);

                for y in 1..height.saturating_sub(1) {
                    self.set_block(x, y, z, 3);
                }

                if height > 1 {
                    self.set_block(x, height - 1, z, 2);
                }

                self.heightmap[z * CHUNK_WIDTH + x] = height as i32;
            }
        }
    }

    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();

        for heightmap_value in &self.heightmap {
            buf.put_i32(*heightmap_value);
        }

        for section in &self.sections {
            if section.is_empty() {
                buf.put_u8(0);
            } else {
                buf.put_u8(1);
                buf.put_slice(&section.encode());
            }
        }

        for biome in &self.biomes {
            buf.put_i32(*biome);
        }

        buf.freeze()
    }
}
