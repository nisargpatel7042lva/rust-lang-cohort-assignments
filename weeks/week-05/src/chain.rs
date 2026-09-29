use serde::{Deserialize, Serialize};

use crate::{validate_merkle_root, Block, BtcLibError, Network, Transaction, Validate};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blockchain {
    pub network: Network,
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new(network: Network) -> Self {
        Self {
            network,
            blocks: Vec::new(),
        }
    }

    pub fn from_genesis(genesis: Block) -> Result<Self, BtcLibError> {
        genesis.validate()?;
        validate_merkle_root(&genesis)?;
        let mut chain = Self::new(genesis.network);
        chain.blocks.push(genesis);
        Ok(chain)
    }

    pub fn height(&self) -> u64 {
        self.tip().map(|b| b.height).unwrap_or(0)
    }

    pub fn tip(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn tip_hash(&self) -> Option<&str> {
        self.tip().map(|b| b.header.block_hash.as_str())
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), BtcLibError> {
        block.validate()?;
        validate_merkle_root(&block)?;
        if self.blocks.is_empty() {
            if block.height != 0 {
                return Err(BtcLibError::InvalidPreviousHash);
            }
        } else {
            let tip = self.tip().unwrap();
            if block.header.previous_block_hash != tip.header.block_hash {
                return Err(BtcLibError::InvalidPreviousHash);
            }
            if block.height != tip.height + 1 {
                return Err(BtcLibError::InvalidPreviousHash);
            }
        }
        self.blocks.push(block);
        Ok(())
    }

    pub fn find_block_by_hash(&self, block_hash: &str) -> Option<&Block> {
        self.blocks
            .iter()
            .find(|b| b.header.block_hash == block_hash)
    }

    pub fn find_transaction(&self, txid: &str) -> Option<&Transaction> {
        self.blocks
            .iter()
            .find_map(|b| b.find_transaction(txid))
    }

    pub fn total_transactions(&self) -> usize {
        self.blocks.iter().map(|b| b.transaction_count()).sum()
    }

    pub fn validate(&self) -> Result<(), BtcLibError> {
        if self.blocks.is_empty() {
            return Err(BtcLibError::EmptyChain);
        }
        for (i, block) in self.blocks.iter().enumerate() {
            block.validate()?;
            validate_merkle_root(block)?;
            if i > 0 {
                let prev = &self.blocks[i - 1];
                if block.header.previous_block_hash != prev.header.block_hash {
                    return Err(BtcLibError::InvalidPreviousHash);
                }
                if block.height != prev.height + 1 {
                    return Err(BtcLibError::InvalidPreviousHash);
                }
            }
        }
        Ok(())
    }
}

/// Return a lowercase label for the network.
pub fn network_label(network: Network) -> &'static str {
    match network {
        Network::Mainnet => "mainnet",
        Network::Testnet => "testnet",
        Network::Signet => "signet",
        Network::Regtest => "regtest",
    }
}

/// Build a compact chain summary string.
pub fn chain_summary(chain: &Blockchain) -> String {
    format!(
        "network:{}|height:{}|blocks:{}|tip:{}|txs:{}",
        network_label(chain.network),
        chain.height(),
        chain.blocks.len(),
        chain.tip_hash().unwrap_or("none"),
        chain.total_transactions()
    )
}
