#![allow(unused_variables)]

#[derive(Debug, PartialEq, Eq)]
pub struct ParsedOutpoint<'a> {
    pub txid: &'a str,
    pub vout: u32,
}

/// Return true when `input` reads the same forward and backward.
///
/// Ignore ASCII case, whitespace, and punctuation.
pub fn is_palindrome(input: &str) -> bool {
    let cleaned: Vec<char> = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let reversed: Vec<char> = cleaned.iter().rev().cloned().collect();
    cleaned == reversed
}

/// Compute the assignment toy hash.
///
/// Start at zero and, for each byte, update with `hash = hash * 31 + byte`
/// using wrapping arithmetic.
pub fn simple_hash(input: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

/// Return `input_sats - output_sats` when inputs cover outputs.
///
/// Return `None` if outputs are larger than inputs.
pub fn calculate_fee(input_sats: u64, output_sats: u64) -> Option<u64> {
    if output_sats > input_sats {
        None
    } else {
        Some(input_sats - output_sats)
    }
}

/// Return the fee rate in sats/vbyte, rounded up.
///
/// Return `None` when `vbytes` is zero.
pub fn fee_rate(fee_sats: u64, vbytes: u64) -> Option<u64> {
    if vbytes == 0 {
        return None;
    }
    Some((fee_sats + vbytes - 1) / vbytes)
}

/// Return the longer borrowed string slice.
///
/// If both have the same length, return `left`.
pub fn select_longer<'a>(left: &'a str, right: &'a str) -> &'a str {
    if right.len() > left.len() {
        right
    } else {
        left
    }
}

/// Return the first whitespace-separated word from `input`.
///
/// Skip leading whitespace and return an empty slice for empty or all-whitespace
/// input.
pub fn first_word(input: &str) -> &str {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return "";
    }
    match trimmed.find(char::is_whitespace) {
        Some(idx) => &trimmed[..idx],
        None => trimmed,
    }
}

/// Return the last whitespace-separated word from `input`.
///
/// Ignore trailing whitespace and return an empty slice for empty or all-whitespace
/// input.
pub fn last_word(input: &str) -> &str {
    let trimmed = input.trim_end();
    if trimmed.is_empty() {
        return "";
    }
    match trimmed.rfind(char::is_whitespace) {
        Some(idx) => &trimmed[idx + 1..],
        None => trimmed,
    }
}

/// Remove `prefix` from the front of `input` when it is present.
///
/// Return the original borrowed `input` slice when the prefix is missing.
pub fn trim_prefix<'a>(input: &'a str, prefix: &str) -> &'a str {
    if prefix.is_empty() {
        return input;
    }
    match input.strip_prefix(prefix) {
        Some(rest) => rest,
        None => input,
    }
}

/// Parse a trimmed unsigned satoshi amount.
///
/// Return `None` for empty, negative, or non-numeric input.
pub fn parse_sats(input: &str) -> Option<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<u64>().ok()
}

/// Split `input` once on the first colon and trim both sides.
///
/// Return `None` when no colon exists.
pub fn split_once_colon(input: &str) -> Option<(&str, &str)> {
    let idx = input.find(':')?;
    let left = input[..idx].trim();
    let right = input[idx + 1..].trim();
    Some((left, right))
}

/// Join transaction ids with commas.
///
/// Return an empty string for an empty slice.
pub fn join_txids(txids: &[&str]) -> String {
    txids.join(",")
}

/// Trim, lowercase, and replace runs of whitespace with single hyphens.
pub fn normalize_label(input: &str) -> String {
    input
        .trim()
        .split_whitespace()
        .map(|word| word.to_lowercase())
        .collect::<Vec<String>>()
        .join("-")
}

/// Return true when `needle` exactly matches one of the owned txids.
pub fn contains_txid(txids: &[String], needle: &str) -> bool {
    txids.iter().any(|txid| txid == needle)
}

/// Return a newly allocated string containing `input` followed by `suffix`.
pub fn duplicate_with_suffix(input: &str, suffix: &str) -> String {
    let mut result = String::from(input);
    result.push_str(suffix);
    result
}

/// Sum the byte lengths of all string slices in `parts`.
pub fn total_byte_len(parts: &[&str]) -> usize {
    parts.iter().map(|part| part.len()).sum()
}

/// Return the borrowed value when present, otherwise return the borrowed default.
pub fn borrowed_or_default<'a>(value: Option<&'a str>, default: &'a str) -> &'a str {
    match value {
        Some(text) => text,
        None => default,
    }
}

/// Find `key` in a slice of `(name, amount)` pairs and return the amount.
pub fn lookup_amount(pairs: &[(&str, u64)], key: &str) -> Option<u64> {
    for (name, amount) in pairs {
        if *name == key {
            return Some(*amount);
        }
    }
    None
}

/// Parse an outpoint written as `txid:vout`.
///
/// Trim both fields, borrow the txid from the input, and return `None` for
/// missing separators, empty txids, or non-numeric vouts.
pub fn parse_outpoint(input: &str) -> Option<ParsedOutpoint<'_>> {
    let colon_pos = input.find(':')?;
    let txid = input[..colon_pos].trim();
    let vout_str = input[colon_pos + 1..].trim();
    if txid.is_empty() {
        return None;
    }
    let vout = vout_str.parse::<u32>().ok()?;
    Some(ParsedOutpoint { txid, vout })
}
