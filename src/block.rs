use crate::errors::{BlockValidationError, TransactionError};
use crate::transaction::Transaction;
use crate::utils;
use bincode::{Decode, Encode};
use sha2::{Digest, Sha256};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// TODO: use custom Digest type implementing From

#[derive(Encode)]
struct BlockNoDigest<'a> {
    index: u64,
    timestamp: u64,
    prev_hash: &'a [u8; 32],
    target: &'a [u8; 32],
    transactions: &'a Vec<Transaction>,
    nonce: u64,
}

#[derive(Encode, Decode)]
pub struct Block {
    pub digest: [u8; 32],
    pub index: u64,
    pub timestamp: u64,
    pub prev_hash: [u8; 32],
    pub target: [u8; 32],
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
}

impl Block {
    pub fn new(
        index: u64,
        prev_hash: [u8; 32],
        target: [u8; 32],
        transactions: Vec<Transaction>,
    ) -> Block {
        let mut block = Block {
            digest: [0; 32],
            index,
            timestamp: utils::unix_timestamp(),
            prev_hash,
            target,
            transactions,
            nonce: 0,
        };
        block.update_digest();
        block
    }

    pub fn from_bincode(data: &[u8]) -> Block {
        bincode::decode_from_slice(data, bincode::config::standard())
            .unwrap()
            .0
    }

    pub fn genesis() -> Block {
        Block {
            digest: hex::decode("00094ec2294b08eff5da9c713f9d7cbdb5b84243b0e03f1842bdfe7cc9a66fcd")
                .unwrap()
                .as_slice().try_into().unwrap(),
            index: 0,
            timestamp: 1747162780,
            prev_hash: [0; 32],
            target: hex::decode(
                "000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ).unwrap().as_slice().try_into().unwrap(),
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

    pub fn hash(&self) -> [u8; 32] {
        let block_data = self.as_bincode_no_digest();

        let mut hasher = Sha256::new();
        hasher.update(block_data);
        hasher.finalize().as_slice().try_into().unwrap()
    }

    pub fn update_digest(&mut self) {
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

    pub fn add_coinbase_tx(&mut self, pkhash: [u8; 32], reward: u64) {
        self.transactions
            .insert(0, Transaction::new_coinbase(pkhash, reward, self.index));
        self.update_digest();
    }

    pub fn validate(&self) -> Result<(), BlockValidationError> {
        if self.hash() >= self.target {
            return Err(BlockValidationError::InvalidProofOfWork);
        }
        if self.digest != self.hash() {
            return Err(BlockValidationError::HashDigestMismatch);
        }
        if self.timestamp
            > SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        {
            return Err(BlockValidationError::TimestampInFuture);
        }

        if !self.check_double_spend() {
            return Err(BlockValidationError::InvalidTransactions(TransactionError::DoubleSpend));
        }

        self.validate_transactions()
            .map_err(BlockValidationError::InvalidTransactions)?;

        Ok(())
    }

    fn validate_transactions(&self) -> Result<(), TransactionError> {
        if self.transactions.is_empty() {
            return Err(TransactionError::InvalidCoinbase);
        }
        self.transactions[0].verify_coinbase()?;
        for tx in &self.transactions[1..] {
            tx.verify()?;
        }
        Ok(())
    }

    fn get_spent_utxos(&self) -> Vec<([u8; 32], u16)> {
        let mut spent_utxos = Vec::new();
        for tx in &self.transactions[1..] {
            for input in &tx.inputs {
                spent_utxos.push((input.txid, input.output));
            }
        }
        spent_utxos
    }

    fn check_double_spend(&self) -> bool {
        let mut spent_utxos = self.get_spent_utxos();
        spent_utxos.sort();
        spent_utxos.dedup();
        spent_utxos.len() == self.get_spent_utxos().len()
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Block #{}", self.index)?;
        writeln!(f, "  Timestamp: {}", self.timestamp)?;
        writeln!(f, "  Previous Hash: {}", hex::encode(self.prev_hash))?;
        writeln!(f, "  Nonce: {}", self.nonce)?;
        writeln!(f, "  Hash: {}", hex::encode(self.digest))?;
        writeln!(f, "  Transactions:")?;
        for tx in &self.transactions {
            let indented = tx
                .to_string()
                .lines()
                .map(|line| format!("    {}", line))
                .collect::<Vec<_>>()
                .join("\n");
            writeln!(f, "{}", indented)?;
        }
        Ok(())
    }
}

mod tests {
    use super::*;
    use crate::transaction::{Transaction, TxInput, TxOutput};

    #[test]
    fn test_invalid_pow() {
        let block = Block::new(
            0,
            [0; 32],
            hex::decode("000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            Vec::new(),
        );
        assert_eq!(
            block.validate(),
            Err(BlockValidationError::InvalidProofOfWork)
        );
    }

    #[test]
    fn test_invalid_digest() {
        let mut block = Block::new(
            0,
            [0; 32],
            hex::decode("000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            Vec::new(),
        );
        utils::mine(&mut block, [0; 32], 0);
        block.digest = [0; 32];
        assert_eq!(
            block.validate(),
            Err(BlockValidationError::HashDigestMismatch)
        );
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut block = Block::new(
            0,
            [0; 32],
            hex::decode("000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            Vec::new(),
        );
        block.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 1000;

        while !utils::hash_less_than_target(&block.digest, &block.target) {
            block.nonce += 1;
            block.update_digest();
        }

        assert_eq!(
            block.validate(),
            Err(BlockValidationError::TimestampInFuture)
        );
    }

    #[test]
    fn test_invalid_transactions() {
        let mut block = Block::new(
            0,
            [0; 32],
            hex::decode("000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            Vec::new(),
        );
        let tx = Transaction::new(vec![], vec![]);
        block.add_tx(tx);
        utils::mine(&mut block, [0; 32], 0);
        assert_eq!(
            block.validate(),
            Err(BlockValidationError::InvalidTransactions(
                TransactionError::EmptyInputs
            ))
        );
    }

    #[test]
    fn test_deserialise_block() {
        let mut block = Block::new(
            10,
            hex::decode("abcd000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            hex::decode("000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            vec![Transaction::new(
                vec![TxInput::new_unsigned([1; 32], 0)],
                vec![TxOutput::new(50, [2; 32])],
            )],
        );
        utils::mine(&mut block, [0; 32], 0);

        let serialised = block.as_bincode();
        let deserialised = Block::from_bincode(&serialised);

        assert_eq!(block.index, deserialised.index);
        assert_eq!(block.prev_hash, deserialised.prev_hash);
        assert_eq!(block.target, deserialised.target);
        assert_eq!(block.transactions.len(), deserialised.transactions.len());
        assert_eq!(block.timestamp, deserialised.timestamp);
        assert_eq!(block.nonce, deserialised.nonce);

        assert_eq!(deserialised.transactions[0].id, block.transactions[0].id);
        assert_eq!(
            deserialised.transactions[0].inputs[0].txid,
            block.transactions[0].inputs[0].txid
        );
        assert_eq!(
            deserialised.transactions[0].inputs[0].output,
            block.transactions[0].inputs[0].output
        );
        assert_eq!(
            deserialised.transactions[0].outputs[0].value,
            block.transactions[0].outputs[0].value
        );
        assert_eq!(
            deserialised.transactions[0].outputs[0].pkhash,
            block.transactions[0].outputs[0].pkhash
        );
    }
}
