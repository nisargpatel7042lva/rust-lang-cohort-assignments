use crate::{Block, NodeError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeRequest {
    Ping,
    Height,
    GetTip,
    GetBlock(String),
    SubmitBlock(Block),
    AddPeer(String),
    GetPeers,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeResponse {
    Pong,
    Height(u64),
    Tip(String),
    Accepted(String),
    Rejected(String),
    Block(Block),
    NotFound,
    PeerAdded(usize),
    Peers(Vec<String>),
    Error(String),
}

/// Parse a block in `<hash>|<previous_hash>|<height>|<payload>` format.
pub fn parse_block(input: &str) -> Result<Block, NodeError> {
    let fields: Vec<&str> = input.splitn(4, '|').collect();
    if fields.len() != 4 {
        return Err(NodeError::MalformedMessage);
    }
    let hash = fields[0].trim();
    let previous_hash = fields[1].trim();
    let height_str = fields[2].trim();
    let payload = fields[3].trim();
    if hash.is_empty() || previous_hash.is_empty() || height_str.is_empty() || payload.is_empty() {
        return Err(NodeError::MalformedMessage);
    }
    let height: u64 = height_str
        .parse()
        .map_err(|_| NodeError::MalformedMessage)?;
    Ok(Block::new(hash, previous_hash, height, payload))
}

/// Parse one text protocol request.
pub fn parse_request(line: &str) -> Result<NodeRequest, NodeError> {
    let trimmed = line.trim_end_matches('\n').trim();
    match trimmed {
        "ping" => return Ok(NodeRequest::Ping),
        "height" => return Ok(NodeRequest::Height),
        "get_tip" => return Ok(NodeRequest::GetTip),
        "get_peers" => return Ok(NodeRequest::GetPeers),
        _ => {}
    }
    if let Some(rest) = trimmed.strip_prefix("get_block ") {
        let arg = rest.trim();
        if arg.is_empty() {
            return Err(NodeError::MalformedMessage);
        }
        return Ok(NodeRequest::GetBlock(arg.to_string()));
    }
    if trimmed == "get_block" {
        return Err(NodeError::MalformedMessage);
    }
    if let Some(rest) = trimmed.strip_prefix("add_peer ") {
        let arg = rest.trim();
        if arg.is_empty() {
            return Err(NodeError::MalformedMessage);
        }
        return Ok(NodeRequest::AddPeer(arg.to_string()));
    }
    if trimmed == "add_peer" {
        return Err(NodeError::MalformedMessage);
    }
    if let Some(rest) = trimmed.strip_prefix("submit_block ") {
        let arg = rest.trim();
        if arg.is_empty() {
            return Err(NodeError::MalformedMessage);
        }
        let block = parse_block(arg)?;
        return Ok(NodeRequest::SubmitBlock(block));
    }
    if trimmed == "submit_block" {
        return Err(NodeError::MalformedMessage);
    }
    Err(NodeError::UnknownCommand)
}

/// Encode a response as one newline-terminated protocol line.
pub fn encode_response(response: &NodeResponse) -> String {
    match response {
        NodeResponse::Pong => "pong\n".to_string(),
        NodeResponse::Height(h) => format!("height {h}\n"),
        NodeResponse::Tip(hash) => format!("tip {hash}\n"),
        NodeResponse::Accepted(hash) => format!("accepted {hash}\n"),
        NodeResponse::Rejected(reason) => format!("rejected {reason}\n"),
        NodeResponse::Block(block) => format!("block {}\n", block.wire_format()),
        NodeResponse::NotFound => "not_found\n".to_string(),
        NodeResponse::PeerAdded(count) => format!("peer_added {count}\n"),
        NodeResponse::Peers(peers) => format!("peers {}\n", peers.join(",")),
        NodeResponse::Error(msg) => format!("error {msg}\n"),
    }
}

/// Parse a response produced by `encode_response`.
pub fn parse_response(line: &str) -> Result<NodeResponse, NodeError> {
    let trimmed = line.trim();
    if trimmed == "pong" {
        return Ok(NodeResponse::Pong);
    }
    if trimmed == "not_found" {
        return Ok(NodeResponse::NotFound);
    }
    if let Some(rest) = trimmed.strip_prefix("height ") {
        let n: u64 = rest
            .trim()
            .parse()
            .map_err(|_| NodeError::MalformedMessage)?;
        return Ok(NodeResponse::Height(n));
    }
    if let Some(rest) = trimmed.strip_prefix("tip ") {
        return Ok(NodeResponse::Tip(rest.trim().to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("accepted ") {
        return Ok(NodeResponse::Accepted(rest.trim().to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("rejected ") {
        return Ok(NodeResponse::Rejected(rest.trim().to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("error ") {
        return Ok(NodeResponse::Error(rest.trim().to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("block ") {
        let block = parse_block(rest.trim())?;
        return Ok(NodeResponse::Block(block));
    }
    if let Some(rest) = trimmed.strip_prefix("peers ") {
        let peers: Vec<String> = rest
            .trim()
            .split(',')
            .map(|s| s.to_string())
            .collect();
        return Ok(NodeResponse::Peers(peers));
    }
    // Check for known prefixes with malformed content
    if trimmed.starts_with("height") || trimmed.starts_with("block") {
        return Err(NodeError::MalformedMessage);
    }
    Err(NodeError::UnknownCommand)
}
