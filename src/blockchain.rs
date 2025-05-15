use crate::block::Block;
use std::fmt;

pub struct Blockchain {
    chain: Vec<Block>,
    pub target: String,
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

    pub fn add_block(&mut self, block: Block) -> Result<(), &str> {
        if self.validate_block(&block) {
            self.chain.push(block);
            Ok(())
        } else {
            Err("Invalid block")
        }
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        block.validate()
            && block.prev_hash == self.prev_hash()
            && block.index == self.chain.last().unwrap().index + 1
            && block.timestamp >= self.chain.last().unwrap().timestamp
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
