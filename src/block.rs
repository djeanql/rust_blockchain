use crate::transaction::Transaction;
use crate::utils;
use sha2::{Digest, Sha256};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Block {
    pub digest: String,
    pub index: u32,
    pub timestamp: u64,
    pub prev_hash: String,
    pub target: String,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
}

impl Block {
    pub fn new(
        index: u32,
        prev_hash: String,
        target: String,
        transactions: Vec<Transaction>,
    ) -> Block {
        let mut block = Block {
            digest: String::new(),
            index,
            timestamp: utils::unix_timestamp(),
            prev_hash,
            target,
            transactions: transactions,
            nonce: 0,
        };
        block.update_digest();
        block
    }

    pub fn genesis() -> Block {
        Block {
            digest: String::from(
                "00094ec2294b08eff5da9c713f9d7cbdb5b84243b0e03f1842bdfe7cc9a66fcd",
            ),
            index: 0,
            timestamp: 1747162780,
            prev_hash: String::new(),
            target: String::from(
                "000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ),
            transactions: Vec::new(),
            nonce: 8376,
        }
    }

    fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
          "index": self.index,
          "timestamp": self.timestamp,
          "prev_hash": self.prev_hash,
          "transactions": self.transactions,
          "target": self.target,
          "nonce": self.nonce})
    }

    pub fn hash(&self) -> String {
        let block_data = self.as_json().to_string();

        let mut hasher = Sha256::new();
        hasher.update(block_data);
        format!("{:x}", hasher.finalize())
    }

    fn update_digest(&mut self) {
        self.digest = self.hash();
    }

    pub fn update_nonce_and_timestamp(&mut self) {
        self.nonce += 1;
        if self.nonce % 1000 == 0 {
            self.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        self.update_digest();
    }

    pub fn add_tx(&mut self, tx: Transaction) {
        self.transactions.push(tx);
        self.update_digest();
    }

    pub fn validate(&self) -> bool {
        self.hash() < self.target
            && self.digest == self.hash()
            && self.timestamp
                <= SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            && self.validate_transactions()
    }

    fn validate_transactions(&self) -> bool {
        for tx in &self.transactions {
            if !tx.verify_signature() {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Block #{}", self.index)?;
        writeln!(f, "  Timestamp: {}", self.timestamp)?;
        writeln!(f, "  Previous Hash: {}", self.prev_hash)?;
        writeln!(f, "  Nonce: {}", self.nonce)?;
        writeln!(f, "  Hash: {}", self.digest)?;
        writeln!(f, "  Transactions:")?;
        for tx in &self.transactions {
            writeln!(f, "    {}", tx)?;
        }
        Ok(())
    }
}
