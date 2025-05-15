use crate::utils::unix_timestamp;
use bincode::Encode;
use k256::ecdsa::{Signature, SigningKey, signature::Signer, signature::Verifier};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Encode)]
pub struct TransactionNoSignature<'a> {
    pub sender: &'a String,
    pub receiver: &'a String,
    pub amount: f64,
    pub timestamp: u64,
}

#[derive(Encode)]
pub struct TransactionNoID<'a> {
    pub sender: &'a String,
    pub receiver: &'a String,
    pub amount: f64,
    pub timestamp: u64,
    pub signature: String,
}

#[derive(Encode)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub id: String,
    pub timestamp: u64,
    pub signature: String,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: f64) -> Transaction {
        Transaction {
            sender,
            receiver,
            amount,
            id: String::new(),
            timestamp: unix_timestamp(),
            signature: String::new()
        }
    }

    pub fn set_id(&mut self) {
        self.id = self.hash();
    }

    pub fn sighash(&self) -> String {
        let data = self.as_bincode_no_signature();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        format!("{:x}", hasher.finalize())
    }

    pub fn hash(&self) -> String {
        let data = self.as_bincode_no_id();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        format!("{:x}", hasher.finalize())
    }

    fn as_bincode(&self) -> Vec<u8> {
        bincode::encode_to_vec(self, bincode::config::standard()).unwrap()
    }

    fn as_bincode_no_signature(&self) -> Vec<u8> {
        let no_signature = TransactionNoSignature {
            sender: &self.sender,
            receiver: &self.receiver,
            amount: self.amount,
            timestamp: self.timestamp,
        };

        bincode::encode_to_vec(no_signature, bincode::config::standard()).unwrap()
    }

    fn as_bincode_no_id(&self) -> Vec<u8> {
        let no_id = TransactionNoID {
            sender: &self.sender,
            receiver: &self.receiver,
            amount: self.amount,
            timestamp: self.timestamp,
            signature: self.signature.clone(),
        };

        bincode::encode_to_vec(no_id, bincode::config::standard()).unwrap()
    }

    pub fn sign(&mut self, key: &SigningKey) {
        let sig: Signature = key.sign(self.sighash().as_bytes());
        self.signature = hex::encode(sig.to_der());
        self.set_id();
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
