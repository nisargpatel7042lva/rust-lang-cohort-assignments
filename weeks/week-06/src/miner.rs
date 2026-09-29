use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::{
    calculate_merkle_root, hash_candidate, hash_meets_difficulty, Block, BlockHeader,
    CandidateBlock, Mempool, MinerError, Transaction,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MiningConfig {
    pub difficulty_prefix: String,
    pub start_nonce: u64,
    pub max_nonce: u64,
    pub worker_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinedNonce {
    pub nonce: u64,
    pub hash: String,
    pub attempts: u64,
    pub worker_id: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MiningReport {
    pub block: Block,
    pub nonce: u64,
    pub hash: String,
    pub attempts: u64,
    pub worker_count: usize,
}

/// Build a candidate from a mempool and remove selected transactions.
pub fn build_candidate_from_mempool(
    mempool: &mut Mempool,
    previous_block_hash: &str,
    height: u64,
    coinbase_recipient: &str,
    reward_sats: u64,
    timestamp: u64,
    max_mempool_txs: usize,
) -> CandidateBlock {
    let coinbase = Transaction::coinbase(
        &format!("coinbase-{height}"),
        coinbase_recipient,
        reward_sats,
    );
    let mut transactions = vec![coinbase];
    transactions.extend(mempool.drain_for_candidate(max_mempool_txs));
    CandidateBlock {
        previous_block_hash: previous_block_hash.to_string(),
        height,
        transactions,
        coinbase_recipient: coinbase_recipient.to_string(),
        reward_sats,
        timestamp,
    }
}

/// Build a concrete block from a candidate and nonce.
pub fn build_candidate_block(
    candidate: &CandidateBlock,
    nonce: u64,
    difficulty_prefix: &str,
) -> Result<Block, MinerError> {
    if candidate.transactions.is_empty() {
        return Err(MinerError::EmptyCandidate);
    }
    let merkle_root = calculate_merkle_root(&candidate.transactions)?;
    let header = BlockHeader {
        previous_block_hash: candidate.previous_block_hash.clone(),
        merkle_root,
        timestamp: candidate.timestamp,
        nonce,
        difficulty_prefix: difficulty_prefix.to_string(),
    };
    Ok(Block {
        header,
        height: candidate.height,
        transactions: candidate.transactions.clone(),
    })
}

/// Split an inclusive nonce range across workers.
pub fn split_nonce_ranges(
    start_nonce: u64,
    max_nonce: u64,
    worker_count: usize,
) -> Result<Vec<(u64, u64)>, MinerError> {
    if worker_count == 0 || start_nonce > max_nonce {
        return Err(MinerError::InvalidDifficulty);
    }
    let total = max_nonce - start_nonce + 1;
    let actual_workers = worker_count.min(total as usize);
    let base = total / actual_workers as u64;
    let remainder = total % actual_workers as u64;
    let mut ranges = Vec::new();
    let mut current = start_nonce;
    for i in 0..actual_workers {
        let extra = if (i as u64) < remainder { 1 } else { 0 };
        let end = current + base + extra - 1;
        ranges.push((current, end));
        current = end + 1;
    }
    Ok(ranges)
}

/// Search one inclusive nonce range.
pub fn mine_range(
    candidate: &CandidateBlock,
    difficulty_prefix: &str,
    start_nonce: u64,
    end_nonce: u64,
    worker_id: usize,
) -> Result<Option<MinedNonce>, MinerError> {
    if start_nonce > end_nonce {
        return Err(MinerError::InvalidDifficulty);
    }
    let mut attempts: u64 = 0;
    let mut nonce = start_nonce;
    loop {
        let hash = hash_candidate(candidate, nonce)?;
        attempts += 1;
        if hash_meets_difficulty(&hash, difficulty_prefix)? {
            return Ok(Some(MinedNonce {
                nonce,
                hash,
                attempts,
                worker_id,
            }));
        }
        if nonce == end_nonce {
            break;
        }
        nonce += 1;
    }
    Ok(None)
}

/// Mine using a single worker over the configured nonce range.
pub fn mine_single_threaded(
    candidate: &CandidateBlock,
    config: &MiningConfig,
) -> Result<MiningReport, MinerError> {
    let result = mine_range(
        candidate,
        &config.difficulty_prefix,
        config.start_nonce,
        config.max_nonce,
        0,
    )?;
    match result {
        Some(mined) => {
            let block = build_candidate_block(candidate, mined.nonce, &config.difficulty_prefix)?;
            Ok(MiningReport {
                block,
                nonce: mined.nonce,
                hash: mined.hash,
                attempts: mined.attempts,
                worker_count: 1,
            })
        }
        None => Err(MinerError::NoSolution),
    }
}

/// Mine using several workers and return the first solution reported.
pub fn mine_multi_threaded(
    candidate: CandidateBlock,
    config: MiningConfig,
) -> Result<MiningReport, MinerError> {
    let ranges = split_nonce_ranges(config.start_nonce, config.max_nonce, config.worker_count)?;
    let worker_count = ranges.len();
    let (tx, rx) = std::sync::mpsc::channel::<MinedNonce>();
    let cancelled = Arc::new(AtomicBool::new(false));
    let candidate = Arc::new(candidate);
    let difficulty_prefix = Arc::new(config.difficulty_prefix.clone());

    let mut handles = Vec::new();
    for (worker_id, (start, end)) in ranges.into_iter().enumerate() {
        let tx = tx.clone();
        let cancelled = Arc::clone(&cancelled);
        let candidate = Arc::clone(&candidate);
        let difficulty_prefix = Arc::clone(&difficulty_prefix);

        let handle = std::thread::spawn(move || {
            if cancelled.load(Ordering::Relaxed) {
                return;
            }
            let mut attempts: u64 = 0;
            let mut nonce = start;
            loop {
                if cancelled.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(hash) = hash_candidate(&candidate, nonce) {
                    attempts += 1;
                    if let Ok(true) = hash_meets_difficulty(&hash, &difficulty_prefix) {
                        cancelled.store(true, Ordering::Relaxed);
                        let _ = tx.send(MinedNonce {
                            nonce,
                            hash,
                            attempts,
                            worker_id,
                        });
                        break;
                    }
                }
                if nonce == end {
                    break;
                }
                nonce += 1;
            }
        });
        handles.push(handle);
    }
    drop(tx);

    let result = rx.recv().ok();

    for handle in handles {
        let _ = handle.join();
    }

    match result {
        Some(mined) => {
            let block =
                build_candidate_block(&candidate, mined.nonce, &config.difficulty_prefix)?;
            Ok(MiningReport {
                block,
                nonce: mined.nonce,
                hash: mined.hash,
                attempts: mined.attempts,
                worker_count,
            })
        }
        None => Err(MinerError::NoSolution),
    }
}

/// Build a compact mining progress line.
pub fn progress_line(report: &MiningReport) -> String {
    format!(
        "workers:{}|nonce:{}|attempts:{}|hash:{}",
        report.worker_count, report.nonce, report.attempts, report.hash
    )
}
