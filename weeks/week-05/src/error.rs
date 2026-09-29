use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum BtcLibError {
    #[error("malformed data")]
    MalformedData,
    #[error("empty transaction id")]
    EmptyTxId,
    #[error("missing transaction inputs")]
    MissingInputs,
    #[error("missing transaction outputs")]
    MissingOutputs,
    #[error("zero value output")]
    ZeroValueOutput,
    #[error("empty block")]
    EmptyBlock,
    #[error("duplicate transaction id")]
    DuplicateTxId,
    #[error("invalid previous block hash")]
    InvalidPreviousHash,
    #[error("invalid merkle root")]
    InvalidMerkleRoot,
    #[error("missing block")]
    MissingBlock,
    #[error("empty chain")]
    EmptyChain,
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("io error: {0}")]
    Io(String),
}

impl From<std::io::Error> for BtcLibError {
    fn from(error: std::io::Error) -> Self {
        BtcLibError::Io(error.to_string())
    }
}

impl From<serde_json::Error> for BtcLibError {
    fn from(error: serde_json::Error) -> Self {
        BtcLibError::Serialization(error.to_string())
    }
}
