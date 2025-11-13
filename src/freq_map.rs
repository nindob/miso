use std::collections::HashMap;

/// FreqMap holds a *per-payload* mapping between:
/// - original token IDs (from the tokenizer), and
/// - dense, frequency-ranked "mapped IDs" starting at 0.
///
/// High-level idea:
///   - For a given payload, we look at all token IDs that appear.
///   - We count their frequencies.
///   - We sort tokens by frequency (desc), then by token ID (asc).
///   - We assign mapped IDs 0, 1, 2, ... in that order.
/// This makes common tokens map to small integers, which helps later zigzag/varint compression.
#[derive(Debug, Clone)]
pub struct FreqMap {
    /// original token ID -> mapped ID (0..N-1)
    token_to_mapped: HashMap<i32, i32>,

    /// mapped ID (as index) -> original token ID
    ///
    /// invariant: mapped_to_token[mapped_id] == original_token
    mapped_to_token: Vec<i32>,
}

impl FreqMap {
    /// Build a frequency-based mapping from a slice of token IDs.
    ///
    /// Algorithm:
    ///  1. Count frequency of each token in `ids`.
    ///  2. Collect (token, count) into a Vec.
    ///  3. Sort by count desc, then token asc (for deterministic tie-breaking).
    ///  4. Assign mapped IDs 0..N-1 in that order and fill both lookup structures.
    pub fn from_token_ids(ids: &[i32]) -> Self {
        // 1) Count frequencies for this payload.
        let mut counts: HashMap<i32, usize> = HashMap::new();
        for &token in ids {
            *counts.entry(token).or_insert(0) += 1;
        }

        // 2) Collect into a sortable Vec so ordering is deterministic.
        let mut entries: Vec<(i32, usize)> = counts.into_iter().collect();

        // 3) Sort:
        //    - primary key: frequency descending (higher count first)
        //    - secondary key: token ID ascending (for stable, deterministic ordering)
        entries.sort_by(|a, b| {
            b.1.cmp(&a.1)                // compare counts, reversed for DESC
                .then_with(|| a.0.cmp(&b.0)) // tie-break with token ID ASC
        });

        // 4) Assign mapped IDs and build forward & reverse lookups.
        let mut token_to_mapped = HashMap::with_capacity(entries.len());
        let mut mapped_to_token = Vec::with_capacity(entries.len());

        for (mapped_id, (token, _count)) in entries.into_iter().enumerate() {
            token_to_mapped.insert(token, mapped_id as i32);
            mapped_to_token.push(token);
        }

        Self {
            token_to_mapped,
            mapped_to_token,
        }
    }

    /// Look up the mapped ID for an original token.
    ///
    /// Returns:
    ///   - Some(mapped_id) if the token is present in this payload,
    ///   - None if the token never appeared when we built the map.
    pub fn map_token(&self, token: i32) -> Option<i32> {
        self.token_to_mapped.get(&token).copied()
    }

    /// Inverse lookup: mapped ID -> original token ID.
    ///
    /// Returns:
    ///   - Some(token) if `mapped` is a valid mapped ID (0 <= mapped < len),
    ///   - None if out-of-range.
    pub fn unmap_token(&self, mapped: i32) -> Option<i32> {
        if mapped < 0 {
            return None;
        }
        self.mapped_to_token.get(mapped as usize).copied()
    }

    /// Returns a slice of original token IDs, in mapped-ID order.
    ///
    /// That is, ordered_tokens()[i] == token whose mapped ID is `i`.
    pub fn ordered_tokens(&self) -> &[i32] {
        &self.mapped_to_token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freq_map_basic_ordering() {
        // Example payload: 1 appears 3 times, 2 appears 2 times, 3 appears 1 time.
        // So mapping should be: 1 -> 0, 2 -> 1, 3 -> 2.
        let ids = [1, 2, 1, 3, 2, 1];
        let fm = FreqMap::from_token_ids(&ids);

        // Forward mapping checks.
        assert_eq!(fm.map_token(1), Some(0));
        assert_eq!(fm.map_token(2), Some(1));
        assert_eq!(fm.map_token(3), Some(2));

        // Reverse mapping checks.
        assert_eq!(fm.unmap_token(0), Some(1));
        assert_eq!(fm.unmap_token(1), Some(2));
        assert_eq!(fm.unmap_token(2), Some(3));

        // Out-of-range mapped IDs should return None.
        assert_eq!(fm.unmap_token(3), None);

        // ordered_tokens should be [1, 2, 3] for this example.
        assert_eq!(fm.ordered_tokens(), &[1, 2, 3]);
    }

    #[test]
    fn freq_map_tie_break_on_token_id() {
        // Here 10 and 20 both appear once; 5 appears once too.
        // All counts are equal, so ordering falls back to token ID ascending:
        // tokens = [5, 10, 20], so mapped IDs: 5->0, 10->1, 20->2.
        let ids = [10, 20, 5];
        let fm = FreqMap::from_token_ids(&ids);

        assert_eq!(fm.map_token(5), Some(0));
        assert_eq!(fm.map_token(10), Some(1));
        assert_eq!(fm.map_token(20), Some(2));
        assert_eq!(fm.ordered_tokens(), &[5, 10, 20]);
    }
}