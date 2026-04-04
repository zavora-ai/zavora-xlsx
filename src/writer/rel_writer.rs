use crate::xml::xml_writer::XmlWriter;

pub fn write_rels(rels: &[(&str, &str, &str)]) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("Relationships", &[("xmlns", "http://schemas.openxmlformats.org/package/2006/relationships")]);
    for &(id, rel_type, target) in rels {
        w.empty_tag("Relationship", &[("Id", id), ("Type", rel_type), ("Target", target)]);
    }
    w.end_tag("Relationships");
    w.into_bytes()
}

#[allow(dead_code)]
pub fn write_root_rels() -> Vec<u8> {
    write_rels(&[
        ("rId1", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", "xl/workbook.xml"),
    ])
}

pub fn write_workbook_rels(sheet_count: usize, has_vba: bool) -> Vec<u8> {
    let mut rels: Vec<(String, &str, String)> = Vec::new();
    for i in 0..sheet_count {
        rels.push((
            format!("rId{}", i + 1),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet",
            format!("worksheets/sheet{}.xml", i + 1),
        ));
    }
    let mut next = sheet_count + 1;
    rels.push((format!("rId{next}"), "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles", "styles.xml".into()));
    next += 1;
    rels.push((format!("rId{next}"), "http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings", "sharedStrings.xml".into()));
    if has_vba {
        next += 1;
        rels.push((format!("rId{next}"), "http://schemas.microsoft.com/office/2006/relationships/vbaProject", "vbaProject.bin".into()));
    }

    let refs: Vec<(&str, &str, &str)> = rels.iter().map(|(a, b, c)| (a.as_str(), *b, c.as_str())).collect();
    write_rels(&refs)
}
