use crate::xml::xml_writer::XmlWriter;

pub fn write_content_types(sheet_count: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("Types", &[("xmlns", "http://schemas.openxmlformats.org/package/2006/content-types")]);
    w.empty_tag("Default", &[("Extension", "rels"), ("ContentType", "application/vnd.openxmlformats-package.relationships+xml")]);
    w.empty_tag("Default", &[("Extension", "xml"), ("ContentType", "application/xml")]);
    w.empty_tag("Override", &[("/xl/workbook.xml", "/xl/workbook.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml")].map(|(k, v)| if k.starts_with('/') { ("PartName", v) } else { (k, v) }));

    // Fix: use proper Override elements
    let mut w2 = XmlWriter::new();
    w2.declaration();
    w2.start_tag("Types", &[("xmlns", "http://schemas.openxmlformats.org/package/2006/content-types")]);
    w2.empty_tag("Default", &[("Extension", "rels"), ("ContentType", "application/vnd.openxmlformats-package.relationships+xml")]);
    w2.empty_tag("Default", &[("Extension", "xml"), ("ContentType", "application/xml")]);
    w2.empty_tag("Override", &[("PartName", "/xl/workbook.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml")]);
    for i in 1..=sheet_count {
        let part = format!("/xl/worksheets/sheet{i}.xml");
        w2.empty_tag("Override", &[("PartName", &part), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml")]);
    }
    w2.empty_tag("Override", &[("PartName", "/xl/styles.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml")]);
    w2.empty_tag("Override", &[("PartName", "/xl/sharedStrings.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml")]);
    w2.end_tag("Types");
    w2.into_bytes()
}
