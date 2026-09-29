use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum NodeError {
    #[error("malformed message")]
    MalformedMessage,
    #[error("unknown command")]
    UnknownCommand,
    #[error("invalid block")]
    InvalidBlock,
    #[error("block rejected: {0}")]
    BlockRejected(String),
    #[error("missing block")]
    MissingBlock,
    #[error("channel closed")]
    ChannelClosed,
    #[error("io error: {0}")]
    Io(String),
}

impl From<std::io::Error> for NodeError {
    fn from(error: std::io::Error) -> Self {
        NodeError::Io(error.to_string())
    }
}
