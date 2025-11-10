// src/zigzag.rs

/// Zigzag-encode a signed 32 bit integer to an unsigned 32 bit integer
/// Mapping: 0->0, -1->1, 1->2, -2->3, 2->4, ...
#[inline]
pub fn encode(value: i32) -> u32 {
    // Classic protobuf zigzag: (n << 1) ^ (n >> 31)
    // Note: shifts on signed types are defined; the cast to u32 at the end
    // gives the intended bit pattern in unsigned space
    ((value << 1) ^ (value >> 31)) as u32
}

/// Reverse zigzag: recover the original signed integer
#[inline]
pub fn decode(value: u32) -> i32 {
    // ((n >> 1) as i32) ^ (-((n & 1) as i32))
    ((value >> 1) as i32) ^ (-((value & 1) as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_small_values() {
        let cases = [-2, -1, 0, 1, 2, 17, -17, 123456, -123456];
        for &x in &cases {
            let z = encode(x);
            let y = decode(z);
            assert_eq!(y, x, "failed round-trip for {x} -> {z}");
        }
    }

    #[test]
    fn exact_known_mappings() {
        // Spot-check canonical mappings
        assert_eq!(encode(0), 0);
        assert_eq!(encode(-1), 1);
        assert_eq!(encode(1), 2);
        assert_eq!(encode(-2), 3);
        assert_eq!(encode(2), 4);
    }

    #[test]
    fn extremes_round_trip() {
        // i32::MIN maps to 0xFFFF_FFFF; should still decode back.
        assert_eq!(decode(encode(i32::MIN)), i32::MIN);
        assert_eq!(decode(encode(i32::MAX)), i32::MAX);
    }
}