#![allow(unused_variables)]

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const COINBASE_PREVIOUS_OUTPUT: &str = "-";

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmountSummary {
    pub output_count: usize,
    pub total_sats: u64,
    pub spent_sats: u64,
    pub unspent_sats: u64,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum BtcLibError {
    #[error("malformed transaction data")]
    MalformedData,
    #[error("missing transaction")]
    MissingTransaction,
    #[error("empty transaction id")]
    EmptyTxId,
    #[error("missing transaction inputs")]
    MissingInputs,
    #[error("missing transaction outputs")]
    MissingOutputs,
    #[error("zero value output")]
    ZeroValueOutput,
    #[error("empty block")]
    EmptyBlock,
    #[error("duplicate transaction id")]
    DuplicateTxId,
    #[error("invalid hash hex")]
    InvalidHash,
    #[error("io error: {0}")]
    Io(String),
}

pub trait Hashable {
    fn hash_material(&self) -> String;

    fn hash_hex(&self) -> String {
        sha256::digest(self.hash_material())
    }
}

pub trait Validate {
    fn validate(&self) -> Result<(), BtcLibError>;
}

impl From<std::io::Error> for BtcLibError {
    fn from(error: std::io::Error) -> Self {
        BtcLibError::Io(error.to_string())
    }
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

impl Validate for TxOutput {
    fn validate(&self) -> Result<(), BtcLibError> {
        if self.value_sats == 0 {
            return Err(BtcLibError::ZeroValueOutput);
        }
        Ok(())
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
}

impl Hashable for Transaction {
    fn hash_material(&self) -> String {
        let mut material = format!("tx:{}|inputs:", self.txid);
        for input in &self.inputs {
            material.push_str(&format!("{}:{};", input.previous_txid, input.previous_vout));
        }
        material.push_str("|outputs:");
        for output in &self.outputs {
            let status = match output.status {
                TxStatus::Spent => "spent",
                TxStatus::Unspent => "unspent",
            };
            material.push_str(&format!("{}:{}:{};", output.value_sats, output.recipient, status));
        }
        material
    }
}

impl Validate for Transaction {
    fn validate(&self) -> Result<(), BtcLibError> {
        if self.txid.is_empty() {
            return Err(BtcLibError::EmptyTxId);
        }
        if !self.is_coinbase() && self.inputs.is_empty() {
            return Err(BtcLibError::MissingInputs);
        }
        if self.outputs.is_empty() {
            return Err(BtcLibError::MissingOutputs);
        }
        for output in &self.outputs {
            output.validate()?;
        }
        Ok(())
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

    pub fn find_transaction(&self, txid: &str) -> Option<&Transaction> {
        self.transactions.iter().find(|tx| tx.txid == txid)
    }
}

impl Hashable for Block {
    fn hash_material(&self) -> String {
        let mut material = format!(
            "block:{}|prev:{}|height:{}|txs:",
            self.header.block_hash, self.header.previous_block_hash, self.height
        );
        for tx in &self.transactions {
            material.push_str(&format!("{};", tx.txid));
        }
        material
    }
}

impl Validate for Block {
    fn validate(&self) -> Result<(), BtcLibError> {
        if self.transactions.is_empty() {
            return Err(BtcLibError::EmptyBlock);
        }
        let mut seen = HashSet::new();
        for tx in &self.transactions {
            if !seen.insert(tx.txid.as_str()) {
                return Err(BtcLibError::DuplicateTxId);
            }
            tx.validate()?;
        }
        Ok(())
    }
}

/// Parse `spent` or `unspent` into a `TxStatus`.
pub fn parse_status(input: &str) -> Result<TxStatus, BtcLibError> {
    match input.trim().to_lowercase().as_str() {
        "spent" => Ok(TxStatus::Spent),
        "unspent" => Ok(TxStatus::Unspent),
        _ => Err(BtcLibError::MalformedData),
    }
}

/// Parse a previous output reference.
pub fn parse_outpoint(input: &str) -> Result<Option<TxInput>, BtcLibError> {
    let trimmed = input.trim();
    if trimmed == COINBASE_PREVIOUS_OUTPUT {
        return Ok(None);
    }
    let colon_pos = trimmed.find(':').ok_or(BtcLibError::MalformedData)?;
    let txid = trimmed[..colon_pos].trim();
    let vout_str = trimmed[colon_pos + 1..].trim();
    if txid.is_empty() {
        return Err(BtcLibError::MalformedData);
    }
    let vout = vout_str.parse::<u32>().map_err(|_| BtcLibError::MalformedData)?;
    Ok(Some(TxInput::new(txid, vout)))
}

/// Parse a row into the Week 3 transaction model.
pub fn parse_transaction(input: &str) -> Result<Transaction, BtcLibError> {
    let fields: Vec<&str> = input.split(',').collect();
    if fields.len() != 5 {
        return Err(BtcLibError::MalformedData);
    }
    let txid = fields[0].trim();
    let prev_output_str = fields[1].trim();
    let recipient = fields[2].trim();
    let amount_str = fields[3].trim();
    let status_str = fields[4].trim();

    if txid.is_empty() || recipient.is_empty() || amount_str.is_empty() || status_str.is_empty() {
        return Err(BtcLibError::MalformedData);
    }

    let maybe_input = parse_outpoint(prev_output_str)?;

    // coinbase marker requires txid == "coinbase"
    if prev_output_str == COINBASE_PREVIOUS_OUTPUT && txid != "coinbase" {
        return Err(BtcLibError::MalformedData);
    }
    // non-coinbase prev output requires txid != "coinbase"
    if maybe_input.is_some() && txid == "coinbase" {
        return Err(BtcLibError::MalformedData);
    }

    let amount: u64 = amount_str.parse().map_err(|_| BtcLibError::MalformedData)?;
    if amount == 0 {
        return Err(BtcLibError::MalformedData);
    }

    let status = parse_status(status_str)?;

    let output = TxOutput::new(amount, recipient, status);
    let inputs: Vec<TxInput> = maybe_input.into_iter().collect();
    Ok(Transaction::new(txid, inputs, vec![output]))
}

/// Parse every row into a transaction.
pub fn parse_transactions(lines: &[&str]) -> Result<Vec<Transaction>, BtcLibError> {
    let mut result = Vec::new();
    for line in lines {
        result.push(parse_transaction(line)?);
    }
    Ok(result)
}

/// Parse all valid rows and skip malformed rows.
pub fn valid_transactions_only(lines: &[&str]) -> Vec<Transaction> {
    lines
        .iter()
        .filter_map(|line| parse_transaction(line).ok())
        .collect()
}

/// Build and validate a block from parsed transaction rows.
pub fn build_block_from_rows(
    header: BlockHeader,
    rows: &[&str],
    height: u64,
    network: Network,
) -> Result<Block, BtcLibError> {
    let transactions = parse_transactions(rows)?;
    let block = Block::new(header, transactions, height, network);
    block.validate()?;
    Ok(block)
}

/// Validate every item in order.
pub fn validate_all<T: Validate>(items: &[T]) -> Result<(), BtcLibError> {
    for item in items {
        item.validate()?;
    }
    Ok(())
}

/// Return the SHA-256 hex hash for every hashable item, preserving input order.
pub fn hash_all<T: Hashable>(items: &[T]) -> Vec<String> {
    items.iter().map(|item| item.hash_hex()).collect()
}

/// Decode a 64-character SHA-256 hex string into 32 bytes.
pub fn decode_hash_hex(input: &str) -> Result<[u8; 32], BtcLibError> {
    let trimmed = input.trim();
    let bytes = hex::decode(trimmed).map_err(|_| BtcLibError::InvalidHash)?;
    if bytes.len() != 32 {
        return Err(BtcLibError::InvalidHash);
    }
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}

/// Sum unspent output amounts across all transactions.
pub fn total_unspent(transactions: &[Transaction]) -> u64 {
    transactions
        .iter()
        .flat_map(|tx| tx.outputs.iter())
        .filter(|output| output.status == TxStatus::Unspent)
        .map(|output| output.value_sats)
        .sum()
}

/// Return a borrowed transaction with the matching txid, if one exists.
pub fn find_by_txid<'a>(transactions: &'a [Transaction], txid: &str) -> Option<&'a Transaction> {
    transactions.iter().find(|tx| tx.txid == txid)
}

/// Return the matching transaction or `BtcLibError::MissingTransaction`.
pub fn require_transaction<'a>(
    transactions: &'a [Transaction],
    txid: &str,
) -> Result<&'a Transaction, BtcLibError> {
    find_by_txid(transactions, txid).ok_or(BtcLibError::MissingTransaction)
}

/// Build an amount summary from all transaction outputs.
pub fn summarize_amounts(transactions: &[Transaction]) -> AmountSummary {
    let mut output_count = 0;
    let mut total_sats = 0;
    let mut spent_sats = 0;
    let mut unspent_sats = 0;
    for tx in transactions {
        for output in &tx.outputs {
            output_count += 1;
            total_sats += output.value_sats;
            match output.status {
                TxStatus::Spent => spent_sats += output.value_sats,
                TxStatus::Unspent => unspent_sats += output.value_sats,
            }
        }
    }
    AmountSummary {
        output_count,
        total_sats,
        spent_sats,
        unspent_sats,
    }
}
