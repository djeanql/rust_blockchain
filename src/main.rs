mod block;
mod blockchain;
mod transaction;
mod utils;
mod wallet;
use blockchain::Blockchain;
use transaction::Transaction;
use utils::*;
use wallet::Wallet;

fn main() {
    println!("Hello, world!");

    let mut blockchain = Blockchain::new();
    let mut block = blockchain.next_block();

    let wallet = Wallet::new();

    println!("{:?}", block.digest);

    let mut tx = Transaction::new(
        wallet.address.clone(),
        wallet.address.clone(),
        1.0,
    );

    wallet.sign_transaction(&mut tx);

    block.add_tx(tx);

    mine(&mut block);

    println!("{:?}", block.digest);
    println!("{:?}", block.timestamp);

    blockchain.add_block(block).expect("Failed to add block");

    let mut block2 = blockchain.next_block();
    mine(&mut block2);
    blockchain.add_block(block2).expect("Failed to add block");

    println!("{}", blockchain);
}

#[cfg(test)]
mod tests {
    use super::*;
    use block::Block;

    #[test]
    fn test_invalid_hash() {
        let mut blockchain = Blockchain::new();
        assert_eq!(
            blockchain.add_block(blockchain.next_block()),
            Err("Invalid block")
        )
    }

    #[test]
    fn test_invalid_index() {
        let mut blockchain = Blockchain::new();
        let mut block = blockchain.next_block();
        block.index = 2;
        mine(&mut block);
        assert_eq!(blockchain.add_block(block), Err("Invalid block"))
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut blockchain = Blockchain::new();
        let mut block = blockchain.next_block();
        block.timestamp = utils::unix_timestamp();
        mine(&mut block);
        blockchain.add_block(block).unwrap();
        println!("{}", blockchain);

        let mut block2 = blockchain.next_block();
        block2.timestamp = 1000;
        while block2.hash() > block2.target {
            block2.nonce += 1;
        }

        assert_eq!(blockchain.add_block(block2), Err("Invalid block"))
    }

    #[test]
    fn test_invalid_prev_hash() {
        let mut blockchain = Blockchain::new();
        let mut block = blockchain.next_block();
        block.prev_hash = String::from("invalid_hash");
        mine(&mut block);
        assert_eq!(blockchain.add_block(block), Err("Invalid block"))
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
        block.update_nonce_and_timestamp();
        assert_ne!(block.digest, old_digest);
    }

    #[test]
    fn test_transaction_sign_and_verify() {
        let wallet = Wallet::new();

        let mut tx = Transaction::new(
            wallet.address.clone(),
            wallet.address.clone(), //send to self for testing
            42.0,
        );

        wallet.sign_transaction(&mut tx);

        assert!(!tx.signature.is_empty());
        assert!(tx.verify_signature());
    }
}
