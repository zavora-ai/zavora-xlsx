//! Constant-memory write mode: rows are serialized to XML immediately,
//! so cell data is not retained in memory. Rows must be written in ascending order.

use std::fmt::Write;

use crate::cell::CellType;
use crate::format::Format;
use crate::model::shared_strings::SharedStringTable;
use crate::model::style_registry::StyleRegistry;
use crate::utility::{col_to_letter, ColNum, RowNum};
use crate::xml::xml_writer::XmlWriter;
use crate::zip::zip_writer::ZipOutput;
use crate::writer::{sst_writer, style_writer, rel_writer};
use crate::properties::{self, DocProperties};

/// A workbook that writes rows in streaming mode for minimal memory usage.
/// Rows must be written in ascending order per sheet.
pub struct StreamingWorkbook {
    sheets: Vec<StreamingSheet>,
    sst: SharedStringTable,
    styles: StyleRegistry,
    properties: DocProperties,
    current_sheet: usize,
}

struct StreamingSheet {
    name: String,
    xml_buf: Vec<u8>,
    last_row: Option<RowNum>,
    row_open: bool,
    header_written: bool,
    col_widths: Vec<(ColNum, f64)>,
    merge_ranges: Vec<(RowNum, ColNum, RowNum, ColNum)>,
}

impl StreamingWorkbook {
    pub fn new() -> Self {
        Self {
            sheets: vec![StreamingSheet::new("Sheet1")],
            sst: SharedStringTable::new(),
            styles: StyleRegistry::new(),
            properties: DocProperties::default(),
            current_sheet: 0,
        }
    }

    pub fn add_worksheet(&mut self, name: &str) -> usize {
        let idx = self.sheets.len();
        self.sheets.push(StreamingSheet::new(name));
        self.current_sheet = idx;
        idx
    }

    pub fn set_current_sheet(&mut self, idx: usize) {
        self.current_sheet = idx;
    }

    pub fn set_column_width(&mut self, col: ColNum, width: f64) {
        self.sheets[self.current_sheet].col_widths.push((col, width));
    }

    pub fn merge_range(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum) {
        self.sheets[self.current_sheet].merge_ranges.push((r1, c1, r2, c2));
    }

    /// Write a string to a cell. Rows must be written in ascending order.
    pub fn write_string(&mut self, row: RowNum, col: ColNum, s: &str) -> crate::Result<()> {
        self.write_string_fmt(row, col, s, None)
    }

    pub fn write_string_with_format(&mut self, row: RowNum, col: ColNum, s: &str, fmt: &Format) -> crate::Result<()> {
        self.write_string_fmt(row, col, s, Some(fmt))
    }

    fn write_string_fmt(&mut self, row: RowNum, col: ColNum, s: &str, fmt: Option<&Format>) -> crate::Result<()> {
        let idx = self.sst.intern(s);
        let xf = fmt.map(|f| self.styles.register_format(f)).unwrap_or(0);
        let sheet = &mut self.sheets[self.current_sheet];
        sheet.ensure_header();
        sheet.ensure_row(row)?;
        sheet.write_cell_xml(row, col, &CellType::SharedString(idx), xf);
        Ok(())
    }

    /// Write a number to a cell.
    pub fn write_number(&mut self, row: RowNum, col: ColNum, n: f64) -> crate::Result<()> {
        self.write_number_fmt(row, col, n, None)
    }

    pub fn write_number_with_format(&mut self, row: RowNum, col: ColNum, n: f64, fmt: &Format) -> crate::Result<()> {
        self.write_number_fmt(row, col, n, Some(fmt))
    }

    fn write_number_fmt(&mut self, row: RowNum, col: ColNum, n: f64, fmt: Option<&Format>) -> crate::Result<()> {
        let xf = fmt.map(|f| self.styles.register_format(f)).unwrap_or(0);
        let sheet = &mut self.sheets[self.current_sheet];
        sheet.ensure_header();
        sheet.ensure_row(row)?;
        sheet.write_cell_xml(row, col, &CellType::Number(n), xf);
        Ok(())
    }

    /// Write a boolean to a cell.
    pub fn write_boolean(&mut self, row: RowNum, col: ColNum, b: bool) -> crate::Result<()> {
        let sheet = &mut self.sheets[self.current_sheet];
        sheet.ensure_header();
        sheet.ensure_row(row)?;
        sheet.write_cell_xml(row, col, &CellType::Bool(b), 0);
        Ok(())
    }

    /// Write a formula to a cell.
    pub fn write_formula(&mut self, row: RowNum, col: ColNum, formula: &str) -> crate::Result<()> {
        let sheet = &mut self.sheets[self.current_sheet];
        sheet.ensure_header();
        sheet.ensure_row(row)?;
        sheet.write_cell_xml(row, col, &CellType::Formula { text: formula.to_string(), cached_number: None }, 0);
        Ok(())
    }

    pub fn set_properties(&mut self, props: DocProperties) {
        self.properties = props;
    }

    /// Save to file.
    pub fn save(mut self, path: impl AsRef<std::path::Path>) -> crate::Result<()> {
        let bytes = self.save_to_buffer()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Save to buffer.
    pub fn save_to_buffer(&mut self) -> crate::Result<Vec<u8>> {
        // Finalize all sheets
        for sheet in &mut self.sheets {
            sheet.finalize();
        }

        let mut zip = ZipOutput::new();
        let sheet_count = self.sheets.len();
        let has_props = self.properties.title.is_some() || self.properties.author.is_some();

        zip.add_file("[Content_Types].xml", &write_streaming_content_types(sheet_count, has_props))?;
        zip.add_file("_rels/.rels", &write_streaming_root_rels(has_props))?;
        zip.add_file("xl/_rels/workbook.xml.rels", &rel_writer::write_workbook_rels(sheet_count, false, 0))?;
        zip.add_file("xl/workbook.xml", &write_streaming_workbook(&self.sheets))?;

        for (i, sheet) in self.sheets.iter().enumerate() {
            let path = format!("xl/worksheets/sheet{}.xml", i + 1);
            zip.add_file(&path, &sheet.xml_buf)?;
        }

        zip.add_file("xl/styles.xml", &style_writer::write_styles(&self.styles))?;
        zip.add_file("xl/sharedStrings.xml", &sst_writer::write_sst(&self.sst))?;
        zip.add_file("xl/theme/theme1.xml", &crate::writer::theme_writer::write_theme())?;

        if has_props {
            zip.add_file("docProps/core.xml", &properties::write_core_xml(&self.properties))?;
            zip.add_file("docProps/app.xml", &properties::write_app_xml(&self.properties))?;
        }

        zip.finish()
    }
}

impl Default for StreamingWorkbook {
    fn default() -> Self { Self::new() }
}

impl StreamingSheet {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            xml_buf: Vec::with_capacity(4096),
            last_row: None,
            row_open: false,
            header_written: false,
            col_widths: Vec::new(),
            merge_ranges: Vec::new(),
        }
    }

    fn ensure_header(&mut self) {
        if self.header_written { return; }
        self.header_written = true;
        let mut w = XmlWriter::new();
        w.declaration();
        w.start_tag("worksheet", &[
            ("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main"),
            ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
        ]);
        if !self.col_widths.is_empty() {
            w.start_tag("cols", &[]);
            for &(col, width) in &self.col_widths {
                let c = (col + 1).to_string();
                let ws = format!("{width:.2}");
                w.empty_tag("col", &[("min", &c), ("max", &c), ("width", &ws), ("customWidth", "1")]);
            }
            w.end_tag("cols");
        }
        w.start_tag("sheetData", &[]);
        self.xml_buf.extend_from_slice(&w.into_bytes());
    }

    fn ensure_row(&mut self, row: RowNum) -> crate::Result<()> {
        if let Some(last) = self.last_row {
            if row < last {
                return Err(crate::Error::InvalidData(
                    format!("Streaming mode requires ascending row order: got row {} after row {}", row, last),
                ));
            }
            if row != last {
                // Close previous row
                self.xml_buf.extend_from_slice(b"</row>");
                self.row_open = false;
            }
        }
        if !self.row_open || self.last_row != Some(row) {
            let r = (row + 1).to_string();
            self.xml_buf.extend_from_slice(b"<row r=\"");
            self.xml_buf.extend_from_slice(r.as_bytes());
            self.xml_buf.extend_from_slice(b"\">");
            self.row_open = true;
            self.last_row = Some(row);
        }
        Ok(())
    }

    fn write_cell_xml(&mut self, row: RowNum, col: ColNum, cell: &CellType, xf: u32) {
        let ref_str = format!("{}{}", col_to_letter(col), row + 1);
        let xf_s = xf.to_string();
        let mut w = XmlWriter::new();

        match cell {
            CellType::Number(n) => {
                let mut v = String::new();
                let _ = write!(v, "{n}");
                if xf > 0 { w.start_tag("c", &[("r", &ref_str), ("s", &xf_s)]); }
                else { w.start_tag("c", &[("r", &ref_str)]); }
                w.text_element("v", &[], &v);
                w.end_tag("c");
            }
            CellType::SharedString(idx) => {
                let v = idx.to_string();
                let mut attrs: Vec<(&str, &str)> = vec![("r", &ref_str), ("t", "s")];
                if xf > 0 { attrs.push(("s", &xf_s)); }
                w.start_tag("c", &attrs);
                w.text_element("v", &[], &v);
                w.end_tag("c");
            }
            CellType::Bool(b) => {
                let v = if *b { "1" } else { "0" };
                let mut attrs: Vec<(&str, &str)> = vec![("r", &ref_str), ("t", "b")];
                if xf > 0 { attrs.push(("s", &xf_s)); }
                w.start_tag("c", &attrs);
                w.text_element("v", &[], v);
                w.end_tag("c");
            }
            CellType::Formula { text, cached_number } => {
                if xf > 0 { w.start_tag("c", &[("r", &ref_str), ("s", &xf_s)]); }
                else { w.start_tag("c", &[("r", &ref_str)]); }
                w.text_element("f", &[], text);
                if let Some(n) = cached_number {
                    let mut v = String::new();
                    let _ = write!(v, "{n}");
                    w.text_element("v", &[], &v);
                }
                w.end_tag("c");
            }
            CellType::DateTime(serial) => {
                let mut v = String::new();
                let _ = write!(v, "{serial}");
                if xf > 0 { w.start_tag("c", &[("r", &ref_str), ("s", &xf_s)]); }
                else { w.start_tag("c", &[("r", &ref_str)]); }
                w.text_element("v", &[], &v);
                w.end_tag("c");
            }
            _ => {}
        }
        self.xml_buf.extend_from_slice(&w.into_bytes());
    }

    fn finalize(&mut self) {
        if !self.header_written { self.ensure_header(); }
        if self.row_open {
            self.xml_buf.extend_from_slice(b"</row>");
        }
        self.xml_buf.extend_from_slice(b"</sheetData>");

        if !self.merge_ranges.is_empty() {
            let mut w = XmlWriter::new();
            let count = self.merge_ranges.len().to_string();
            w.start_tag("mergeCells", &[("count", &count)]);
            for &(r1, c1, r2, c2) in &self.merge_ranges {
                let ref_str = format!("{}{}:{}{}", col_to_letter(c1), r1 + 1, col_to_letter(c2), r2 + 1);
                w.empty_tag("mergeCell", &[("ref", &ref_str)]);
            }
            w.end_tag("mergeCells");
            self.xml_buf.extend_from_slice(&w.into_bytes());
        }

        self.xml_buf.extend_from_slice(b"</worksheet>");
    }
}

fn write_streaming_content_types(sheet_count: usize, has_props: bool) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("Types", &[("xmlns", "http://schemas.openxmlformats.org/package/2006/content-types")]);
    w.empty_tag("Default", &[("Extension", "rels"), ("ContentType", "application/vnd.openxmlformats-package.relationships+xml")]);
    w.empty_tag("Default", &[("Extension", "xml"), ("ContentType", "application/xml")]);
    w.empty_tag("Override", &[("PartName", "/xl/workbook.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml")]);
    for i in 1..=sheet_count {
        let part = format!("/xl/worksheets/sheet{i}.xml");
        w.empty_tag("Override", &[("PartName", &part), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml")]);
    }
    w.empty_tag("Override", &[("PartName", "/xl/styles.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml")]);
    w.empty_tag("Override", &[("PartName", "/xl/theme/theme1.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.theme+xml")]);
    w.empty_tag("Override", &[("PartName", "/xl/sharedStrings.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml")]);
    if has_props {
        w.empty_tag("Override", &[("PartName", "/docProps/core.xml"), ("ContentType", "application/vnd.openxmlformats-package.core-properties+xml")]);
        w.empty_tag("Override", &[("PartName", "/docProps/app.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.extended-properties+xml")]);
    }
    w.end_tag("Types");
    w.into_bytes()
}

fn write_streaming_root_rels(has_props: bool) -> Vec<u8> {
    let mut rels = vec![
        ("rId1", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", "xl/workbook.xml"),
    ];
    if has_props {
        rels.push(("rId2", "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", "docProps/core.xml"));
        rels.push(("rId3", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties", "docProps/app.xml"));
    }
    rel_writer::write_rels(&rels)
}

fn write_streaming_workbook(sheets: &[StreamingSheet]) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("workbook", &[
        ("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
    ]);
    w.start_tag("sheets", &[]);
    for (i, sheet) in sheets.iter().enumerate() {
        let id = (i + 1).to_string();
        let rid = format!("rId{}", i + 1);
        w.empty_tag("sheet", &[("name", &sheet.name), ("sheetId", &id), ("r:id", &rid)]);
    }
    w.end_tag("sheets");
    w.end_tag("workbook");
    w.into_bytes()
}
