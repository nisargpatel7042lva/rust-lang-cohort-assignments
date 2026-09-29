use crate::WalletError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WalletCommand {
    Balance,
    History,
    Sync,
    Send {
        recipient: String,
        amount_sats: u64,
        fee_sats: u64,
    },
}

/// Parse a small wallet command language.
pub fn parse_wallet_command(input: &str) -> Result<WalletCommand, WalletError> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err(WalletError::MalformedData);
    }
    match parts[0] {
        "balance" => {
            if parts.len() != 1 {
                return Err(WalletError::MalformedData);
            }
            Ok(WalletCommand::Balance)
        }
        "history" => {
            if parts.len() != 1 {
                return Err(WalletError::MalformedData);
            }
            Ok(WalletCommand::History)
        }
        "sync" => {
            if parts.len() != 1 {
                return Err(WalletError::MalformedData);
            }
            Ok(WalletCommand::Sync)
        }
        "send" => {
            if parts.len() != 4 {
                return Err(WalletError::MalformedData);
            }
            let recipient = parts[1];
            if recipient.is_empty() {
                return Err(WalletError::InvalidAmount);
            }
            let amount_sats: u64 = parts[2].parse().map_err(|_| WalletError::MalformedData)?;
            if amount_sats == 0 {
                return Err(WalletError::InvalidAmount);
            }
            let fee_sats: u64 = parts[3].parse().map_err(|_| WalletError::MalformedData)?;
            Ok(WalletCommand::Send {
                recipient: recipient.to_string(),
                amount_sats,
                fee_sats,
            })
        }
        _ => Err(WalletError::MalformedData),
    }
}

/// Render a command into a compact log-friendly label.
pub fn wallet_command_label(command: &WalletCommand) -> String {
    match command {
        WalletCommand::Balance => "balance".to_string(),
        WalletCommand::History => "history".to_string(),
        WalletCommand::Sync => "sync".to_string(),
        WalletCommand::Send {
            recipient,
            amount_sats,
            fee_sats,
        } => format!("send:{recipient}:{amount_sats}:{fee_sats}"),
    }
}
