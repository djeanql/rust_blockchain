use crate::utils::unix_timestamp;
use k256::ecdsa::{Signature, SigningKey, signature::Signer, signature::Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub id: String,
    pub timestamp: u64,
    #[serde(skip)]
    pub signature: String,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: f64) -> Transaction {
        let mut tx = Transaction {
            sender,
            receiver,
            amount,
            id: String::new(),
            timestamp: unix_timestamp(),
            signature: String::new(),
        };

        tx.id = tx.hash();
        tx
    }

    fn sighash(&self) -> String {
        let data = serde_json::to_string(self).expect("Failed to serialize transaction");
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn hash(&self) -> String {
        let serializable_tx = SerializableTx {
            sender: &self.sender,
            receiver: &self.receiver,
            amount: self.amount,
            id: &self.id,
            timestamp: self.timestamp,
            signature: &self.signature,
        };

        let data = serde_json::to_string(&serializable_tx).expect("Failed to serialize transaction");
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn sign(&mut self, key: &SigningKey) {
        let sig: Signature = key.sign(self.sighash().as_bytes());
        self.signature = hex::encode(sig.to_der());
    }

    pub fn verify_signature(&self) -> bool {
        let pubkey_bytes = hex::decode(&self.sender).expect("Could not decode sender pubkey");
        let verify_key =
            k256::ecdsa::VerifyingKey::from_sec1_bytes(&pubkey_bytes).expect("Invalid public key");

        let der_bytes = hex::decode(&self.signature).expect("Could not decode signature");
        let signature =
            k256::ecdsa::Signature::from_der(&der_bytes).expect("Invalid DER signature");

        verify_key
            .verify(self.sighash().as_bytes(), &signature)
            .is_ok()
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} -> {}: {} (ID: {})",
            self.sender, self.receiver, self.amount, self.id
        )
    }
}

#[derive(Serialize)]
struct SerializableTx<'a> {
    sender: &'a String,
    receiver: &'a String,
    amount: f64,
    id: &'a String,
    timestamp: u64,
    signature: &'a String,
}
