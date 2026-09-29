use serde::{Deserialize, Serialize};

use crate::NodeError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub hash: String,
    pub previous_hash: String,
    pub height: u64,
    pub payload: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerInfo {
    pub address: String,
    pub last_seen_height: u64,
}

impl Block {
    pub fn new(hash: &str, previous_hash: &str, height: u64, payload: &str) -> Self {
        Self {
            hash: hash.to_string(),
            previous_hash: previous_hash.to_string(),
            height,
            payload: payload.to_string(),
        }
    }

    pub fn genesis() -> Self {
        Self::new("genesis", "0", 0, "genesis")
    }

    pub fn wire_format(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.hash, self.previous_hash, self.height, self.payload
        )
    }

    pub fn validate_against_tip(&self, tip: Option<&Block>) -> Result<(), NodeError> {
        if self.hash.is_empty() || self.payload.is_empty() {
            return Err(NodeError::InvalidBlock);
        }
        match tip {
            None => {
                if self.height != 0 || self.previous_hash != "0" {
                    return Err(NodeError::InvalidBlock);
                }
            }
            Some(tip) => {
                if self.previous_hash != tip.hash || self.height != tip.height + 1 {
                    return Err(NodeError::InvalidBlock);
                }
            }
        }
        Ok(())
    }
}
