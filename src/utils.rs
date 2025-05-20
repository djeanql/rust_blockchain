use crate::block::Block;
use num_bigint::BigUint;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn hash_less_than_target(hash: &str, target: &str) -> bool {
    let hash_num = BigUint::from_bytes_be(&hex::decode(hash).unwrap());
    let target_num = BigUint::from_bytes_be(&hex::decode(target).unwrap());
    hash_num < target_num
}

pub fn mine(block: &mut Block, miner_pkhash: [u8; 32], block_reward: u64) {
    block.add_coinbase_tx(miner_pkhash, block_reward);
    while !hash_less_than_target(&block.digest, &block.target) {
        block.update_nonce_and_timestamp();
    }
}

pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
