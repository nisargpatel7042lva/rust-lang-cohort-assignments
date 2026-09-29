use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Block, MinerError, OutPoint, Transaction, TxInput, TxOutput, Utxo};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct UtxoSet {
    pub entries: BTreeMap<OutPoint, Utxo>,
}

impl UtxoSet {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn insert_output(
        &mut self,
        txid: &str,
        vout: u32,
        output: &TxOutput,
    ) -> Result<(), MinerError> {
        let outpoint = OutPoint {
            txid: txid.to_string(),
            vout,
        };
        if self.entries.contains_key(&outpoint) {
            return Err(MinerError::DuplicateUtxo(crate::outpoint_label(&outpoint)));
        }
        let utxo = Utxo {
            outpoint: outpoint.clone(),
            value_sats: output.value_sats,
            recipient: output.recipient.clone(),
        };
        self.entries.insert(outpoint, utxo);
        Ok(())
    }

    pub fn get(&self, outpoint: &OutPoint) -> Option<&Utxo> {
        self.entries.get(outpoint)
    }

    pub fn spend_input(&mut self, input: &TxInput) -> Result<Utxo, MinerError> {
        let outpoint = input.outpoint();
        self.entries
            .remove(&outpoint)
            .ok_or_else(|| MinerError::MissingUtxo(crate::outpoint_label(&outpoint)))
    }

    pub fn apply_transaction(&mut self, transaction: &Transaction) -> Result<(), MinerError> {
        if !transaction.is_coinbase() {
            // Validate all inputs exist before mutating
            for input in &transaction.inputs {
                let outpoint = input.outpoint();
                if !self.entries.contains_key(&outpoint) {
                    return Err(MinerError::MissingUtxo(crate::outpoint_label(&outpoint)));
                }
            }
            // Now spend them
            for input in &transaction.inputs {
                self.spend_input(input)?;
            }
        }
        for (vout, output) in transaction.outputs.iter().enumerate() {
            self.insert_output(&transaction.txid, vout as u32, output)?;
        }
        Ok(())
    }

    pub fn apply_transactions(&mut self, transactions: &[Transaction]) -> Result<(), MinerError> {
        for tx in transactions {
            self.apply_transaction(tx)?;
        }
        Ok(())
    }

    pub fn apply_block(&mut self, block: &Block) -> Result<(), MinerError> {
        self.apply_transactions(&block.transactions)
    }

    pub fn total_for_recipient(&self, recipient: &str) -> u64 {
        self.entries
            .values()
            .filter(|utxo| utxo.recipient == recipient)
            .map(|utxo| utxo.value_sats)
            .sum()
    }
}

/// Convert an outpoint into `txid:vout`.
pub fn outpoint_label(outpoint: &OutPoint) -> String {
    format!("{}:{}", outpoint.txid, outpoint.vout)
}
