use hex;
use num_bigint::BigUint;
use crate::block::Block;
use std::time::{SystemTime, UNIX_EPOCH};



fn hash_less_than_target(hash: &str, target: &str) -> bool {
    let hash_num = BigUint::from_bytes_be(&hex::decode(hash).unwrap());
    let target_num = BigUint::from_bytes_be(&hex::decode(target).unwrap());
    hash_num < target_num
}

pub fn mine(block: &mut Block) {
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
