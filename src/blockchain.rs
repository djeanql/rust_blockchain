use crate::block::Block;
use crate::errors::{BlockValidationError, TransactionError};
use crate::utxo::UTXOSet;
use sha2::{Digest, Sha256};
use std::fmt;

pub struct Blockchain {
    chain: Vec<Block>,
    target: String,
    pub utxos: UTXOSet,
}

//TODO: add difficulty adjustment

impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            chain: vec![Block::genesis()],
            target: String::from(
                "000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ),
            utxos: UTXOSet::new(),
        }
    }

    pub fn get_block_reward(&self) -> u64 {
        50_000_000
    }

    pub fn next_block(&self) -> Block {
        Block::new(
            self.chain.len() as u64,
            self.prev_hash(),
            self.target.clone(),
            Vec::new(),
        )
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockValidationError> {
        self.validate_block(&block)?;
        self.utxos.update_with_block(&block);
        self.chain.push(block);
        Ok(())
    }

    fn validate_transactions_stateful(&self, block: &Block) -> Result<(), TransactionError> {
        //TODO: check coinbase reward
        //TODO: check for double spend

        for tx in &block.transactions[1..] {
            let mut inputs_total = 0;

            for input in &tx.inputs {
                if self.utxos.get_utxo(input.txid, input.output).is_none() {
                    return Err(TransactionError::InvalidUTXO);
                }

                let utxo = self.utxos.get_utxo(input.txid, input.output).unwrap();

                let input_pkhash: [u8; 32] =
                    Sha256::digest(input.pubkey).as_slice().try_into().unwrap();
                if input_pkhash != utxo.pkhash {
                    return Err(TransactionError::UnauthorizedSpend);
                }

                inputs_total += utxo.value as i64;
            }

            let total_fees = inputs_total - tx.outputs.iter().map(|o| o.value as i64).sum::<i64>();
            if total_fees < 0 {
                return Err(TransactionError::Overspend);
            }
        }

        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> Result<(), BlockValidationError> {
        block.validate()?;
        self.validate_transactions_stateful(block)
            .map_err(BlockValidationError::InvalidTransactions)?;

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
