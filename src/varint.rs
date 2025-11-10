// src/varint.rs
use anyhow::{bail, Result};

/// Encode a slice of u32 values using unsigned LEB128.
/// Little-endian base-128, 7 bits of payload per byte.
/// For each value:
///   - emit low 7 bits with MSB=1 while more bits remain
///   - emit final 7 bits with MSB=0
pub fn encode(values: &[u32]) -> Vec<u8> {
    // Max 5 bytes per u32, so this capacity hint is a small win.
    let mut out = Vec::with_capacity(values.len() * 5);

    for mut v in values.iter().copied() {
        // At least one byte is emitted even when v == 0
        loop {
            let byte = (v & 0x7F) as u8;
            v >>= 7;
            if v != 0 {
                // More bits remain: set continuation bit
                out.push(byte | 0x80);
            } else {
                // Final byte: continuation bit clear
                out.push(byte);
                break;
            }
        }
    }
    out
}

/// Decode a byte slice of unsigned LEB128 values into u32s.
/// Returns an error when the stream is malformed:
///  - ends with a dangling continuation bit
///  - would overflow u32 (shift >= 32 before finishing a value)
pub fn decode(bytes: &[u8]) -> Result<Vec<u32>> {
    let mut out = Vec::new();
    let mut acc: u32 = 0;
    let mut shift: u32 = 0;

    for &b in bytes {
        let payload = (b & 0x7F) as u32;

        // Guard: shifting past 31 would overflow u32
        if shift >= 32 {
            bail!("varint overflow: shift exceeded 31 bits");
        }

        acc |= payload << shift;

        if (b & 0x80) == 0 {
            // Final byte of this value
            out.push(acc);
            acc = 0;
            shift = 0;
        } else {
            // Continuation: prepare to place next 7 bits
            shift += 7;
        }
    }

    // If we ended with a continuation bit set, it's incomplete
    if shift != 0 {
        bail!("incomplete varint at end of stream");
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_basic_values() {
        let vals = [0u32, 1, 2, 3, 127, 128, 129, 300, 16_384, u32::MAX];
        let enc = encode(&vals);
        let dec = decode(&enc).unwrap();
        assert_eq!(dec, vals);
    }

    #[test]
    fn encodings_match_known_cases() {
        // 0 -> [0x00]
        assert_eq!(encode(&[0]), vec![0x00]);

        // 127 -> [0x7F]
        assert_eq!(encode(&[127]), vec![0x7F]);

        // 128 -> [0x80, 0x01]
        assert_eq!(encode(&[128]), vec![0x80, 0x01]);

        // 300 -> [0xAC, 0x02]
        assert_eq!(encode(&[300]), vec![0xAC, 0x02]);
    }

    #[test]
    fn decode_errors_on_incomplete_stream() {
        // 0x80 alone means "continue", but stream ends → error
        let err = decode(&[0x80]).unwrap_err().to_string();
        assert!(err.contains("incomplete"));
    }

    #[test]
    fn decode_errors_on_overflow() {
        // Craft a stream that tries to shift beyond 31 bits for a single value.
        // Ten continuation bytes of 0xFF followed by 0x01 will exceed u32.
        let mut bytes = vec![0xFF; 10];
        bytes.push(0x01);
        let err = decode(&bytes).unwrap_err().to_string();
        assert!(err.contains("overflow"));
    }
}
