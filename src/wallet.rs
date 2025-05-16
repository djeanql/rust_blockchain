use crate::transaction::Transaction;
use k256::ecdsa::{SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};

pub struct Wallet {
    signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub pkhash: [u8; 32],
    pub address: String,
}

impl Wallet {
    pub fn new() -> Wallet {
        let signing_key = SigningKey::random(&mut rand_core::OsRng);
        let verifying_key = signing_key.verifying_key().clone();

        let encoded_point = verifying_key.to_encoded_point(false);
        let pubkey_bytes = encoded_point.as_bytes();
        let pubkey_hex = hex::encode(pubkey_bytes);

        Wallet {
            signing_key,
            verifying_key,
            pkhash: Sha256::digest(&pubkey_bytes).into(),
            address: pubkey_hex,
        }
    }

    pub fn sign_transaction(&self, tx: &mut Transaction) {
        tx.sign(&self.signing_key);
    }
}
