use rust_blockchain::{wallet, blockchain, block, transaction, errors, utils};

use block::Block;
use errors::{BlockValidationError, TransactionError};
use transaction::{Transaction, TxInput, TxOutput};
use utils::mine;
use wallet::Wallet;
use blockchain::Blockchain;

//TODO: separate out the tests into separate modules
//TODO: Check all stateful validation errors and double spending


#[test]
fn test_spend_utxo() {
    let wallet = Wallet::new();
    let mut blockchain = Blockchain::new();
    let mut block = blockchain.next_block();

    mine(&mut block, wallet.pkhash, blockchain.get_block_reward());

    assert_eq!(blockchain.add_block(block), Ok(()));

    let (txid, index) = &blockchain.utxos.utxos_from_pkhash(wallet.pkhash)[0];

    let mut block2 = blockchain.next_block();

    let inputs = vec![TxInput::new_unsigned(*txid, *index)];

    let outputs = vec![
        TxOutput::new(100, [0; 32]),       // unspendable
        TxOutput::new(200, wallet.pkhash), // send to self
    ];

    let mut tx = Transaction::new(inputs, outputs);

    wallet.sign_transaction(&mut tx);
    let txid = tx.id;

    block2.add_tx(tx);
    mine(&mut block2, wallet.pkhash, blockchain.get_block_reward());

    assert_eq!(blockchain.add_block(block2), Ok(()));
    assert!(
        blockchain.utxos.get_utxo(txid, 0).is_some()
            && blockchain.utxos.get_utxo(txid, 1).is_some()
    );
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
    assert_eq!(
        blockchain.add_block(block),
        Err(BlockValidationError::InvalidIndex)
    );
}

#[test]
fn test_invalid_timestamp() {
    let mut blockchain = Blockchain::new();
    let mut block = blockchain.next_block();
    mine(&mut block, [0; 32], blockchain.get_block_reward());
    blockchain.add_block(block).unwrap();

    let mut block2 = blockchain.next_block();
    block2.timestamp = 1000;
    block2.add_coinbase_tx([0; 32], blockchain.get_block_reward());
    while block2.hash() > block2.target {
        block2.nonce += 1;
    }
    block2.update_digest();

    assert_eq!(
        blockchain.add_block(block2),
        Err(BlockValidationError::InvalidTimestamp)
    )
}

#[test]
fn test_invalid_prev_hash() {
    let mut blockchain = Blockchain::new();
    let mut block = blockchain.next_block();
    block.prev_hash = String::from("invalid_hash");
    mine(&mut block, [0; 32], blockchain.get_block_reward());
    assert_eq!(
        blockchain.add_block(block),
        Err(BlockValidationError::InvalidPreviousHash)
    )
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

    let outputs = vec![TxOutput::new(100, [0; 32]), TxOutput::new(200, [1; 32])];

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

    let outputs = vec![TxOutput::new(100, [0; 32]), TxOutput::new(200, [1; 32])];

    let mut tx = Transaction::new(inputs, outputs);

    wallet.sign_transaction(&mut tx);

    assert!(tx.verify().is_ok());
}

#[test]
fn test_unauthorized_spend_rejected() {
    let mut blockchain = Blockchain::new();
    let wallet = Wallet::new();

    let mut block = blockchain.next_block();

    mine(&mut block, [0; 32], blockchain.get_block_reward()); // block reward UTXO will be owned by null address
    blockchain.add_block(block).unwrap();

    let (txid, output_index) = blockchain.utxos.utxos_from_pkhash([0; 32])[0];

    let inputs = vec![
        TxInput::new_unsigned(txid, output_index), // attempt to spend coinbase UTXO
    ];

    let outputs = vec![TxOutput::new(100, [0; 32])];

    let mut tx = Transaction::new(inputs, outputs);
    let mut block = blockchain.next_block();
    wallet.sign_transaction(&mut tx);
    block.add_tx(tx);

    mine(&mut block, [0; 32], blockchain.get_block_reward());

    let result = blockchain.add_block(block);

    assert_eq!(
        result,
        Err(BlockValidationError::InvalidTransactions(
            TransactionError::UnauthorizedSpend
        ))
    );
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

    let outputs = vec![TxOutput::new(100, [0; 32]), TxOutput::new(200, [1; 32])];

    let mut tx = Transaction::new(inputs, outputs);
    wallet.sign_transaction(&mut tx);

    block.add_tx(tx);

    while block.hash() > block.target {
        block.nonce += 1;
    }
    block.update_digest();

    assert_eq!(
        blockchain.add_block(block),
        Err(BlockValidationError::InvalidTransactions(
            TransactionError::InvalidCoinbase
        ))
    );
}

#[test]
fn test_invalid_block_reward() {
    let mut blockchain = Blockchain::new();
    let mut block = blockchain.next_block();
    mine(&mut block, [0; 32], 1000);

    assert_eq!(
        blockchain.add_block(block),
        Err(BlockValidationError::InvalidTransactions(
            TransactionError::InvalidCoinbase
        ))
    )
}

#[test]
fn test_duplicate_coinbase_tx() {
    let mut blockchain = Blockchain::new();
    let mut block = blockchain.next_block();

    block.add_coinbase_tx([0; 32], blockchain.get_block_reward());
    // add coinbase tx again in mine function
    mine(&mut block, [0; 32], blockchain.get_block_reward());

    assert_eq!(
        blockchain.add_block(block),
        Err(BlockValidationError::InvalidTransactions(
            TransactionError::InvalidPublicKey
        ))
    )
}

