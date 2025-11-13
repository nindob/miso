use anyhow::{bail, Result};
use crate::freq_map::FreqMap;

/// Metadata describing how tokens were remapped for this payload.
///
/// The key idea: we only need to store the *ordering* of original tokens
/// by their mapped ID. During encode, mapped IDs are assigned in rank order:
///   mapped_id = index in `tokens`
/// So if `tokens = [11, 42, -5]`, then:
///   11 -> mapped 0
///   42 -> mapped 1
///   -5 -> mapped 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Original tokens ordered by mapped ID (highest frequency first).
    pub tokens: Vec<i32>,
    /// Number of entries in the frequency map (cached for convenience).
    pub len: usize,
}

impl Header {
    /// Build a header from an existing `FreqMap`, capturing the token ordering.
    ///
    /// `FreqMap::ordered_tokens()` returns a slice already sorted by:
    ///   - descending frequency
    ///   - then ascending token ID on ties
    pub fn from_freq_map(freq: &FreqMap) -> Self {
        let tokens = freq.ordered_tokens().to_vec();
        let len = tokens.len();
        Self { tokens, len }
    }

    /// Serialize the header into bytes (tokens + length) for inclusion in the payload.
    ///
    /// Format (little-endian):
    ///   [0..4)    : u32 length (number of tokens)
    ///   [4..]     : `len` i32 values (4 bytes each) representing the original tokens
    pub fn encode(&self) -> Vec<u8> {
        // Ensure `len` matches the actual tokens length.
        let len = self.tokens.len() as u32;

        // Allocate capacity: 4 bytes for length + 4 bytes per token.
        let mut out = Vec::with_capacity(4 + self.tokens.len() * 4);

        // Write length as little-endian u32.
        out.extend_from_slice(&len.to_le_bytes());

        // Write each token as little-endian i32.
        for &token in &self.tokens {
            out.extend_from_slice(&token.to_le_bytes());
        }

        out
    }

    /// Parse a header from bytes, reconstructing the token ordering information.
    ///
    /// Performs basic validation:
    ///   - At least 4 bytes for the length.
    ///   - Remaining bytes must be exactly `len * 4`.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        // Need at least 4 bytes to read the length.
        if bytes.len() < 4 {
            bail!("header too short: missing length prefix");
        }

        // First 4 bytes: length as little-endian u32.
        let len_bytes: [u8; 4] = bytes[0..4]
            .try_into()
            .expect("slice of length 4 will always convert");
        let len = u32::from_le_bytes(len_bytes) as usize;

        // Remaining bytes must be exactly `len * 4`.
        let expected_bytes = len
            .checked_mul(4)
            .ok_or_else(|| anyhow::anyhow!("header length overflow"))?;
        let actual_bytes = bytes.len() - 4;

        if actual_bytes != expected_bytes {
            bail!(
                "header size mismatch: expected {} bytes for tokens, got {}",
                expected_bytes,
                actual_bytes
            );
        }

        let mut tokens = Vec::with_capacity(len);
        let mut offset = 4;

        for _ in 0..len {
            let end = offset + 4;
            let chunk: [u8; 4] = bytes[offset..end]
                .try_into()
                .expect("slice of length 4 will always convert");
            let token = i32::from_le_bytes(chunk);
            tokens.push(token);
            offset = end;
        }

        Ok(Self { tokens, len })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_round_trip_manual_tokens() {
        let tokens = vec![10, 20, -5, 42];
        let header = Header {
            tokens: tokens.clone(),
            len: tokens.len(),
        };

        let bytes = header.encode();
        let decoded = Header::decode(&bytes).unwrap();

        assert_eq!(decoded.tokens, tokens);
        assert_eq!(decoded.len, tokens.len());
    }

    #[test]
    fn header_handles_empty() {
        let header = Header {
            tokens: Vec::new(),
            len: 0,
        };

        let bytes = header.encode();
        let decoded = Header::decode(&bytes).unwrap();

        assert!(decoded.tokens.is_empty());
        assert_eq!(decoded.len, 0);
    }

    // Optional: only if you want to hook up FreqMap here.
    // This verifies that `from_freq_map` + encode/decode preserves ordering.
    #[test]
    fn header_from_freq_map_round_trip() {
        // Fake a tiny frequency map:
        // You already implemented FreqMap::from_token_ids in freq_map.rs.
        let freq = FreqMap::from_token_ids(&[1, 1, 2, 3, 3, 3]);

        let header = Header::from_freq_map(&freq);
        let bytes = header.encode();
        let decoded = Header::decode(&bytes).unwrap();

        // Both token ordering and len should be stable.
        assert_eq!(decoded.tokens, header.tokens);
        assert_eq!(decoded.len, header.len);
    }
}
