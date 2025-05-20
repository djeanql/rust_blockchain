mod block;
mod blockchain;
mod transaction;
mod utils;
mod wallet;
mod errors;
use blockchain::{Blockchain};
use transaction::{Transaction, TxInput, TxOutput};
use utils::*;
use wallet::Wallet;
use errors::{BlockValidationError, TransactionError};

fn main() {
    println!("Hello, world!");

    let mut blockchain = Blockchain::new();
    let mut block = blockchain.next_block();

    let wallet = Wallet::new();

    let inputs = vec![
        TxInput::new_unsigned([0; 32], 2),
        TxInput::new_unsigned([0; 32], 1),
    ];

    let outputs = vec![
        TxOutput::new(100, [0; 32]),
        TxOutput::new(200, [1; 32]),
    ];

    let mut tx = Transaction::new(inputs, outputs);

    wallet.sign_transaction(&mut tx);

    println!("Transaction ID: {}", hex::encode(tx.id));
    block.add_tx(tx);

    mine(&mut block, wallet.pkhash, blockchain.get_block_reward());

    println!("Block Digest: {:?}", block.digest);
    println!("Block Timestamp: {:?}", block.timestamp);

    blockchain.add_block(block).unwrap();

    let mut block2 = blockchain.next_block();
    mine(&mut block2, wallet.pkhash, blockchain.get_block_reward());
    blockchain.add_block(block2).unwrap();

    println!("{}", blockchain);
}

//TODO: separate out the tests into separate modules
#[cfg(test)]
mod tests {
    use super::*;
    use block::Block;

    #[test]
    fn test_valid_block() {
        let wallet = Wallet::new();
        let mut blockchain = Blockchain::new();
        let mut block = blockchain.next_block();

        let inputs = vec![
            TxInput::new_unsigned([0; 32], 2),
            TxInput::new_unsigned([0; 32], 1),
        ];

        let outputs = vec![
            TxOutput::new(100, [0; 32]),
            TxOutput::new(200, [1; 32]),
        ];

        let mut tx = Transaction::new(inputs, outputs);

        wallet.sign_transaction(&mut tx);
        block.add_tx(tx);
        mine(&mut block, wallet.pkhash, blockchain.get_block_reward());
        assert_eq!(blockchain.add_block(block), Ok(()));
    }

    #[test]
    fn test_invalid_pow() {
        let mut blockchain = Blockchain::new();
        assert_eq!(
            blockchain.add_block(blockchain.next_block()),
            Err(BlockValidationError::InvalidProofOfWork),
        )
    }

    #[test]
    fn test_invalid_index() {
        let mut blockchain = Blockchain::new();
        let mut block = blockchain.next_block();
        block.index = 2;
        mine(&mut block, [0; 32], blockchain.get_block_reward());
        assert_eq!(blockchain.add_block(block), Err(BlockValidationError::InvalidIndex));
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut blockchain = Blockchain::new();
        let mut block = blockchain.next_block();
        block.timestamp = utils::unix_timestamp();
        mine(&mut block, [0; 32], blockchain.get_block_reward());
        blockchain.add_block(block).unwrap();
        println!("{}", blockchain);

        let mut block2 = blockchain.next_block();
        block2.timestamp = 1000;
        block2.add_coinbase_tx([0; 32], 0);
        while block2.hash() > block2.target {
            block2.nonce += 1;
        }
        block2.update_digest();

        assert_eq!(blockchain.add_block(block2), Err(BlockValidationError::InvalidTimestamp))
    }

    #[test]
    fn test_invalid_prev_hash() {
        let mut blockchain = Blockchain::new();
        let mut block = blockchain.next_block();
        block.prev_hash = String::from("invalid_hash");
        mine(&mut block, [0; 32], blockchain.get_block_reward());
        assert_eq!(blockchain.add_block(block), Err(BlockValidationError::InvalidPreviousHash))
    }

    #[test]
    fn test_digest_update() {
        let mut block = Block::new(
            0,
            String::from(""),
            String::from("000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            Vec::new(),
        );
        let old_digest = block.digest.clone();
        block.update_nonce_and_timestamp();
        assert_ne!(block.digest, old_digest);

        let inputs = vec![
            TxInput::new_unsigned([0; 32], 2),
            TxInput::new_unsigned([0; 32], 1),
        ];

        let outputs = vec![
            TxOutput::new(100, [0; 32]),
            TxOutput::new(200, [1; 32]),
        ];

        let tx = Transaction::new(inputs, outputs);

        block.add_tx(tx);
        block.update_nonce_and_timestamp();
        assert_ne!(block.digest, old_digest);
    }

    #[test]
    fn test_transaction_sign_and_verify() {
        let wallet = Wallet::new();

        let inputs = vec![
            TxInput::new_unsigned([0; 32], 2),
            TxInput::new_unsigned([0; 32], 1),
        ];

        let outputs = vec![
            TxOutput::new(100, [0; 32]),
            TxOutput::new(200, [1; 32]),
        ];

        let mut tx = Transaction::new(inputs, outputs);

        wallet.sign_transaction(&mut tx);

        assert!(tx.verify().is_ok());
    }

    #[test]
    fn test_transaction_invalid_signature() {
        let mut blockchain = Blockchain::new();

        let mut block = blockchain.next_block();

        let inputs = vec![
            TxInput::new_unsigned([0; 32], 2),
            TxInput::new_unsigned([0; 32], 1),
        ];

        let outputs = vec![
            TxOutput::new(100, [0; 32]),
            TxOutput::new(200, [1; 32]),
        ];

        let tx = Transaction::new(inputs, outputs);

        block.add_tx(tx);
        mine(&mut block, [0; 32], blockchain.get_block_reward());

        let result = blockchain.add_block(block);
        
        assert!(result == Err(BlockValidationError::InvalidTransactions(TransactionError::InvalidSignature)) ||
            result == Err(BlockValidationError::InvalidTransactions(TransactionError::InvalidPublicKey)));
    }

    #[test]
    fn test_missing_coinbase_tx() {
        let wallet = Wallet::new();
        let mut blockchain = Blockchain::new();
        let mut block = blockchain.next_block();

        let inputs = vec![
            TxInput::new_unsigned([0; 32], 2),
            TxInput::new_unsigned([0; 32], 1),
        ];

        let outputs = vec![
            TxOutput::new(100, [0; 32]),
            TxOutput::new(200, [1; 32]),
        ];

        let mut tx = Transaction::new(inputs, outputs);
        wallet.sign_transaction(&mut tx);

        block.add_tx(tx);
        
        while block.hash() > block.target {
            block.nonce += 1;
        }
        block.update_digest();

        assert_eq!(blockchain.add_block(block), Err(BlockValidationError::InvalidTransactions(TransactionError::InvalidCoinbase)));
    }

    #[test]
    fn test_deserialise_block() {
        let blockchain = Blockchain::new();
        let block = blockchain.next_block();

        let serialised = block.as_bincode();
        let deserialised = Block::from_bincode(&serialised);

        assert_eq!(block.index, deserialised.index);
        assert_eq!(block.prev_hash, deserialised.prev_hash);
        assert_eq!(block.target, deserialised.target);
        assert_eq!(block.transactions.len(), deserialised.transactions.len());
    }
}
