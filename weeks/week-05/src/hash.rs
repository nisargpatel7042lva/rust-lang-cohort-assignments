use crate::{Block, BtcLibError, Transaction};

pub trait Hashable {
    fn hash_material(&self) -> String;

    fn hash_hex(&self) -> String {
        sha256::digest(self.hash_material())
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
                crate::TxStatus::Spent => "spent",
                crate::TxStatus::Unspent => "unspent",
            };
            material.push_str(&format!("{}:{}:{};", output.value_sats, output.recipient, status));
        }
        material
    }
}

impl Hashable for Block {
    fn hash_material(&self) -> String {
        let mut material = format!(
            "block:{}|prev:{}|merkle:{}|height:{}|txs:",
            self.header.block_hash,
            self.header.previous_block_hash,
            self.header.merkle_root,
            self.height
        );
        for tx in &self.transactions {
            material.push_str(&format!("{};", tx.txid));
        }
        material
    }
}

/// Hash two child hashes into their parent merkle node.
pub fn pair_hash(left: &str, right: &str) -> String {
    sha256::digest(format!("{left}{right}"))
}

/// Calculate a simple merkle root from transaction hashes.
pub fn calculate_merkle_root(transactions: &[Transaction]) -> Result<String, BtcLibError> {
    if transactions.is_empty() {
        return Err(BtcLibError::EmptyBlock);
    }
    let mut level: Vec<String> = transactions.iter().map(|tx| tx.hash_hex()).collect();
    while level.len() > 1 {
        let mut next = Vec::new();
        let mut i = 0;
        while i < level.len() {
            let left = &level[i];
            let right = if i + 1 < level.len() {
                &level[i + 1]
            } else {
                &level[i]
            };
            next.push(pair_hash(left, right));
            i += 2;
        }
        level = next;
    }
    Ok(level.remove(0))
}

/// Validate that the block header stores the merkle root for its transactions.
pub fn validate_merkle_root(block: &Block) -> Result<(), BtcLibError> {
    let expected = calculate_merkle_root(&block.transactions)?;
    if expected == block.header.merkle_root {
        Ok(())
    } else {
        Err(BtcLibError::InvalidMerkleRoot)
    }
}
