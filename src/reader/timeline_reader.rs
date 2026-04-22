//! Reader for timeline XML parts.

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::features::timeline::{Timeline, TimelineLevel};
use crate::xml::xml_reader::get_attr;

/// Parse a timeline XML part (xl/timelines/timeline{N}.xml) into a list of Timelines.
pub fn parse_timelines(data: &[u8]) -> Vec<Timeline> {
    let mut timelines = Vec::new();
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(1024);

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"timeline" {
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
                    let level_val = get_attr(e.attributes(), b"level")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .and_then(|v| v.parse::<u8>().ok())
                        .unwrap_or(2);

                    let level = match level_val {
                        0 => TimelineLevel::Years,
                        1 => TimelineLevel::Quarters,
                        2 => TimelineLevel::Months,
                        3 => TimelineLevel::Days,
                        _ => TimelineLevel::Months,
                    };

                    // Derive source_name from cache name (NativeTimeline_FieldName → FieldName)
                    let source_name = if let Some(stripped) = cache.strip_prefix("NativeTimeline_")
                    {
                        stripped.replace('_', " ")
                    } else {
                        caption.clone()
                    };

                    let timeline = Timeline {
                        name,
                        caption,
                        source_name,
                        row: 0,
                        col: 0,
                        level,
                    };
                    timelines.push(timeline);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    timelines
}

/// Find timeline paths from sheet relationships data.
pub fn find_timeline_paths_from_rels(sheet_rels_data: &[u8]) -> Vec<String> {
    let rels = match crate::reader::rel_parser::parse_rels(sheet_rels_data) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let timeline_rel_type = "http://schemas.microsoft.com/office/2011/relationships/timeline";
    rels.iter()
        .filter(|r| r.rel_type == timeline_rel_type)
        .map(|r| {
            let target = &r.target;
            if let Some(stripped) = target.strip_prefix("../") {
                format!("xl/{}", stripped)
            } else if let Some(stripped) = target.strip_prefix("/xl/") {
                stripped.to_string()
            } else if target.starts_with("xl/") {
                target.to_string()
            } else {
                format!("xl/timelines/{target}")
            }
        })
        .collect()
}
