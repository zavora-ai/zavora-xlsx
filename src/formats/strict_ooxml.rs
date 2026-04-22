//! Strict OOXML namespace detection and mapping.
//!
//! ISO/IEC 29500 defines two conformance classes for OOXML:
//! - **Transitional** (the common format, used by default)
//! - **Strict** (uses different namespace URIs)
//!
//! This module detects Strict namespace URIs and maps them to their
//! Transitional equivalents so the existing reader can process them.

/// Strict OOXML namespace URIs.
pub mod strict_ns {
    pub const SPREADSHEET_ML: &str = "http://purl.oclc.org/ooxml/spreadsheetml/main";
    pub const RELATIONSHIPS: &str = "http://purl.oclc.org/ooxml/officeDocument/relationships";
    pub const DRAWING_ML: &str = "http://purl.oclc.org/ooxml/drawingml/main";
    pub const CONTENT_TYPES: &str = "http://purl.oclc.org/ooxml/presentationml/main";
    pub const OFFICE_DOC_RELS: &str =
        "http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument";
}

/// Transitional OOXML namespace URIs (the standard ones used by the crate).
pub mod transitional_ns {
    pub const SPREADSHEET_ML: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
    pub const RELATIONSHIPS: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    pub const DRAWING_ML: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
    pub const CONTENT_TYPES: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
    pub const OFFICE_DOC_RELS: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
}

/// A mapping entry from Strict to Transitional namespace.
struct NsMapping {
    strict: &'static str,
    transitional: &'static str,
}

/// All known Strict → Transitional namespace mappings.
const NS_MAPPINGS: &[NsMapping] = &[
    NsMapping {
        strict: strict_ns::SPREADSHEET_ML,
        transitional: transitional_ns::SPREADSHEET_ML,
    },
    NsMapping {
        strict: strict_ns::RELATIONSHIPS,
        transitional: transitional_ns::RELATIONSHIPS,
    },
    NsMapping {
        strict: strict_ns::DRAWING_ML,
        transitional: transitional_ns::DRAWING_ML,
    },
    NsMapping {
        strict: strict_ns::OFFICE_DOC_RELS,
        transitional: transitional_ns::OFFICE_DOC_RELS,
    },
];

/// Detect whether an XML document uses Strict OOXML namespaces.
///
/// Checks if the content contains any known Strict namespace URI.
pub fn is_strict_ooxml(xml_content: &[u8]) -> bool {
    let s = String::from_utf8_lossy(xml_content);
    NS_MAPPINGS.iter().any(|m| s.contains(m.strict))
}

/// Map a Strict namespace URI to its Transitional equivalent.
///
/// Returns the Transitional URI if a mapping exists, or `None` if the
/// input is not a known Strict namespace.
pub fn map_strict_to_transitional(strict_uri: &str) -> Option<&'static str> {
    NS_MAPPINGS
        .iter()
        .find(|m| m.strict == strict_uri)
        .map(|m| m.transitional)
}

/// Convert all Strict namespace URIs in an XML byte slice to Transitional.
///
/// This performs a simple string replacement, which is sufficient for
/// namespace URIs that appear in attribute values. The returned bytes
/// can then be processed by the standard Transitional reader.
pub fn convert_strict_to_transitional(xml_content: &[u8]) -> Vec<u8> {
    let mut s = String::from_utf8_lossy(xml_content).into_owned();
    for mapping in NS_MAPPINGS {
        s = s.replace(mapping.strict, mapping.transitional);
    }
    s.into_bytes()
}

/// Generate Strict OOXML output for a workbook.
///
/// Currently returns `UnsupportedFormat` — Strict output will be implemented
/// in a future release. Reading Strict files is supported via namespace mapping.
pub fn save_strict(_path: &std::path::Path) -> crate::Result<()> {
    Err(crate::Error::UnsupportedFormat(
        "Strict OOXML output is not yet implemented; use convert_strict_to_transitional() for reading".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_strict_ooxml() {
        let strict_xml = br#"<worksheet xmlns="http://purl.oclc.org/ooxml/spreadsheetml/main"><sheetData/></worksheet>"#;
        assert!(is_strict_ooxml(strict_xml));
    }

    #[test]
    fn test_detect_transitional_not_strict() {
        let transitional_xml = br#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData/></worksheet>"#;
        assert!(!is_strict_ooxml(transitional_xml));
    }

    #[test]
    fn test_map_strict_to_transitional() {
        assert_eq!(
            map_strict_to_transitional(strict_ns::SPREADSHEET_ML),
            Some(transitional_ns::SPREADSHEET_ML)
        );
        assert_eq!(
            map_strict_to_transitional(strict_ns::RELATIONSHIPS),
            Some(transitional_ns::RELATIONSHIPS)
        );
        assert_eq!(map_strict_to_transitional("http://unknown.namespace"), None);
    }

    #[test]
    fn test_convert_strict_to_transitional() {
        let strict = format!(r#"<worksheet xmlns="{}">"#, strict_ns::SPREADSHEET_ML);
        let result = convert_strict_to_transitional(strict.as_bytes());
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains(transitional_ns::SPREADSHEET_ML));
        assert!(!result_str.contains(strict_ns::SPREADSHEET_ML));
    }

    #[test]
    fn test_convert_multiple_namespaces() {
        let strict = format!(
            r#"<worksheet xmlns="{}" xmlns:r="{}">"#,
            strict_ns::SPREADSHEET_ML,
            strict_ns::RELATIONSHIPS,
        );
        let result = convert_strict_to_transitional(strict.as_bytes());
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains(transitional_ns::SPREADSHEET_ML));
        assert!(result_str.contains(transitional_ns::RELATIONSHIPS));
    }

    #[test]
    fn test_save_strict_returns_unsupported() {
        let result = save_strict(std::path::Path::new("test.xlsx"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::Error::UnsupportedFormat(_)
        ));
    }
}
