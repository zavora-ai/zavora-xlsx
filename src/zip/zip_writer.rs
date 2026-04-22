use std::io::{Cursor, Write};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// Wraps ZipWriter for building xlsx archives.
pub struct ZipOutput {
    writer: ZipWriter<Cursor<Vec<u8>>>,
}

impl ZipOutput {
    pub fn new() -> Self {
        Self {
            writer: ZipWriter::new(Cursor::new(Vec::new())),
        }
    }

    /// Add a file entry with deflate compression.
    pub fn add_file(&mut self, path: &str, data: &[u8]) -> crate::Result<()> {
        let opts =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        self.writer.start_file(path, opts)?;
        self.writer.write_all(data)?;
        Ok(())
    }

    /// Finish writing and return the zip bytes.
    pub fn finish(self) -> crate::Result<Vec<u8>> {
        let cursor = self.writer.finish()?;
        Ok(cursor.into_inner())
    }
}
