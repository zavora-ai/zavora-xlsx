use crate::model::shared_strings::SharedStringTable;
use crate::xml::xml_writer::XmlWriter;

pub fn write_sst(sst: &SharedStringTable) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    let count = sst.len().to_string();
    w.start_tag(
        "sst",
        &[
            (
                "xmlns",
                "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
            ),
            ("count", &count),
            ("uniqueCount", &count),
        ],
    );
    for s in sst.iter() {
        w.start_tag("si", &[]);
        w.text_element("t", &[], s);
        w.end_tag("si");
    }
    w.end_tag("sst");
    w.into_bytes()
}
