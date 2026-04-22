//! XLS (BIFF8) reader — skeleton implementation.
//!
//! Legacy `.xls` files use the OLE2 Compound Document format (also known as
//! Microsoft Compound Binary File Format). This module provides the
//! infrastructure for detecting OLE2 files and locating the Workbook stream.
//! Full BIFF8 record parsing will be implemented in a future release.

use crate::error::{Error, Result};

/// OLE2 magic bytes: `D0 CF 11 E0 A1 B1 1A E1`
const OLE2_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// BIFF8 record types.
#[allow(dead_code)]
pub mod biff8 {
    pub const BOF: u16 = 0x0809;
    pub const EOF: u16 = 0x000A;
    pub const SHEET: u16 = 0x0085;
    pub const SST: u16 = 0x00FC;
    pub const LABELSST: u16 = 0x00FD;
    pub const NUMBER: u16 = 0x0203;
    pub const FORMULA: u16 = 0x0006;
    pub const RK: u16 = 0x027E;
    pub const MULRK: u16 = 0x00BD;
    pub const BLANK: u16 = 0x0201;
}

/// OLE2 directory entry (simplified).
#[derive(Debug, Clone)]
pub struct Ole2DirectoryEntry {
    /// Entry name (UTF-16LE decoded).
    pub name: String,
    /// Entry type: 1=storage, 2=stream, 5=root.
    pub entry_type: u8,
    /// Start sector of the stream data.
    pub start_sector: u32,
    /// Stream size in bytes.
    pub size: u32,
}

/// Detect whether a byte slice starts with the OLE2 magic signature.
pub fn is_ole2(data: &[u8]) -> bool {
    data.len() >= 8 && data[..8] == OLE2_MAGIC
}

/// Parse the OLE2 compound document header.
///
/// Returns the sector size and the first directory sector index.
pub fn parse_ole2_header(data: &[u8]) -> Result<(usize, u32)> {
    if !is_ole2(data) {
        return Err(Error::InvalidData("Not an OLE2 compound document".into()));
    }
    if data.len() < 512 {
        return Err(Error::InvalidData("OLE2 header too short".into()));
    }

    // Sector size is at offset 30 (2 bytes, power of 2)
    let sector_power = u16::from_le_bytes([data[30], data[31]]);
    let sector_size = 1usize << sector_power;

    // First directory sector SECID at offset 48
    let first_dir_sector = u32::from_le_bytes([data[48], data[49], data[50], data[51]]);

    Ok((sector_size, first_dir_sector))
}

/// Locate the "Workbook" or "Book" stream in OLE2 directory entries.
pub fn find_workbook_stream(entries: &[Ole2DirectoryEntry]) -> Option<&Ole2DirectoryEntry> {
    entries.iter().find(|e| {
        let name_lower = e.name.to_lowercase();
        name_lower == "workbook" || name_lower == "book"
    })
}

/// Attempt to read an XLS (BIFF8) file. Currently returns `UnsupportedFormat`.
///
/// The infrastructure for OLE2 header parsing and stream location is in place;
/// full BIFF8 record extraction will be implemented in a future release.
pub fn read_xls(_path: &std::path::Path) -> Result<()> {
    Err(Error::UnsupportedFormat(
        "XLS (BIFF8) reading is not yet fully implemented".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ole2_detection() {
        assert!(is_ole2(&OLE2_MAGIC));
        assert!(is_ole2(&[
            0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00
        ]));
        assert!(!is_ole2(&[0x50, 0x4B, 0x03, 0x04])); // ZIP magic
        assert!(!is_ole2(&[0x00; 4]));
    }

    #[test]
    fn test_read_xls_returns_unsupported() {
        let result = read_xls(std::path::Path::new("nonexistent.xls"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::UnsupportedFormat(_)));
    }

    #[test]
    fn test_find_workbook_stream() {
        let entries = vec![
            Ole2DirectoryEntry {
                name: "Root Entry".into(),
                entry_type: 5,
                start_sector: 0,
                size: 0,
            },
            Ole2DirectoryEntry {
                name: "Workbook".into(),
                entry_type: 2,
                start_sector: 1,
                size: 4096,
            },
        ];
        let found = find_workbook_stream(&entries);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Workbook");
    }

    #[test]
    fn test_find_workbook_stream_not_found() {
        let entries = vec![Ole2DirectoryEntry {
            name: "Root Entry".into(),
            entry_type: 5,
            start_sector: 0,
            size: 0,
        }];
        assert!(find_workbook_stream(&entries).is_none());
    }
}
