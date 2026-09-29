use crate::{NodeStatus, Transaction, WalletError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeClient {
    pub accepted_transactions: Vec<Transaction>,
    pub height: u64,
    pub tip_hash: String,
    pub history: Vec<Transaction>,
    pub reject_next: Option<String>,
}

impl NodeClient {
    pub fn new(height: u64, tip_hash: &str) -> Self {
        Self {
            accepted_transactions: Vec::new(),
            height,
            tip_hash: tip_hash.to_string(),
            history: Vec::new(),
            reject_next: None,
        }
    }

    pub async fn submit_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<String, WalletError> {
        if let Some(reason) = self.reject_next.take() {
            return Err(WalletError::NodeRejected(reason));
        }
        let txid = transaction.txid.clone();
        self.history.push(transaction.clone());
        self.accepted_transactions.push(transaction);
        Ok(txid)
    }

    pub async fn status(&self) -> Result<NodeStatus, WalletError> {
        Ok(NodeStatus {
            height: self.height,
            tip_hash: self.tip_hash.clone(),
        })
    }

    pub async fn wallet_history(&self, owner: &str) -> Result<Vec<Transaction>, WalletError> {
        let result = self
            .history
            .iter()
            .filter(|tx| tx.pays_owner(owner))
            .cloned()
            .collect();
        Ok(result)
    }
}
