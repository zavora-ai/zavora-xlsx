use crate::features::table::Table;
use crate::utility::col_to_letter;
use crate::xml::xml_writer::XmlWriter;

pub fn write_table_xml(table: &Table, table_id: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    let id_s = table_id.to_string();
    let name = table.name.clone().unwrap_or_else(|| format!("Table{table_id}"));
    let display_name = name.clone();
    let range_ref = format!("{}{}:{}{}",
        col_to_letter(table.first_col), table.first_row + 1,
        col_to_letter(table.last_col), table.last_row + 1);

    w.start_tag("table", &[
        ("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main"),
        ("id", &id_s), ("name", &name), ("displayName", &display_name), ("ref", &range_ref),
        ("totalsRowShown", if table.total_row { "1" } else { "0" }),
    ]);

    if table.autofilter {
        w.empty_tag("autoFilter", &[("ref", &range_ref)]);
    }

    // Columns
    let col_count = table.columns.len();
    if col_count > 0 {
        w.start_tag("tableColumns", &[("count", &col_count.to_string())]);
        for (i, col) in table.columns.iter().enumerate() {
            let cid = (i + 1).to_string();
            let mut attrs: Vec<(&str, &str)> = vec![("id", &cid), ("name", &col.name)];
            if let Some(ref label) = col.total_label {
                attrs.push(("totalsRowLabel", label));
            }
            if let Some(ref func) = col.total_function {
                attrs.push(("totalsRowFunction", func));
            }
            w.empty_tag("tableColumn", &attrs);
        }
        w.end_tag("tableColumns");
    }

    // Style
    if let Some(ref style) = table.style {
        w.empty_tag("tableStyleInfo", &[
            ("name", &style.name()),
            ("showFirstColumn", "0"), ("showLastColumn", "0"),
            ("showRowStripes", "1"), ("showColumnStripes", "0"),
        ]);
    }

    w.end_tag("table");
    w.into_bytes()
}
