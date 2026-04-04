use std::collections::HashMap;
use std::sync::Arc;

/// Deduplicated shared string table for xlsx.
#[derive(Debug, Default)]
pub struct SharedStringTable {
    strings: Vec<Arc<str>>,
    index_map: HashMap<Arc<str>, u32>,
}

impl SharedStringTable {
    pub fn new() -> Self { Self::default() }

    /// Get or insert a string, returning its index.
    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&idx) = self.index_map.get(s) {
            return idx;
        }
        let idx = self.strings.len() as u32;
        let arc: Arc<str> = Arc::from(s);
        self.strings.push(Arc::clone(&arc));
        self.index_map.insert(arc, idx);
        idx
    }

    /// Look up a string by index.
    pub fn get(&self, index: u32) -> Option<&str> {
        self.strings.get(index as usize).map(|s| &**s)
    }

    /// Number of unique strings.
    pub fn len(&self) -> u32 { self.strings.len() as u32 }
    pub fn is_empty(&self) -> bool { self.strings.is_empty() }

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
