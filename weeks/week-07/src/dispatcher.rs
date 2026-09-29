use tokio::sync::{mpsc, oneshot};

use crate::{NodeError, NodeRequest, NodeResponse, NodeState};

pub struct NodeCommand {
    pub request: NodeRequest,
    pub response: oneshot::Sender<NodeResponse>,
}

/// Apply one request to node state and return a response.
pub fn handle_request(state: &mut NodeState, request: NodeRequest) -> NodeResponse {
    match request {
        NodeRequest::Ping => NodeResponse::Pong,
        NodeRequest::Height => NodeResponse::Height(state.height()),
        NodeRequest::GetTip => match state.tip_hash() {
            Some(hash) => NodeResponse::Tip(hash.to_string()),
            None => NodeResponse::Error("no tip".to_string()),
        },
        NodeRequest::GetBlock(hash) => match state.get_block(&hash) {
            Some(block) => NodeResponse::Block(block.clone()),
            None => NodeResponse::NotFound,
        },
        NodeRequest::AddPeer(address) => {
            let count = state.add_peer(&address);
            NodeResponse::PeerAdded(count)
        }
        NodeRequest::GetPeers => NodeResponse::Peers(state.peer_addresses()),
        NodeRequest::SubmitBlock(block) => {
            let hash = block.hash.clone();
            match state.append_block(block) {
                Ok(()) => NodeResponse::Accepted(hash),
                Err(err) => NodeResponse::Rejected(err.to_string()),
            }
        }
    }
}

/// Run a state manager task that serializes access to `NodeState`.
pub async fn run_state_manager(mut state: NodeState, mut receiver: mpsc::Receiver<NodeCommand>) {
    while let Some(cmd) = receiver.recv().await {
        let response = handle_request(&mut state, cmd.request);
        let _ = cmd.response.send(response);
    }
}

/// Spawn a state manager and return a bounded command sender.
pub fn spawn_state_manager(state: NodeState, capacity: usize) -> mpsc::Sender<NodeCommand> {
    let (sender, receiver) = mpsc::channel(capacity);
    tokio::spawn(run_state_manager(state, receiver));
    sender
}

/// Send one request to the state manager and wait for its response.
pub async fn send_request(
    sender: &mpsc::Sender<NodeCommand>,
    request: NodeRequest,
) -> Result<NodeResponse, NodeError> {
    let (response_tx, response_rx) = oneshot::channel();
    sender
        .send(NodeCommand {
            request,
            response: response_tx,
        })
        .await
        .map_err(|_| NodeError::ChannelClosed)?;
    response_rx.await.map_err(|_| NodeError::ChannelClosed)
}
