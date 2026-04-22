//! ODS (OpenDocument Spreadsheet) reader/writer — skeleton implementation.
//!
//! ODS files use the OpenDocument format (ISO/IEC 26300) with `content.xml`
//! containing `<table:table>` elements. This module provides the infrastructure
//! for detecting ODS files and basic read/write skeletons.

use crate::error::{Error, Result};

/// ODS MIME type.
pub const ODS_MIMETYPE: &str = "application/vnd.oasis.opendocument.spreadsheet";

/// ODS XML namespaces.
pub mod ns {
    pub const OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
    pub const TABLE: &str = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";
    pub const TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
    pub const STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
    pub const FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
}

/// Detect whether a ZIP archive contains ODS content (by checking for `mimetype` entry).
pub fn is_ods(mimetype_content: &[u8]) -> bool {
    let s = String::from_utf8_lossy(mimetype_content);
    s.trim() == ODS_MIMETYPE
}

/// Attempt to read an ODS file. Currently returns `UnsupportedFormat`.
///
/// The infrastructure for ODS detection and namespace definitions is in place;
/// full `content.xml` parsing will be implemented in a future release.
pub fn read_ods(_path: &std::path::Path) -> Result<()> {
    Err(Error::UnsupportedFormat(
        "ODS (OpenDocument Spreadsheet) reading is not yet fully implemented".into(),
    ))
}

/// Attempt to write an ODS file. Currently returns `UnsupportedFormat`.
///
/// The infrastructure for ODS namespace definitions is in place;
/// full serialization will be implemented in a future release.
pub fn write_ods(_path: &std::path::Path) -> Result<()> {
    Err(Error::UnsupportedFormat(
        "ODS (OpenDocument Spreadsheet) writing is not yet fully implemented".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ods_detection() {
        assert!(is_ods(b"application/vnd.oasis.opendocument.spreadsheet"));
        assert!(is_ods(b"application/vnd.oasis.opendocument.spreadsheet\n"));
        assert!(!is_ods(
            b"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        ));
    }

    #[test]
    fn test_read_ods_returns_unsupported() {
        let result = read_ods(std::path::Path::new("nonexistent.ods"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::UnsupportedFormat(_)));
    }

    #[test]
    fn test_write_ods_returns_unsupported() {
        let result = write_ods(std::path::Path::new("nonexistent.ods"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::UnsupportedFormat(_)));
    }

    #[test]
    fn test_ods_namespaces() {
        assert!(ns::OFFICE.contains("opendocument"));
        assert!(ns::TABLE.contains("table"));
        assert!(ns::TEXT.contains("text"));
    }
}
