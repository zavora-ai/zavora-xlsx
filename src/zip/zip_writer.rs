use std::io::{Cursor, Write};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// Wraps ZipWriter for building xlsx archives.
pub struct ZipOutput {
    writer: ZipWriter<Cursor<Vec<u8>>>,
    /// Every entry written so far.
    ///
    /// Kept so that parts carried over from the original file can yield to the ones this save
    /// generates. A workbook opened with a chart in it holds both the parsed chart and the
    /// original drawing part, and both wanted to be `xl/drawings/drawing1.xml` — so adding a
    /// second chart failed the whole save with "Duplicate filename".
    written: std::collections::HashSet<String>,
}

impl ZipOutput {
    pub fn new() -> Self {
        Self {
            writer: ZipWriter::new(Cursor::new(Vec::new())),
            written: std::collections::HashSet::new(),
        }
    }

    /// Add a file entry with deflate compression.
    pub fn add_file(&mut self, path: &str, data: &[u8]) -> crate::Result<()> {
        let opts =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        // Named, because "Duplicate filename" without the name tells whoever is looking nothing
        // about which part of the workbook was written twice.
        self.writer.start_file(path, opts).map_err(|error| {
            crate::Error::InvalidData(format!("could not write {path}: {error}"))
        })?;
        self.writer.write_all(data)?;
        self.written.insert(path.to_string());
        Ok(())
    }

    /// Add an entry unless something has already been written under that name.
    ///
    /// For parts carried over from the original file, where what this save generated is the more
    /// current of the two.
    pub fn add_file_if_absent(&mut self, path: &str, data: &[u8]) -> crate::Result<()> {
        if self.written.contains(path) {
            return Ok(());
        }
        self.add_file(path, data)
    }

    /// Finish writing and return the zip bytes.
    pub fn finish(self) -> crate::Result<Vec<u8>> {
        let cursor = self.writer.finish()?;
        Ok(cursor.into_inner())
    }
}
