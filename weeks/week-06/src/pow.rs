use crate::{CandidateBlock, Hashable, MinerError, Transaction};

/// Return true when a hash starts with the configured difficulty prefix.
pub fn hash_meets_difficulty(hash: &str, difficulty_prefix: &str) -> Result<bool, MinerError> {
    if !difficulty_prefix
        .chars()
        .all(|c| c.is_ascii_hexdigit())
    {
        return Err(MinerError::InvalidDifficulty);
    }
    Ok(hash
        .to_lowercase()
        .starts_with(&difficulty_prefix.to_lowercase()))
}

/// Calculate a simple merkle root from transaction hashes.
pub fn calculate_merkle_root(transactions: &[Transaction]) -> Result<String, MinerError> {
    if transactions.is_empty() {
        return Err(MinerError::EmptyCandidate);
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
            next.push(sha256::digest(format!("{left}{right}")));
            i += 2;
        }
        level = next;
    }
    Ok(level.remove(0))
}

/// Build deterministic candidate hash material for a nonce.
pub fn candidate_hash_material(
    candidate: &CandidateBlock,
    nonce: u64,
) -> Result<String, MinerError> {
    let merkle = calculate_merkle_root(&candidate.transactions)?;
    let mut material = format!(
        "candidate:{}|height:{}|merkle:{}|time:{}|nonce:{}|txs:",
        candidate.previous_block_hash,
        candidate.height,
        merkle,
        candidate.timestamp,
        nonce
    );
    for tx in &candidate.transactions {
        material.push_str(&format!("{};", tx.txid));
    }
    Ok(material)
}

/// Hash a candidate block at one nonce.
pub fn hash_candidate(candidate: &CandidateBlock, nonce: u64) -> Result<String, MinerError> {
    let material = candidate_hash_material(candidate, nonce)?;
    Ok(sha256::digest(material))
}
