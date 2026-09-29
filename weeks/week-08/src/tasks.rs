use crate::{EventLog, NodeClient, NodeStatus, Transaction, Wallet, WalletError};

/// Submit a wallet transaction and update wallet/log state on success.
pub async fn submit_wallet_transaction(
    wallet: &mut Wallet,
    client: &mut NodeClient,
    transaction: Transaction,
    log: &mut EventLog,
) -> Result<String, WalletError> {
    log.record("submit", &format!("submitting {}", transaction.txid));
    match client.submit_transaction(transaction.clone()).await {
        Ok(txid) => {
            wallet.record_pending(transaction)?;
            log.record("accepted", &txid);
            Ok(txid)
        }
        Err(err) => {
            log.record("rejected", &err.to_string());
            Err(err)
        }
    }
}

/// Fetch node status and log the sync.
pub async fn sync_wallet_from_node(
    wallet: &mut Wallet,
    client: &NodeClient,
    log: &mut EventLog,
) -> Result<NodeStatus, WalletError> {
    let status = client.status().await?;
    let history = client.wallet_history(&wallet.owner).await?;
    for tx in history {
        wallet.apply_confirmed_transaction(tx);
    }
    log.record(
        "sync",
        &format!("height:{} tip:{}", status.height, status.tip_hash),
    );
    Ok(status)
}

/// Build and submit a transaction in one flow.
pub async fn build_send_and_submit(
    wallet: &mut Wallet,
    client: &mut NodeClient,
    recipient: &str,
    amount_sats: u64,
    fee_sats: u64,
    log: &mut EventLog,
) -> Result<String, WalletError> {
    let transaction = wallet.build_transaction(recipient, amount_sats, fee_sats)?;
    submit_wallet_transaction(wallet, client, transaction, log).await
}
