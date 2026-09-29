use std::collections::BTreeMap;

use crate::{Block, NodeError, PeerInfo};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeState {
    pub chain: Vec<Block>,
    pub peers: BTreeMap<String, PeerInfo>,
}

impl NodeState {
    pub fn new(genesis: Block) -> Result<Self, NodeError> {
        genesis.validate_against_tip(None)?;
        Ok(Self {
            chain: vec![genesis],
            peers: BTreeMap::new(),
        })
    }

    pub fn height(&self) -> u64 {
        self.chain.last().map(|b| b.height).unwrap_or(0)
    }

    pub fn tip_hash(&self) -> Option<&str> {
        self.chain.last().map(|b| b.hash.as_str())
    }

    pub fn add_peer(&mut self, address: &str) -> usize {
        let height = self.height();
        self.peers.insert(
            address.to_string(),
            PeerInfo {
                address: address.to_string(),
                last_seen_height: height,
            },
        );
        self.peers.len()
    }

    pub fn peer_addresses(&self) -> Vec<String> {
        self.peers.keys().cloned().collect()
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), NodeError> {
        block.validate_against_tip(self.chain.last())?;
        self.chain.push(block);
        Ok(())
    }

    pub fn get_block(&self, hash: &str) -> Option<&Block> {
        self.chain.iter().find(|b| b.hash == hash)
    }
}
