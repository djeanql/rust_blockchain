use crate::block::{Block, BlockIntegrityError};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum BlockValidationError {
    Integrity(BlockIntegrityError),
    InvalidPreviousHash,
    InvalidIndex,
    InvalidTimestamp,
}

pub struct Blockchain {
    chain: Vec<Block>,
    target: String,
}

//TODO: add difficulty adjustment
//TODO: coinbase transactions

impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            chain: vec![Block::genesis()],
            target: String::from(
                "000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ),
        }
    }

    pub fn next_block(&self) -> Block {
        Block::new(
            self.chain.len() as u32,
            self.prev_hash(),
            self.target.clone(),
            Vec::new(),
        )
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockValidationError> {
        self.validate_block(&block)?;
        self.chain.push(block);
        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> Result<(), BlockValidationError> {
        block.validate().map_err(|e| BlockValidationError::Integrity(e))?;

        if block.prev_hash != self.prev_hash() {
            return Err(BlockValidationError::InvalidPreviousHash);
        }
        if block.index != self.chain.last().unwrap().index + 1 {
            return Err(BlockValidationError::InvalidIndex);
        }
        if block.timestamp < self.chain.last().unwrap().timestamp {
            return Err(BlockValidationError::InvalidTimestamp);
        }
        Ok(())
    }

    pub fn prev_hash(&self) -> String {
        match self.chain.last() {
            Some(block) => block.digest.clone(),
            None => String::from(""),
        }
    }
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\nBlockchain:")?;
        writeln!(f, "Target: {}", self.target)?;
        writeln!(f, "Number of blocks: {}", self.chain.len())?;
        for block in &self.chain {
            writeln!(f, "\n{}", block)?;
        }
        Ok(())
    }
}
