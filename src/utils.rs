use crate::block::Block;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn mine(block: &mut Block) {
    while block.digest > block.target {
        block.update_nonce_and_timestamp();
    }
}

pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
