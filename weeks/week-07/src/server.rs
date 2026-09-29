use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};

use crate::{encode_response, parse_request, send_request, NodeCommand, NodeError, NodeResponse};

/// Parse a line, send it to the state manager, and encode the response.
pub async fn handle_line(
    sender: &mpsc::Sender<NodeCommand>,
    line: &str,
) -> Result<String, NodeError> {
    let request = parse_request(line)?;
    let response = send_request(sender, request).await?;
    Ok(encode_response(&response))
}

/// Handle one TCP connection line-by-line.
pub async fn handle_connection(
    stream: TcpStream,
    sender: mpsc::Sender<NodeCommand>,
) -> Result<(), NodeError> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let response_line = match handle_line(&sender, &line).await {
            Ok(encoded) => encoded,
            Err(err) => encode_response(&NodeResponse::Error(err.to_string())),
        };
        writer.write_all(response_line.as_bytes()).await?;
    }
    Ok(())
}

/// Run a TCP server until the shutdown signal is received.
pub async fn run_tcp_server(
    listener: TcpListener,
    sender: mpsc::Sender<NodeCommand>,
    mut shutdown: oneshot::Receiver<()>,
) -> Result<(), NodeError> {
    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                let (stream, _) = accept_result?;
                let sender = sender.clone();
                tokio::spawn(handle_connection(stream, sender));
            }
            _ = &mut shutdown => {
                break;
            }
        }
    }
    Ok(())
}
