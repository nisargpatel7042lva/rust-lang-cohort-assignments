use serde::{Deserialize, Serialize};

use crate::MinerError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxInput {
    pub previous_txid: String,
    pub previous_vout: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOutput {
    pub value_sats: u64,
    pub recipient: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: String,
    pub vout: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value_sats: u64,
    pub recipient: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub previous_block_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub difficulty_prefix: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub height: u64,
    pub transactions: Vec<Transaction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateBlock {
    pub previous_block_hash: String,
    pub height: u64,
    pub transactions: Vec<Transaction>,
    pub coinbase_recipient: String,
    pub reward_sats: u64,
    pub timestamp: u64,
}

pub trait Hashable {
    fn hash_material(&self) -> String;

    fn hash_hex(&self) -> String {
        sha256::digest(self.hash_material())
    }
}

impl TxInput {
    pub fn new(previous_txid: &str, previous_vout: u32) -> Self {
        Self {
            previous_txid: previous_txid.to_string(),
            previous_vout,
        }
    }

    pub fn outpoint(&self) -> OutPoint {
        OutPoint {
            txid: self.previous_txid.clone(),
            vout: self.previous_vout,
        }
    }
}

impl TxOutput {
    pub fn new(value_sats: u64, recipient: &str) -> Self {
        Self {
            value_sats,
            recipient: recipient.to_string(),
        }
    }
}

impl Transaction {
    pub fn new(txid: &str, inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
        Self {
            txid: txid.to_string(),
            inputs,
            outputs,
        }
    }

    pub fn coinbase(txid: &str, recipient: &str, reward_sats: u64) -> Self {
        Self::new(txid, vec![], vec![TxOutput::new(reward_sats, recipient)])
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.is_empty()
    }

    pub fn total_output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value_sats).sum()
    }

    pub fn fee_from_utxos<F>(&self, mut lookup: F) -> Result<u64, MinerError>
    where
        F: FnMut(&OutPoint) -> Option<u64>,
    {
        if self.is_coinbase() {
            return Ok(0);
        }
        let mut input_total: u64 = 0;
        for input in &self.inputs {
            let outpoint = input.outpoint();
            let label = crate::outpoint_label(&outpoint);
            let value = lookup(&outpoint).ok_or(MinerError::MissingUtxo(label))?;
            input_total += value;
        }
        let output_total = self.total_output_value();
        if output_total > input_total {
            return Err(MinerError::InvalidSpend(self.txid.clone()));
        }
        Ok(input_total - output_total)
    }
}

impl Hashable for Transaction {
    fn hash_material(&self) -> String {
        let mut material = format!("tx:{}|inputs:", self.txid);
        for input in &self.inputs {
            material.push_str(&format!("{}:{};", input.previous_txid, input.previous_vout));
        }
        material.push_str("|outputs:");
        for output in &self.outputs {
            material.push_str(&format!("{}:{};", output.value_sats, output.recipient));
        }
        material
    }
}

impl Hashable for Block {
    fn hash_material(&self) -> String {
        let mut material = format!(
            "block:{}|height:{}|merkle:{}|time:{}|nonce:{}|txs:",
            self.header.previous_block_hash,
            self.height,
            self.header.merkle_root,
            self.header.timestamp,
            self.header.nonce
        );
        for tx in &self.transactions {
            material.push_str(&format!("{};", tx.txid));
        }
        material
    }
}
