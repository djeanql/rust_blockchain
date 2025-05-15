use bincode::{Encode, Decode};
use k256::ecdsa::{Signature, SigningKey, signature::Signer};
use k256::ecdsa::signature::Verifier;
use sha2::{Digest, Sha256};
use crate::{utils, wallet::Wallet};

#[derive(Encode, Decode, Clone)]
struct TxInput {
    pub txid: String,
    pub output: u16,
    pub signature: String,
    pub pubkey: String,
}

impl TxInput {
    pub fn sign(&mut self, signing_key: &SigningKey) {
        let tx_for_sign: TxInputForSign = self.clone().into();
        let signature: Signature = signing_key.sign(tx_for_sign.sighash().as_bytes());
        self.signature = hex::encode(signature.to_der().as_bytes());
    }

    pub fn verify_signature(&self) -> bool {
        let tx_for_sign: TxInputForSign = self.clone().into();


        let pubkey_bytes = hex::decode(&self.pubkey).expect("Could not decode sender pubkey");
        let verify_key =
            k256::ecdsa::VerifyingKey::from_sec1_bytes(&pubkey_bytes).expect("Invalid public key");

        let der_bytes = hex::decode(&self.signature).expect("Could not decode signature");
        let signature =
            k256::ecdsa::Signature::from_der(&der_bytes).expect("Invalid DER signature");

        verify_key
            .verify(tx_for_sign.sighash().as_bytes(), &signature)
            .is_ok()
    }
}

#[derive(Encode, Decode)]
pub struct TxInputForSign {
    pub txid: String,
    pub output: u16,
    pub pubkey: String,
}

impl TxInputForSign {
    fn as_bincode(&self) -> Vec<u8> {
        bincode::encode_to_vec(
            self,
            bincode::config::standard(),
        )
        .unwrap()
    }

    fn sighash(&self) -> String {
        let data = self.as_bincode();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        format!("{:x}", hasher.finalize())
    }
}

impl From<TxInput> for TxInputForSign {
    fn from(input: TxInput) -> Self {
        TxInputForSign {
            txid: input.txid,
            output: input.output,
            pubkey: input.pubkey,
        }
    }
}


//todo: use public key hash and include pubkey in inputs
#[derive(Encode, Decode, Clone)]
struct TxOutput {
    pub value: u64,
    pub receiver_pk: String,
}

pub struct Transaction {
    pub id: String,
    pub timestamp: u64,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

#[derive(Encode, Decode, Clone)]
struct TransactionNoID {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub timestamp: u64,
}

impl Transaction {
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Transaction {
        Transaction {
            id: String::new(),
            timestamp: utils::unix_timestamp(),
            inputs,
            outputs,
        }
    }

    fn as_bincode_no_id(&self) -> Vec<u8> {
        let no_id = TransactionNoID {
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            timestamp: self.timestamp,
        };

        bincode::encode_to_vec(no_id, bincode::config::standard()).unwrap()
    }

    fn hash(&self) -> String {
        let data = self.as_bincode_no_id();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        format!("{:x}", hasher.finalize())
    }

    pub fn sign(&mut self, signing_key: &SigningKey) {
        self.inputs.iter_mut().for_each(|input| {
            input.sign(signing_key);
        });
        self.id = self.hash();
    }

    pub fn verify_signatures(&self) -> bool {
        self.inputs.iter().all(|input| input.verify_signature())
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transaction() {
        let inputs = vec![
            TxInput {
                txid: "txid1".to_string(),
                output: 0,
                signature: "signature1".to_string(),
                pubkey: "pubkey".to_string(),
            },
            TxInput {
                txid: "txid2".to_string(),
                output: 1,
                signature: "signature2".to_string(),
                pubkey: "pubkey".to_string(),
            },
        ];

        let outputs = vec![
            TxOutput {
                value: 100,
                receiver_pk: "receiver_pk1".to_string(),
            },
            TxOutput {
                value: 200,
                receiver_pk: "receiver_pk2".to_string(),
            },
        ];

        let transaction = Transaction::new(inputs, outputs);
        assert_eq!(transaction.inputs.len(), 2);
        assert_eq!(transaction.outputs.len(), 2);
    }

    #[test]
    fn test_sign() {
        let wallet = Wallet::new();

        let inputs = vec![
            TxInput {
                txid: "txid1".to_string(),
                output: 0,
                signature: String::new(),
                pubkey: wallet.address.clone(),
            },
            TxInput {
                txid: "txid2".to_string(),
                output: 1,
                signature: String::new(),
                pubkey: wallet.address.clone(),
            },
        ];

        let outputs = vec![
            TxOutput {
                value: 100,
                receiver_pk: wallet.address.clone(),
            },
            TxOutput {
                value: 200,
                receiver_pk: wallet.address.clone(),
            },
        ];

        let mut transaction = Transaction::new(inputs, outputs);

        wallet.sign_utxo_based_transaction(&mut transaction);

        assert!(!transaction.id.is_empty());
        assert!(!transaction.inputs[0].signature.is_empty());
        assert!(transaction.inputs[0].signature != transaction.inputs[1].signature);
        assert!(transaction.id == transaction.hash());

        assert!(transaction.verify_signatures());
    }
}
