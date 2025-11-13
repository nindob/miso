// src/freq_map.rs
use std::collections::HashMap;

/// Per-payload frequency mapping.
/// - `forward[token] = mapped_id`
/// - `reverse[mapped_id] = token` (tokens ordered by rank)
///
/// Later we'll also store/serialize minimal header metadata
/// (e.g., lengths) so decode can rebuild the map.
#[derive(Debug, Clone)]
pub struct FreqMap {
    /// Original token -> mapped ID (dense ranks starting at 0)
    forward: HashMap<i32, i32>,
    /// Indexed by mapped ID; holds the original token at that rank.
    reverse: Vec<i32>,
    /// Optional: raw counts we used to build the ranking (kept for debugging/metrics).
    counts: HashMap<i32, u32>,
}

impl FreqMap {
    /// Build a FreqMap from raw token IDs.
    ///
    /// Steps to implement (in the next pass):
    /// 1) Count occurrences of each token.
    /// 2) Produce a ranked list of tokens:
    ///    - Sort by descending count, then ascending token ID for stable ties.
    /// 3) Assign mapped IDs [0..len), fill `forward` and `reverse`.
    pub fn from_token_ids(_ids: &[i32]) -> Self {
        // TODO: implement logic
        // Placeholder so we compile for now.
        Self {
            forward: HashMap::new(),
            reverse: Vec::new(),
            counts: HashMap::new(),
        }
    }

    /// Map an original token -> mapped ID for the encode path.
    /// Returns None if the token was not present in this payload.
    pub fn map_token(&self, _token: i32) -> Option<i32> {
        // TODO: implement
        todo!("return self.forward.get(&token).copied()")
    }

    /// Reverse lookup: mapped ID -> original token (for decode).
    pub fn unmap_token(&self, _mapped: i32) -> Option<i32> {
        // TODO: implement
        todo!("self.reverse.get(mapped as usize).copied()")
    }

    /// Slice of tokens ordered by mapped ID; used when serializing the header.
    pub fn ordered_tokens(&self) -> &[i32] {
        // This one can work now even before we fill logic.
        &self.reverse
    }

    /// (Optional) Access counts for metrics/debug.
    pub fn counts(&self) -> &HashMap<i32, u32> {
        &self.counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_compiles_and_has_shapes() {
        // Build from small payload.
        let fm = FreqMap::from_token_ids(&[1, 1, 2, 3, 3, 3]);
        // Later: assert ranks like 3->0, 1->1, 2->2, etc.
        // Example checks we’ll enable once implemented:
        // assert_eq!(fm.map_token(3), Some(0));
        // assert_eq!(fm.map_token(1), Some(1));
        // assert_eq!(fm.map_token(2), Some(2));
        // assert_eq!(fm.unmap_token(0), Some(3));
        // assert_eq!(fm.unmap_token(1), Some(1));
        // assert_eq!(fm.unmap_token(2), Some(2));

        // For now, just ensure ordered_tokens returns a slice (possibly empty).
        let _ot = fm.ordered_tokens();
        // And counts exist.
        let _c = fm.counts();
    }
}
