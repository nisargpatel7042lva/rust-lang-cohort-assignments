use crate::{WalletError, WalletUtxo};

/// Derive a deterministic txid for a wallet-created transaction.
pub fn derive_wallet_txid(
    owner: &str,
    recipient: &str,
    amount_sats: u64,
    fee_sats: u64,
    inputs: &[WalletUtxo],
) -> String {
    let mut material = format!(
        "wallet-tx:{owner}|to:{recipient}|amount:{amount_sats}|fee:{fee_sats}|inputs:"
    );
    for input in inputs {
        material.push_str(&format!("{}:{};", input.outpoint.txid, input.outpoint.vout));
    }
    sha256::digest(material)
}

/// Calculate fee rate as sats per virtual byte.
pub fn fee_rate_sats_per_vbyte(fee_sats: u64, vbytes: u64) -> Result<f64, WalletError> {
    if vbytes == 0 {
        return Err(WalletError::InvalidAmount);
    }
    Ok(fee_sats as f64 / vbytes as f64)
}

/// Estimate a simple transaction weight in virtual bytes.
///
/// Formula: `10 + inputs * 68 + outputs * 31`
pub fn estimate_transaction_vbytes(input_count: usize, output_count: usize) -> u64 {
    10 + input_count as u64 * 68 + output_count as u64 * 31
}

/// Estimate the fee for a transaction shape.
pub fn estimate_fee_sats(input_count: usize, output_count: usize, sats_per_vbyte: u64) -> u64 {
    estimate_transaction_vbytes(input_count, output_count) * sats_per_vbyte
}
