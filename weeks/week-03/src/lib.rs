#![allow(unused_variables)]

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Testnet,
    Signet,
    Regtest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    Spent,
    Unspent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationError {
    EmptyTxId,
    MissingInputs,
    MissingOutputs,
    ZeroValueOutput,
    EmptyBlock,
    DuplicateTxId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxInput {
    pub previous_txid: String,
    pub previous_vout: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOutput {
    pub value_sats: u64,
    pub unique_id: Uuid,
    pub recipient: String,
    pub status: TxStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub block_hash: String,
    pub previous_block_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
    pub nonce: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub height: u64,
    pub network: Network,
}

pub trait Identifiable {
    /// Return the stable identifier for this value.
    fn id(&self) -> &str;
}

impl TxInput {
    pub fn new(previous_txid: &str, previous_vout: u32) -> Self {
        Self {
            previous_txid: previous_txid.to_string(),
            previous_vout,
        }
    }
}

impl TxOutput {
    pub fn new(value_sats: u64, recipient: &str, status: TxStatus) -> Self {
        Self {
            value_sats,
            unique_id: Uuid::new_v4(),
            recipient: recipient.to_string(),
            status,
        }
    }

    pub fn is_unspent(&self) -> bool {
        self.status == TxStatus::Unspent
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

    pub fn is_coinbase(&self) -> bool {
        self.txid == "coinbase" && self.inputs.is_empty()
    }

    pub fn total_output_value(&self) -> u64 {
        self.outputs.iter().map(|o| o.value_sats).sum()
    }

    pub fn unspent_output_count(&self) -> usize {
        self.outputs
            .iter()
            .filter(|o| o.status == TxStatus::Unspent)
            .count()
    }

    pub fn spent_output_count(&self) -> usize {
        self.outputs
            .iter()
            .filter(|o| o.status == TxStatus::Spent)
            .count()
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.txid.is_empty() {
            return Err(ValidationError::EmptyTxId);
        }
        if !self.is_coinbase() && self.inputs.is_empty() {
            return Err(ValidationError::MissingInputs);
        }
        if self.outputs.is_empty() {
            return Err(ValidationError::MissingOutputs);
        }
        for output in &self.outputs {
            if output.value_sats == 0 {
                return Err(ValidationError::ZeroValueOutput);
            }
        }
        Ok(())
    }
}

impl Identifiable for Transaction {
    fn id(&self) -> &str {
        self.txid.as_str()
    }
}

impl BlockHeader {
    pub fn new(
        block_hash: &str,
        previous_block_hash: &str,
        merkle_root: &str,
        timestamp: u64,
        nonce: u64,
    ) -> Self {
        Self {
            block_hash: block_hash.to_string(),
            previous_block_hash: previous_block_hash.to_string(),
            merkle_root: merkle_root.to_string(),
            timestamp,
            nonce,
        }
    }
}

impl Block {
    pub fn new(
        header: BlockHeader,
        transactions: Vec<Transaction>,
        height: u64,
        network: Network,
    ) -> Self {
        Self {
            header,
            transactions,
            height,
            network,
        }
    }

    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    pub fn total_output_value(&self) -> u64 {
        self.transactions
            .iter()
            .map(Transaction::total_output_value)
            .sum()
    }

    pub fn coinbase_transaction(&self) -> Option<&Transaction> {
        self.transactions.iter().find(|tx| tx.is_coinbase())
    }

    pub fn find_transaction(&self, txid: &str) -> Option<&Transaction> {
        self.transactions.iter().find(|tx| tx.txid == txid)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.transactions.is_empty() {
            return Err(ValidationError::EmptyBlock);
        }
        let mut seen = HashSet::new();
        for tx in &self.transactions {
            if !seen.insert(tx.txid.as_str()) {
                return Err(ValidationError::DuplicateTxId);
            }
            tx.validate()?;
        }
        Ok(())
    }
}

impl Identifiable for Block {
    fn id(&self) -> &str {
        self.header.block_hash.as_str()
    }
}

/// Return the Bitcoin network magic value for a network.
pub fn network_magic(network: Network) -> u32 {
    match network {
        Network::Mainnet => 0xD9B4BEF9,
        Network::Testnet => 0x0709110B,
        Network::Signet => 0x40CF030A,
        Network::Regtest => 0xDAB5BFFA,
    }
}

/// Convert a known network magic value back to a `Network`.
///
/// Return `None` for unknown magic values.
pub fn network_from_magic(magic: u32) -> Option<Network> {
    match magic {
        0xD9B4BEF9 => Some(Network::Mainnet),
        0x0709110B => Some(Network::Testnet),
        0x40CF030A => Some(Network::Signet),
        0xDAB5BFFA => Some(Network::Regtest),
        _ => None,
    }
}

/// Count unspent outputs across all transactions.
pub fn count_unspent_outputs(transactions: &[Transaction]) -> usize {
    transactions
        .iter()
        .map(Transaction::unspent_output_count)
        .sum()
}

/// Sum output values whose recipient exactly matches `recipient`.
pub fn total_value_for_recipient(transactions: &[Transaction], recipient: &str) -> u64 {
    transactions
        .iter()
        .flat_map(|tx| tx.outputs.iter())
        .filter(|output| output.recipient == recipient)
        .map(|output| output.value_sats)
        .sum()
}

/// Compare two values through the `Identifiable` trait.
pub fn have_same_id<T: Identifiable, U: Identifiable>(left: &T, right: &U) -> bool {
    left.id() == right.id()
}

/// Collect ids from dynamic trait objects into owned strings.
pub fn collect_ids(items: &[Box<dyn Identifiable>]) -> Vec<String> {
    items.iter().map(|item| item.id().to_string()).collect()
}
