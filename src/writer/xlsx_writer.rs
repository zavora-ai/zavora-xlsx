use crate::model::shared_strings::SharedStringTable;
use crate::model::style_registry::StyleRegistry;
use crate::writer::{content_types_writer, rel_writer, sheet_writer, sst_writer, style_writer};
use crate::writer::sheet_writer::SheetCells;
use crate::xml::xml_writer::XmlWriter;
use crate::zip::zip_writer::ZipOutput;

pub struct WorksheetData<'a> {
    pub name: &'a str,
    pub cells: &'a SheetCells<'a>,
}

pub fn write_xlsx(
    sheets: &[WorksheetData<'_>],
    sst: &SharedStringTable,
    styles: &StyleRegistry,
) -> crate::Result<Vec<u8>> {
    let mut zip = ZipOutput::new();

    // [Content_Types].xml
    zip.add_file("[Content_Types].xml", &content_types_writer::write_content_types(sheets.len()))?;

    // _rels/.rels
    zip.add_file("_rels/.rels", &rel_writer::write_root_rels())?;

    // xl/_rels/workbook.xml.rels
    zip.add_file("xl/_rels/workbook.xml.rels", &rel_writer::write_workbook_rels(sheets.len()))?;

    // xl/workbook.xml
    zip.add_file("xl/workbook.xml", &write_workbook(sheets))?;

    // xl/worksheets/sheet{n}.xml
    for (i, ws) in sheets.iter().enumerate() {
        let path = format!("xl/worksheets/sheet{}.xml", i + 1);
        zip.add_file(&path, &sheet_writer::write_sheet(ws.cells))?;
    }

    // xl/styles.xml
    zip.add_file("xl/styles.xml", &style_writer::write_styles(styles))?;

    // xl/sharedStrings.xml
    zip.add_file("xl/sharedStrings.xml", &sst_writer::write_sst(sst))?;

    zip.finish()
}

fn write_workbook(sheets: &[WorksheetData<'_>]) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("workbook", &[
        ("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
    ]);
    w.start_tag("sheets", &[]);
    for (i, ws) in sheets.iter().enumerate() {
        let id = (i + 1).to_string();
        let rid = format!("rId{}", i + 1);
        w.empty_tag("sheet", &[("name", ws.name), ("sheetId", &id), ("r:id", &rid)]);
    }
    w.end_tag("sheets");
    w.end_tag("workbook");
    w.into_bytes()
}
