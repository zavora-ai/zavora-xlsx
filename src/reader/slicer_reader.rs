//! Reader for slicer XML parts.

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::features::slicer::Slicer;
use crate::xml::xml_reader::get_attr;

/// Parse a slicer XML part (xl/slicers/slicer{N}.xml) into a list of Slicers.
pub fn parse_slicers(data: &[u8]) -> Vec<Slicer> {
    let mut slicers = Vec::new();
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(1024);

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"slicer" {
                    let name = get_attr(e.attributes(), b"name")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .unwrap_or("")
                        .to_string();
                    let caption = get_attr(e.attributes(), b"caption")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .unwrap_or("")
                        .to_string();
                    let cache = get_attr(e.attributes(), b"cache")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .unwrap_or("")
                        .to_string();
                    let style = get_attr(e.attributes(), b"style")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .unwrap_or("SlicerStyleLight1")
                        .to_string();

                    // Derive source_name from cache name (Slicer_ColumnName → ColumnName)
                    let source_name = if let Some(stripped) = cache.strip_prefix("Slicer_") {
                        stripped.replace('_', " ")
                    } else {
                        caption.clone()
                    };

                    let slicer = Slicer {
                        name,
                        caption,
                        source_name,
                        row: 0,
                        col: 0,
                        width: 200,
                        height: 300,
                        style,
                        pivot_cache_name: None,
                        table_id: None,
                        column_index: None,
                    };
                    slicers.push(slicer);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    slicers
}

/// Find slicer paths from sheet relationships data.
pub fn find_slicer_paths_from_rels(sheet_rels_data: &[u8]) -> Vec<String> {
    let rels = match crate::reader::rel_parser::parse_rels(sheet_rels_data) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let slicer_rel_type = "http://schemas.microsoft.com/office/2007/relationships/slicer";
    rels.iter()
        .filter(|r| r.rel_type == slicer_rel_type)
        .map(|r| {
            let target = &r.target;
            if let Some(stripped) = target.strip_prefix("../") {
                format!("xl/{}", stripped)
            } else if let Some(stripped) = target.strip_prefix("/xl/") {
                stripped.to_string()
            } else if target.starts_with("xl/") {
                target.to_string()
            } else {
                format!("xl/slicers/{target}")
            }
        })
        .collect()
}
