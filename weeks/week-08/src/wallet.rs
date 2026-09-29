use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    derive_wallet_txid, OutPoint, Transaction, TxInput, TxOutput, WalletError, WalletUtxo,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wallet {
    pub owner: String,
    pub utxos: BTreeMap<OutPoint, WalletUtxo>,
    pub pending: Vec<Transaction>,
    pub history: Vec<Transaction>,
}

impl Wallet {
    pub fn new(owner: &str) -> Self {
        Self {
            owner: owner.to_string(),
            utxos: BTreeMap::new(),
            pending: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn import_utxo(&mut self, utxo: WalletUtxo) {
        self.utxos.insert(utxo.outpoint.clone(), utxo);
    }

    pub fn confirmed_balance(&self) -> u64 {
        self.utxos
            .values()
            .filter(|utxo| utxo.owner == self.owner && utxo.confirmations > 0)
            .map(|utxo| utxo.value_sats)
            .sum()
    }

    pub fn pending_incoming_balance(&self) -> u64 {
        self.pending
            .iter()
            .flat_map(|tx| tx.outputs.iter())
            .filter(|output| output.recipient == self.owner)
            .map(|output| output.value_sats)
            .sum()
    }

    pub fn available_utxos(&self) -> Vec<WalletUtxo> {
        self.utxos
            .values()
            .filter(|utxo| utxo.owner == self.owner && utxo.confirmations > 0)
            .cloned()
            .collect()
    }

    pub fn select_utxos(
        &self,
        amount_sats: u64,
        fee_sats: u64,
    ) -> Result<Vec<WalletUtxo>, WalletError> {
        if amount_sats == 0 {
            return Err(WalletError::InvalidAmount);
        }
        let target = amount_sats + fee_sats;
        let mut selected = Vec::new();
        let mut total: u64 = 0;
        for utxo in self.available_utxos() {
            selected.push(utxo.clone());
            total += utxo.value_sats;
            if total >= target {
                return Ok(selected);
            }
        }
        Err(WalletError::InsufficientFunds)
    }

    pub fn build_transaction(
        &self,
        recipient: &str,
        amount_sats: u64,
        fee_sats: u64,
    ) -> Result<Transaction, WalletError> {
        if recipient.is_empty() || amount_sats == 0 {
            return Err(WalletError::InvalidAmount);
        }
        let selected = self.select_utxos(amount_sats, fee_sats)?;
        let inputs: Vec<TxInput> = selected
            .iter()
            .map(|utxo| TxInput::new(&utxo.outpoint.txid, utxo.outpoint.vout))
            .collect();
        let selected_total: u64 = selected.iter().map(|u| u.value_sats).sum();
        let change = selected_total - amount_sats - fee_sats;
        let mut outputs = vec![TxOutput::new(amount_sats, recipient)];
        if change > 0 {
            outputs.push(TxOutput::new(change, &self.owner));
        }
        let txid = derive_wallet_txid(&self.owner, recipient, amount_sats, fee_sats, &selected);
        Ok(Transaction::new(&txid, inputs, outputs, fee_sats))
    }

    pub fn record_pending(&mut self, transaction: Transaction) -> Result<(), WalletError> {
        // Validate all inputs exist before mutating
        for input in &transaction.inputs {
            if !self.utxos.contains_key(&input.previous_output) {
                return Err(WalletError::MissingUtxo(input.previous_output.label()));
            }
        }
        for input in &transaction.inputs {
            self.utxos.remove(&input.previous_output);
        }
        self.pending.push(transaction.clone());
        self.history.push(transaction);
        Ok(())
    }

    pub fn apply_confirmed_transaction(&mut self, transaction: Transaction) {
        self.pending.retain(|tx| tx.txid != transaction.txid);
        for (vout, output) in transaction.outputs.iter().enumerate() {
            if output.recipient == self.owner {
                let utxo = WalletUtxo::new(
                    &transaction.txid,
                    vout as u32,
                    output.value_sats,
                    &self.owner,
                    6,
                );
                self.import_utxo(utxo);
            }
        }
        if !self.history.iter().any(|tx| tx.txid == transaction.txid) {
            self.history.push(transaction);
        }
    }

    pub fn history_lines(&self) -> Vec<String> {
        self.history
            .iter()
            .map(|tx| {
                format!(
                    "{}|outputs:{}|fee:{}",
                    tx.txid,
                    tx.total_output_value(),
                    tx.fee_sats
                )
            })
            .collect()
    }
}

/// Return a compact wallet summary.
pub fn wallet_summary(wallet: &Wallet) -> String {
    format!(
        "owner:{}|confirmed:{}|pending_in:{}|pending_txs:{}|history:{}",
        wallet.owner,
        wallet.confirmed_balance(),
        wallet.pending_incoming_balance(),
        wallet.pending.len(),
        wallet.history.len()
    )
}
