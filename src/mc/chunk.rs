use std::collections::HashMap;
use crate::mc::block::{Block, BlockState, BlockPos, BlockDirection, BlockEntity};
use crate::mc::entity::Entity;
use crate::{InstanceRaw, Instance};
use cgmath::Quaternion;
use std::cell::RefCell;

const CHUNK_WIDTH: usize = 16;
const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
const CHUNK_HEIGHT: usize = 256;

type ChunkPos = (u32, u32);

pub struct Chunk<'block> {
    pub pos: ChunkPos,
    pub blocks: [[BlockState<'block>; CHUNK_AREA]; CHUNK_HEIGHT]
}

impl<'block> Chunk<'block> {
    fn blockstate_at_pos(&self, pos: BlockPos) -> BlockState {
        let x = (pos.0 % 16) as usize;
        let y = (pos.1) as usize;
        let z = (pos.2 % 16) as usize;

        self.blocks[y][
            (z * CHUNK_WIDTH) + x
        ]
    }
}

pub struct ChunkManager<'block> {
    pub loaded_chunks: Vec<Chunk<'block>>
}

impl<'block> ChunkManager<'block> {
    pub fn new() -> Self {
        ChunkManager { loaded_chunks: vec![] }
    }
}