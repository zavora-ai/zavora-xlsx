//! XLSB (Excel Binary Workbook) reader — skeleton implementation.
//!
//! XLSB files use a binary record format instead of XML inside the ZIP container.
//! This module provides the infrastructure for detecting and eventually parsing
//! XLSB files. Currently returns `UnsupportedFormat` for actual parsing.

use crate::error::{Error, Result};

/// Binary record header: record type ID + payload length.
#[derive(Debug, Clone)]
pub struct BinaryRecord {
    /// Record type identifier.
    pub record_type: u16,
    /// Raw payload bytes.
    pub payload: Vec<u8>,
}

/// Detect whether a ZIP archive contains XLSB content types.
///
/// XLSB files use `application/vnd.ms-excel.sheet.binary.macroEnabled.main`
/// or similar content types in `[Content_Types].xml`.
pub fn is_xlsb(content_types_xml: &[u8]) -> bool {
    let s = String::from_utf8_lossy(content_types_xml);
    s.contains("application/vnd.ms-excel.sheet.binary")
}

/// Parse a binary record stream from raw bytes.
///
/// XLSB records have a variable-length header:
/// - Record type: 1 or 2 bytes (if high bit of first byte is set, read second byte)
/// - Record size: 1–4 bytes (variable-length encoding)
pub fn parse_records(data: &[u8]) -> Result<Vec<BinaryRecord>> {
    let mut records = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        // Parse record type (1 or 2 bytes)
        if pos >= data.len() {
            break;
        }
        let first = data[pos];
        pos += 1;
        let record_type = if first & 0x80 != 0 {
            if pos >= data.len() {
                break;
            }
            let second = data[pos];
            pos += 1;
            ((second as u16) << 7) | (first as u16 & 0x7F)
        } else {
            first as u16
        };

        // Parse record size (variable-length, 1–4 bytes)
        let mut size: u32 = 0;
        let mut shift = 0;
        for _ in 0..4 {
            if pos >= data.len() {
                return Err(Error::InvalidData("Truncated XLSB record size".into()));
            }
            let b = data[pos];
            pos += 1;
            size |= ((b & 0x7F) as u32) << shift;
            shift += 7;
            if b & 0x80 == 0 {
                break;
            }
        }

        let size = size as usize;
        if pos + size > data.len() {
            break; // Truncated — skip gracefully
        }

        let payload = data[pos..pos + size].to_vec();
        pos += size;

        records.push(BinaryRecord {
            record_type,
            payload,
        });
    }

    Ok(records)
}

/// Attempt to read an XLSB file. Currently returns `UnsupportedFormat`.
///
/// The infrastructure for binary record parsing is in place; full cell
/// extraction will be implemented in a future release.
pub fn read_xlsb(_path: &std::path::Path) -> Result<()> {
    Err(Error::UnsupportedFormat(
        "XLSB (Excel Binary Workbook) reading is not yet fully implemented".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_xlsb_detection() {
        let ct = br#"<?xml version="1.0"?><Types><Override ContentType="application/vnd.ms-excel.sheet.binary.macroEnabled.main"/></Types>"#;
        assert!(is_xlsb(ct));

        let ct_xlsx = br#"<?xml version="1.0"?><Types><Override ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/></Types>"#;
        assert!(!is_xlsb(ct_xlsx));
    }

    #[test]
    fn test_read_xlsb_returns_unsupported() {
        let result = read_xlsb(std::path::Path::new("nonexistent.xlsb"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::UnsupportedFormat(_)));
    }

    #[test]
    fn test_parse_empty_records() {
        let records = parse_records(&[]).unwrap();
        assert!(records.is_empty());
    }
}
