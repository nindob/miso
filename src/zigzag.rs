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