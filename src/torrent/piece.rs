use std::vec::Vec;

const BLOCK_SIZE: i16 = 16384;

struct Block {
    size: i64,
    acquired: bool,
}

pub struct Piece {
    size: i64,
    hash: Vec<u8>,
    verified: bool,
    blocks: Vec<Block>,
}

impl Piece {
    pub fn new(size: i64, hash: Vec<u8>) -> Piece {
        let mut piece = Piece{
            size: size,
            hash: hash,
            verified: false,
            blocks: Vec::new(),
        };

        let block_count: usize = ((size as f64) / (BLOCK_SIZE as f64)).ceil() as usize;
        piece.blocks.reserve(block_count);

        piece
    }
}