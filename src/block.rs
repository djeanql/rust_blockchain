use crate::transaction::{Transaction, TransactionError};
use crate::utils;
use bincode::{Decode, Encode};
use sha2::{Digest, Sha256};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Encode)]
struct BlockNoDigest<'a> {
    index: u32,
    timestamp: u64,
    prev_hash: &'a String,
    target: &'a String,
    transactions: &'a Vec<Transaction>,
    nonce: u64,
}

#[derive(Encode, Decode)]
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

    
    #[allow(dead_code)]
    pub fn from_bincode(data: &[u8]) -> Block {
        bincode::decode_from_slice(data, bincode::config::standard())
            .unwrap()
            .0
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

    #[allow(dead_code)]
    pub fn as_bincode(&self) -> Vec<u8> {
        bincode::encode_to_vec(self, bincode::config::standard()).unwrap()
    }

    fn as_bincode_no_digest(&self) -> Vec<u8> {
        let no_digest = BlockNoDigest {
            index: self.index,
            timestamp: self.timestamp,
            prev_hash: &self.prev_hash,
            target: &self.target,
            transactions: &self.transactions,
            nonce: self.nonce,
        };

        bincode::encode_to_vec(no_digest, bincode::config::standard()).unwrap()
    }

    pub fn hash(&self) -> String {
        let block_data = self.as_bincode_no_digest();

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

    //TODO: validate transaction as added
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
            && self.validate_transactions().is_ok()
    }

    fn validate_transactions(&self) -> Result<(), TransactionError> {
    for tx in &self.transactions {
        tx.verify()?;
    }
    Ok(())
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
