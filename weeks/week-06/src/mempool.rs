use std::collections::BTreeMap;

use crate::{MinerError, Transaction};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mempool {
    pub transactions: BTreeMap<String, Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            transactions: BTreeMap::new(),
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), MinerError> {
        if self.transactions.contains_key(&transaction.txid) {
            return Err(MinerError::DuplicateMempoolTransaction(
                transaction.txid.clone(),
            ));
        }
        self.transactions.insert(transaction.txid.clone(), transaction);
        Ok(())
    }

    pub fn remove_transaction(&mut self, txid: &str) -> Result<Transaction, MinerError> {
        self.transactions
            .remove(txid)
            .ok_or_else(|| MinerError::TransactionNotFound(txid.to_string()))
    }

    pub fn ordered_transactions(&self) -> Vec<Transaction> {
        self.transactions.values().cloned().collect()
    }

    pub fn drain_for_candidate(&mut self, limit: usize) -> Vec<Transaction> {
        if limit == 0 {
            return Vec::new();
        }
        let keys: Vec<String> = self.transactions.keys().take(limit).cloned().collect();
        keys.into_iter()
            .filter_map(|k| self.transactions.remove(&k))
            .collect()
    }

    pub fn total_output_value(&self) -> u64 {
        self.transactions
            .values()
            .map(|tx| tx.total_output_value())
            .sum()
    }
}
