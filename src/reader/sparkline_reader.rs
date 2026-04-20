//! Parser for sparkline groups from `<extLst>` in sheet XML.
//!
//! Sparklines in OOXML are stored inside extension elements at the end of the
//! sheet XML, using the `x14` namespace
//! (`http://schemas.microsoft.com/office/spreadsheetml/2009/9/main`).
//!
//! The structure is:
//! ```xml
//! <extLst>
//!   <ext uri="{05C60535-1F16-4fd2-B633-F4F36F0B64E0}"
//!        xmlns:x14="http://schemas.microsoft.com/office/spreadsheetml/2009/9/main">
//!     <x14:sparklineGroups xmlns:xm="http://schemas.microsoft.com/office/excel/2006/main">
//!       <x14:sparklineGroup type="line|column|stacked">
//!         <x14:sparklines>
//!           <x14:sparkline>
//!             <xm:f>Sheet1!B1:B10</xm:f>
//!             <xm:sqref>A1</xm:sqref>
//!           </x14:sparkline>
//!         </x14:sparklines>
//!       </x14:sparklineGroup>
//!     </x14:sparklineGroups>
//!   </ext>
//! </extLst>
//! ```

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::features::sparkline::{Sparkline, SparklineType};
use crate::utility::parse_cell_ref;
use crate::xml::xml_reader::get_attr;

/// Parse all sparklines from the `<extLst>` section of sheet XML.
///
/// `data` is the raw XML bytes of the entire sheet. The parser scans for
/// `<x14:sparklineGroup>` elements inside `<extLst>` and extracts sparkline
/// data range, location, and type.
///
/// Returns an empty `Vec` when no sparklines are present.
pub fn parse_sparklines(data: &[u8]) -> Vec<Sparkline> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = false;
    let mut buf = Vec::with_capacity(1024);
    let mut results = Vec::new();

    // Scan for sparklineGroup elements (with or without x14: prefix)
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"sparklineGroup" {
                    let sp_type = parse_sparkline_type(
                        get_attr(e.attributes(), b"type")
                            .and_then(|v| std::str::from_utf8(v).ok())
                            .unwrap_or("line"),
                    );
                    // Parse the color attribute if present within the group
                    let color = parse_sparkline_group_color(&mut reader, &mut buf, sp_type, &mut results);
                    // If parse_sparkline_group_color didn't consume the end tag, color is returned
                    let _ = color;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    results
}

/// Parse the sparkline type from the XML attribute value.
fn parse_sparkline_type(s: &str) -> SparklineType {
    match s {
        "column" => SparklineType::Column,
        "stacked" => SparklineType::WinLoss,
        _ => SparklineType::Line,
    }
}

/// Parse the contents of a `<x14:sparklineGroup>` element, extracting
/// individual sparklines with their data ranges and locations.
///
/// Also captures the optional `<x14:colorSeries>` color.
fn parse_sparkline_group_color(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    sp_type: SparklineType,
    results: &mut Vec<Sparkline>,
) -> Option<[u8; 3]> {
    let mut color: Option<[u8; 3]> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"colorSeries" => {
                        // The color is in a child <x14:rgbColor> or <x14:color> element
                        color = parse_color_element(reader, buf);
                    }
                    b"sparkline" => {
                        if let Some(sp) = parse_single_sparkline(reader, buf, sp_type, color) {
                            results.push(sp);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"rgbColor" || local.as_ref() == b"color" {
                    // Sometimes colorSeries has inline rgbColor
                    if let Some(rgb) = parse_rgb_from_attrs(e.attributes()) {
                        color = Some(rgb);
                    }
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"sparklineGroup" => break,
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    color
}

/// Parse a `<x14:colorSeries>` element to extract the RGB color.
fn parse_color_element(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Option<[u8; 3]> {
    let mut color = None;
    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"rgbColor" || local.as_ref() == b"color" {
                    color = parse_rgb_from_attrs(e.attributes());
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"colorSeries" => break,
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    color
}

/// Parse a single `<x14:sparkline>` element, extracting the data range
/// from `<xm:f>` and the location from `<xm:sqref>`.
fn parse_single_sparkline(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    sp_type: SparklineType,
    color: Option<[u8; 3]>,
) -> Option<Sparkline> {
    let mut data_range = String::new();
    let mut location = String::new();
    let mut in_f = false;
    let mut in_sqref = false;

    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"f" => {
                        in_f = true;
                        data_range.clear();
                    }
                    b"sqref" => {
                        in_sqref = true;
                        location.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref t)) => {
                if in_f {
                    if let Ok(s) = t.unescape() {
                        data_range.push_str(&s);
                    }
                } else if in_sqref {
                    if let Ok(s) = t.unescape() {
                        location.push_str(&s);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"f" => in_f = false,
                    b"sqref" => in_sqref = false,
                    b"sparkline" => break,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    if data_range.is_empty() || location.is_empty() {
        return None;
    }

    // Parse the location cell reference (e.g. "A1") to get row/col
    let (row, col) = parse_cell_ref(&location).ok()?;

    let mut sparkline = Sparkline::new(&data_range, sp_type);
    sparkline.row = row;
    sparkline.col = col;
    sparkline.color = color;

    Some(sparkline)
}

/// Parse an RGB color from `rgb="FFRRGGBB"` or `rgb="RRGGBB"` attribute.
fn parse_rgb_from_attrs(attrs: quick_xml::events::attributes::Attributes<'_>) -> Option<[u8; 3]> {
    let val = get_attr(attrs, b"rgb")?;
    let s = std::str::from_utf8(val).ok()?;
    let hex = if s.len() >= 8 {
        &s[2..8]
    } else if s.len() >= 6 {
        &s[0..6]
    } else {
        return None;
    };
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}
