// src/varint.rs
use anyhow::{bail, Result};

/// Encode a slice of u32s into LEB128 (little-endian base-128) bytes.
/// Each value uses 1..=5 bytes.
pub fn encode(values: &[u32]) -> Vec<u8> {
    // Max 5 bytes per u32; reserve a bit to reduce reallocs.
    let mut out = Vec::with_capacity(values.len() * 5);

    for mut v in values.iter().copied() {
        // Emit 7 bits per byte, set MSB if more bytes follow.
        while v >= 0x80 {
            out.push(((v & 0x7F) as u8) | 0x80);
            v >>= 7;
        }
        out.push((v & 0x7F) as u8); // last byte, MSB clear
    }

    out
}

/// Decode a LEB128 byte stream back into u32s.
/// Errors on truncated final value or shift overflow.
pub fn decode(bytes: &[u8]) -> Result<Vec<u32>> {
    let mut out = Vec::new();

    let mut acc: u32 = 0;
    let mut shift: u32 = 0;

    for &b in bytes {
        let data = (b & 0x7F) as u32;

        // Avoid shifting >= 32 for u32
        if shift >= 32 {
            bail!("varint overflow while decoding u32");
        }
        acc |= data << shift;

        if (b & 0x80) == 0 {
            // terminal byte
            out.push(acc);
            acc = 0;
            shift = 0;
        } else {
            // continuation
            shift += 7;
        }
    }

    // If we exited with an unfinished value, it's truncated.
    if shift != 0 {
        bail!("incomplete varint at end of stream");
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_basic_values() {
        let cases: &[u32] = &[
            0, 1, 2, 3, 4, 5, 10, 63, 64, 127, 128, 300, 16384, u32::MAX,
        ];
        let enc = encode(cases);
        let dec = decode(&enc).expect("decode ok");
        assert_eq!(dec, cases);
    }

    #[test]
    fn encodings_match_known_pairs() {
        // A few known LEB128 encodings (little-endian, 7 bits + continuation)
        // 0     -> [0x00]
        // 127   -> [0x7F]
        // 128   -> [0x80, 0x01]
        // 300   -> [0xAC, 0x02]   (0x12C => 0b 1_0101100 -> 0b10101100, 0b00000010)
        let pairs: &[(&[u8], u32)] = &[
            (&[0x00], 0),
            (&[0x7F], 127),
            (&[0x80, 0x01], 128),
            (&[0xAC, 0x02], 300),
        ];

        for (bytes, value) in pairs {
            assert_eq!(decode(bytes).unwrap(), vec![*value]);
            assert_eq!(encode(&[*value]), *bytes);
        }
    }

    #[test]
    fn error_on_truncated_stream() {
        // 0x80 indicates continuation, but we end immediately => error
        let truncated = [0x80u8];
        let err = decode(&truncated).unwrap_err().to_string();
        assert!(err.contains("incomplete varint"));
    }

    #[test]
    fn many_values_roundtrip() {
        let mut vals = Vec::new();
        for i in 0..50_000u32 {
            vals.push((i * 2654435761u32) ^ 0xDEADBEEF); // pseudo-randomish spread
        }
        let enc = encode(&vals);
        let dec = decode(&enc).unwrap();
        assert_eq!(dec, vals);
    }
}
