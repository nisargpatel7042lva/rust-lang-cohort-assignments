use std::fs;
use std::path::Path;

use crate::{Block, Blockchain, BtcLibError, Validate};

/// Save a single block as pretty JSON.
pub fn save_block_to_file<P: AsRef<Path>>(block: &Block, path: P) -> Result<(), BtcLibError> {
    block.validate()?;
    let json = serde_json::to_string_pretty(block)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load a single block from JSON and validate it.
pub fn load_block_from_file<P: AsRef<Path>>(path: P) -> Result<Block, BtcLibError> {
    let content = fs::read_to_string(path)?;
    let block: Block = serde_json::from_str(&content)?;
    block.validate()?;
    Ok(block)
}

/// Save a full blockchain snapshot as pretty JSON.
pub fn save_chain_to_file<P: AsRef<Path>>(chain: &Blockchain, path: P) -> Result<(), BtcLibError> {
    chain.validate()?;
    let json = serde_json::to_string_pretty(chain)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load a full blockchain snapshot from JSON and validate it.
pub fn load_chain_from_file<P: AsRef<Path>>(path: P) -> Result<Blockchain, BtcLibError> {
    let content = fs::read_to_string(path)?;
    let chain: Blockchain = serde_json::from_str(&content)?;
    chain.validate()?;
    Ok(chain)
}
