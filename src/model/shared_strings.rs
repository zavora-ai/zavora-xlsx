use std::collections::HashMap;
use std::sync::Arc;

/// Deduplicated shared string table for xlsx.
#[derive(Debug, Default)]
pub struct SharedStringTable {
    strings: Vec<Arc<str>>,
    index_map: HashMap<Arc<str>, u32>,
    ref_counts: Vec<u32>,
}

impl SharedStringTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or insert a string, returning its index.
    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&idx) = self.index_map.get(s) {
            self.ref_counts[idx as usize] += 1;
            return idx;
        }
        let idx = self.strings.len() as u32;
        let arc: Arc<str> = Arc::from(s);
        self.strings.push(Arc::clone(&arc));
        self.index_map.insert(arc, idx);
        self.ref_counts.push(1);
        idx
    }

    /// Look up a string by index.
    pub fn get(&self, index: u32) -> Option<&str> {
        self.strings.get(index as usize).map(|s| &**s)
    }

    /// Number of unique strings.
    pub fn len(&self) -> u32 {
        self.strings.len() as u32
    }
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Iterate all strings in order.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.strings.iter().map(|s| &**s)
    }

    /// Add a string at a specific index (used during parsing).
    pub fn push(&mut self, s: &str) {
        let arc: Arc<str> = Arc::from(s);
        let idx = self.strings.len() as u32;
        self.index_map.insert(Arc::clone(&arc), idx);
        self.strings.push(arc);
        self.ref_counts.push(1);
    }

    /// Get the reference count for a string by index.
    pub fn ref_count(&self, index: u32) -> u32 {
        self.ref_counts.get(index as usize).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_dedup() {
        let mut sst = SharedStringTable::new();
        assert_eq!(sst.intern("hello"), 0);
        assert_eq!(sst.intern("world"), 1);
        assert_eq!(sst.intern("hello"), 0); // dedup
        assert_eq!(sst.len(), 2);
        assert_eq!(sst.get(0), Some("hello"));
        assert_eq!(sst.get(1), Some("world"));
        assert_eq!(sst.get(2), None);
    }
}
