use crate::transaction::Transaction;
use k256::ecdsa::{SigningKey, VerifyingKey};

pub struct Wallet {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
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
            address: pubkey_hex,
        }
    }

    pub fn sign_transaction(&self, tx: &mut Transaction) {
        tx.sign(&self.signing_key);
    }
}
