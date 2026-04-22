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
        Ok(Self {
            archive,
            path_cache,
        })
    }

    /// Resolve a path through the case-insensitive cache.
    pub fn resolve_path<'a>(&'a self, path: &'a str) -> &'a str {
        let key = path.to_ascii_lowercase();
        self.path_cache
            .get(&key)
            .map(|s| s.as_str())
            .unwrap_or(path)
    }

    /// Read an entry's full contents as bytes. Returns None if not found.
    pub fn read_entry(&mut self, path: &str) -> Option<crate::Result<Vec<u8>>> {
        const MAX_DECOMPRESSED: u64 = 200 * 1024 * 1024; // 200 MB
        let resolved = self.resolve_path(path).to_string();
        match self.archive.by_name(&resolved) {
            Ok(mut entry) => {
                if entry.size() > MAX_DECOMPRESSED {
                    return Some(Err(crate::Error::InvalidData(format!(
                        "Entry '{}' decompressed size {} exceeds limit",
                        path,
                        entry.size()
                    ))));
                }
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
    #[allow(dead_code)]
    pub fn has_entry(&self, path: &str) -> bool {
        let key = path.to_ascii_lowercase();
        self.path_cache.contains_key(&key)
    }
}

impl ZipReader<BufReader<std::fs::File>> {
    pub fn open(path: &std::path::Path) -> crate::Result<Self> {
        // Check for OLE/CFB magic bytes (password-protected xlsx files are CFB containers)
        let mut file = std::fs::File::open(path)?;
        let mut magic = [0u8; 8];
        if std::io::Read::read(&mut file, &mut magic).is_ok()
            && magic == [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
        {
            return Err(crate::Error::Password);
        }
        // Reset to beginning for zip reading
        std::io::Seek::seek(&mut file, std::io::SeekFrom::Start(0))?;
        Self::new(BufReader::new(file))
    }
}
