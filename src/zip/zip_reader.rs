use std::collections::HashMap;
use std::io::{BufReader, Read, Seek};

use zip::ZipArchive;

/// Wraps a ZipArchive with a case-insensitive path cache for robust entry lookup.
pub struct ZipReader<R: Read + Seek> {
    pub archive: ZipArchive<R>,
    path_cache: HashMap<String, String>,
}

impl<R: Read + Seek> ZipReader<R> {
    pub fn new(reader: R) -> crate::Result<Self> {
        let mut archive = ZipArchive::new(reader)?;
        let mut path_cache = HashMap::with_capacity(archive.len());
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index_raw(i) {
                let name = entry.name().to_string();
                let key = name.replace('\\', "/").to_ascii_lowercase();
                path_cache.insert(key, name);
            }
        }
        Ok(Self { archive, path_cache })
    }

    /// Resolve a path through the case-insensitive cache.
    pub fn resolve_path<'a>(&'a self, path: &'a str) -> &'a str {
        let key = path.to_ascii_lowercase();
        self.path_cache.get(&key).map(|s| s.as_str()).unwrap_or(path)
    }

    /// Read an entry's full contents as bytes. Returns None if not found.
    pub fn read_entry(&mut self, path: &str) -> Option<crate::Result<Vec<u8>>> {
        let resolved = self.resolve_path(path).to_string();
        match self.archive.by_name(&resolved) {
            Ok(mut entry) => {
                let mut buf = Vec::with_capacity(entry.size() as usize);
                match std::io::Read::read_to_end(&mut entry, &mut buf) {
                    Ok(_) => Some(Ok(buf)),
                    Err(e) => Some(Err(e.into())),
                }
            }
            Err(zip::result::ZipError::FileNotFound) => None,
            Err(e) => Some(Err(e.into())),
        }
    }

    /// Check if an entry exists.
    pub fn has_entry(&self, path: &str) -> bool {
        let key = path.to_ascii_lowercase();
        self.path_cache.contains_key(&key)
    }
}

impl ZipReader<BufReader<std::fs::File>> {
    pub fn open(path: &std::path::Path) -> crate::Result<Self> {
        let file = std::fs::File::open(path)?;
        Self::new(BufReader::new(file))
    }
}
