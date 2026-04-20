//! Parser for table XML parts (`xl/tables/table{N}.xml`).
//!
//! Converts raw table XML bytes into [`Table`] structs by parsing the
//! `<table>` root element, its `<autoFilter>`, `<tableColumns>`, and
//! `<tableStyleInfo>` children.

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::features::table::{Table, TableColumn, TableStyle};
use crate::utility::parse_range;
use crate::xml::xml_reader::get_attr_str;

/// Parse a table XML part (`xl/tables/table{N}.xml`) into a [`Table`] struct.
pub fn read_table(data: &[u8]) -> crate::Result<Table> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(1024);

    let mut table = Table::new();
    // Default to false; set to true only if <autoFilter> is found in the XML.
    table.autofilter = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                let name = local.as_ref();
                match name {
                    b"table" => {
                        if let Some(n) = get_attr_str(e.attributes(), b"name") {
                            table.name = Some(n.to_string());
                        }
                        if let Some(range_str) = get_attr_str(e.attributes(), b"ref") {
                            if let Some((r1, c1, r2, c2)) = parse_range(range_str) {
                                table.first_row = r1;
                                table.first_col = c1;
                                table.last_row = r2;
                                table.last_col = c2;
                            }
                        }
                        if let Some(v) = get_attr_str(e.attributes(), b"totalsRowShown") {
                            table.total_row = v == "1";
                        }
                        if let Some(v) = get_attr_str(e.attributes(), b"totalsRowCount") {
                            if v != "0" {
                                table.total_row = true;
                            }
                        }
                    }
                    b"autoFilter" => {
                        table.autofilter = true;
                    }
                    b"tableColumn" => {
                        let col_name = get_attr_str(e.attributes(), b"name")
                            .unwrap_or("")
                            .to_string();
                        let mut col = TableColumn::new(&col_name);
                        if let Some(label) = get_attr_str(e.attributes(), b"totalsRowLabel") {
                            col.set_total_label(label);
                        }
                        if let Some(func) = get_attr_str(e.attributes(), b"totalsRowFunction") {
                            col.set_total_function(func);
                        }
                        table.columns.push(col);
                    }
                    b"tableStyleInfo" => {
                        if let Some(style_name) = get_attr_str(e.attributes(), b"name") {
                            if let Some(style) = resolve_table_style(style_name) {
                                table.style = Some(style);
                            }
                        }
                    }
                    _ => {
                        // Skip unknown elements gracefully
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    Ok(table)
}

/// Resolve a table style name string (e.g. "TableStyleMedium9") into a
/// [`TableStyle`] enum variant.
pub fn resolve_table_style(name: &str) -> Option<TableStyle> {
    if let Some(rest) = name.strip_prefix("TableStyleLight") {
        rest.parse::<u8>().ok().map(TableStyle::Light)
    } else if let Some(rest) = name.strip_prefix("TableStyleMedium") {
        rest.parse::<u8>().ok().map(TableStyle::Medium)
    } else if let Some(rest) = name.strip_prefix("TableStyleDark") {
        rest.parse::<u8>().ok().map(TableStyle::Dark)
    } else {
        None
    }
}
