use std::path::Path;

use crate::model::shared_strings::SharedStringTable;
use crate::model::style_registry::StyleRegistry;
use crate::properties::{self, DocProperties};
use crate::reader::xlsx_reader;
use crate::writer::sheet_writer::{self, SheetCells};
use crate::writer::{chart_writer, drawing_writer, rel_writer, sst_writer, style_writer, table_writer};
use crate::worksheet::Worksheet;
use crate::xml::xml_writer::XmlWriter;
use crate::zip::zip_writer::ZipOutput;

/// An Excel workbook. Supports three modes:
/// - `Workbook::new()` — Create a new workbook from scratch
/// - `Workbook::open(path)` — Open an existing file for editing (lazy deserialization)
/// - `Workbook::open_readonly(path)` — Open for reading only
pub struct Workbook {
    worksheets: Vec<Worksheet>,
    sst: SharedStringTable,
    styles: StyleRegistry,
    defined_names: Vec<(String, String)>,
    properties: DocProperties,
    /// Raw zip entries to pass through on save (edit mode).
    passthrough_entries: Vec<(String, Vec<u8>)>,
    workbook_protection: Option<WorkbookProtection>,
    is_xlsm: bool,
}

impl Workbook {
    /// Create a new empty workbook with one sheet ("Sheet1").
    pub fn new() -> Self {
        Self {
            worksheets: vec![Worksheet::new("Sheet1")],
            sst: SharedStringTable::new(),
            styles: StyleRegistry::new(),
            defined_names: Vec::new(),
            properties: DocProperties::default(),
            passthrough_entries: Vec::new(),
            workbook_protection: None,
            is_xlsm: false,
        }
    }

    /// Open an existing xlsx file for reading only.
    pub fn open_readonly(path: impl AsRef<Path>) -> crate::Result<Self> {
        let (data, mut zip) = xlsx_reader::read_xlsx(path.as_ref())?;
        let mut worksheets = Vec::with_capacity(data.sheets.len());
        for sheet_info in &data.sheets {
            let mut ws = Worksheet::new(&sheet_info.name);
            let cells = xlsx_reader::read_sheet_data(&mut zip, &sheet_info.path, &data.sst, &data.styles)?;
            ws.read_cells = Some(cells);
            worksheets.push(ws);
        }
        Ok(Self {
            worksheets, sst: data.sst, styles: StyleRegistry::new(),
            defined_names: data.defined_names,
            properties: data.properties,
            passthrough_entries: Vec::new(),
            workbook_protection: None,
            is_xlsm: false,
        })
    }

    /// Open an existing xlsx file for editing.
    /// Sheets are lazily deserialized — only parsed when first accessed.
    /// Unmodified sheets are written back as raw bytes.
    pub fn open(path: impl AsRef<Path>) -> crate::Result<Self> {
        let (data, mut zip) = xlsx_reader::read_xlsx(path.as_ref())?;
        let mut worksheets = Vec::with_capacity(data.sheets.len());

        for sheet_info in &data.sheets {
            let mut ws = Worksheet::new(&sheet_info.name);
            // Store raw XML for lazy deserialization
            if let Some(raw) = zip.read_entry(&sheet_info.path) {
                ws.raw_xml = Some(raw?);
            }
            worksheets.push(ws);
        }

        // Collect passthrough entries (media, charts, drawings, etc.)
        let known_prefixes = ["xl/worksheets/", "xl/workbook.xml", "xl/sharedStrings.xml",
            "xl/styles.xml", "[Content_Types].xml", "_rels/", "xl/_rels/workbook.xml.rels"];
        let mut passthrough = Vec::new();
        let entry_names: Vec<String> = (0..zip.archive.len())
            .filter_map(|i| zip.archive.by_index_raw(i).ok().map(|e| e.name().to_string()))
            .collect();
        for name in &entry_names {
            let dominated = known_prefixes.iter().any(|p| name.starts_with(p) || name == *p);
            if !dominated {
                if let Some(Ok(bytes)) = zip.read_entry(name) {
                    passthrough.push((name.clone(), bytes));
                }
            }
        }

        let is_xlsm = path.as_ref().extension().map_or(false, |e| e.eq_ignore_ascii_case("xlsm"));

        Ok(Self {
            worksheets,
            sst: data.sst,
            styles: StyleRegistry::new(),
            defined_names: data.defined_names,
            properties: data.properties,
            passthrough_entries: passthrough,
            workbook_protection: None,
            is_xlsm,
        })
    }

    /// Save the workbook to an xlsx file.
    pub fn save(&mut self, path: impl AsRef<Path>) -> crate::Result<()> {
        let bytes = self.save_to_buffer()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Save the workbook to an in-memory buffer.
    pub fn save_to_buffer(&mut self) -> crate::Result<Vec<u8>> {
        for ws in &mut self.worksheets {
            if ws.raw_xml.is_some() && ws.read_cells.is_none() && !ws.dirty { continue; }
            if ws.raw_xml.is_some() && ws.dirty { ws.raw_xml = None; }
            ws.finalize(&mut self.sst, &mut self.styles);
        }

        let mut zip = ZipOutput::new();
        let sheet_count = self.worksheets.len();
        let has_props = self.properties.title.is_some() || self.properties.author.is_some()
            || self.properties.subject.is_some() || self.properties.company.is_some();

        // Count charts, images, tables across all sheets for content types
        let mut total_charts = 0usize;
        let mut total_tables = 0usize;
        let mut sheets_with_drawings = Vec::new();
        let mut image_extensions: Vec<String> = Vec::new();

        for (i, ws) in self.worksheets.iter().enumerate() {
            let has_drawing = !ws.charts.is_empty() || !ws.images.is_empty();
            if has_drawing { sheets_with_drawings.push(i); }
            total_charts += ws.charts.len();
            total_tables += ws.tables.len();
            for img in &ws.images {
                image_extensions.push(img.image_type.extension().to_string());
            }
        }

        let has_vba = self.passthrough_entries.iter().any(|(n, _)| n.eq_ignore_ascii_case("xl/vbaProject.bin"));

        zip.add_file("[Content_Types].xml", &write_content_types_full(
            sheet_count, has_props, total_charts, total_tables, &sheets_with_drawings, &image_extensions, has_vba, self.is_xlsm,
        ))?;
        zip.add_file("_rels/.rels", &write_root_rels(has_props))?;
        zip.add_file("xl/_rels/workbook.xml.rels", &rel_writer::write_workbook_rels(sheet_count, has_vba))?;
        zip.add_file("xl/workbook.xml", &self.write_workbook_xml())?;

        // Pre-compute per-sheet metadata for parallel assembly
        struct SheetMeta {
            drawing_rid: Option<String>,
            table_rids: Vec<String>,
            sheet_rels: Vec<(String, String, String)>,
            global_chart_start: usize,
            global_image_start: usize,
            global_table_start: usize,
        }

        let mut metas = Vec::with_capacity(sheet_count);
        let mut global_chart_idx = 0usize;
        let mut global_image_idx = 0usize;
        let mut global_table_idx = 0usize;

        for (i, ws) in self.worksheets.iter().enumerate() {
            let has_drawing = !ws.charts.is_empty() || !ws.images.is_empty();
            let mut sheet_rels: Vec<(String, String, String)> = Vec::new();
            let mut next_rid = 1;

            let drawing_rid = if has_drawing {
                let rid = format!("rId{next_rid}");
                sheet_rels.push((rid.clone(), "http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing".into(), format!("../drawings/drawing{}.xml", i + 1)));
                next_rid += 1;
                Some(rid)
            } else { None };

            let mut table_rids = Vec::new();
            for t_idx in 0..ws.tables.len() {
                let rid = format!("rId{next_rid}");
                let tid = global_table_idx + t_idx + 1;
                sheet_rels.push((rid.clone(), "http://schemas.openxmlformats.org/officeDocument/2006/relationships/table".into(), format!("../tables/table{tid}.xml")));
                table_rids.push(rid);
                next_rid += 1;
            }

            metas.push(SheetMeta {
                drawing_rid, table_rids, sheet_rels,
                global_chart_start: global_chart_idx,
                global_image_start: global_image_idx,
                global_table_start: global_table_idx,
            });

            global_chart_idx += ws.charts.len();
            global_image_idx += ws.images.len();
            global_table_idx += ws.tables.len();
        }

        // Parallel sheet XML generation
        let sheet_xmls: Vec<Option<Vec<u8>>> = std::thread::scope(|s| {
            let handles: Vec<_> = self.worksheets.iter().zip(metas.iter()).map(|(ws, meta)| {
                s.spawn(move || {
                    if ws.raw_xml.is_some() && !ws.dirty {
                        return None;
                    }
                    let sc = SheetCells {
                        cells: &ws.cells, merge_ranges: &ws.merge_ranges,
                        col_widths: &ws.col_widths, row_heights: &ws.row_heights,
                        freeze_row: ws.freeze_row, freeze_col: ws.freeze_col,
                        drawing_rid: meta.drawing_rid.clone(), table_parts: meta.table_rids.clone(),
                        conditional_formats: &ws.conditional_formats,
                        validations: &ws.validations, sparklines: &ws.sparklines,
                        protection: ws.protection.as_ref(),
                        print_settings: ws.print_settings.as_ref(),
                    };
                    Some(sheet_writer::write_sheet(&sc))
                })
            }).collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        // Sequential zip write
        for (i, (ws, meta)) in self.worksheets.iter().zip(metas.iter()).enumerate() {
            let sheet_path = format!("xl/worksheets/sheet{}.xml", i + 1);

            match &sheet_xmls[i] {
                None => {
                    // Raw passthrough
                    if let Some(ref raw) = ws.raw_xml {
                        zip.add_file(&sheet_path, raw)?;
                    }
                }
                Some(xml) => {
                    zip.add_file(&sheet_path, xml)?;
                }
            }

            // Write sheet rels
            if !meta.sheet_rels.is_empty() {
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", i + 1);
                let refs: Vec<(&str, &str, &str)> = meta.sheet_rels.iter().map(|(a, b, c)| (a.as_str(), b.as_str(), c.as_str())).collect();
                zip.add_file(&rels_path, &rel_writer::write_rels(&refs))?;
            }

            let has_drawing = !ws.charts.is_empty() || !ws.images.is_empty();
            if has_drawing {
                let drawing_path = format!("xl/drawings/drawing{}.xml", i + 1);
                zip.add_file(&drawing_path, &drawing_writer::write_drawing_xml(&ws.charts, &ws.images, i))?;

                let img_types: Vec<&str> = ws.images.iter().map(|img| img.image_type.extension()).collect();
                let drawing_rels_path = format!("xl/drawings/_rels/drawing{}.xml.rels", i + 1);
                zip.add_file(&drawing_rels_path, &drawing_writer::write_drawing_rels(ws.charts.len(), ws.images.len(), &img_types))?;
            }

            for (ci, chart) in ws.charts.iter().enumerate() {
                let idx = meta.global_chart_start + ci + 1;
                let chart_path = format!("xl/charts/chart{idx}.xml");
                zip.add_file(&chart_path, &chart_writer::write_chart_xml(chart, idx))?;
            }

            for (ii, img) in ws.images.iter().enumerate() {
                let idx = meta.global_image_start + ii + 1;
                let media_path = format!("xl/media/image{}.{}", idx, img.image_type.extension());
                zip.add_file(&media_path, &img.data)?;
            }

            for (ti, table) in ws.tables.iter().enumerate() {
                let idx = meta.global_table_start + ti + 1;
                let table_path = format!("xl/tables/table{idx}.xml");
                zip.add_file(&table_path, &table_writer::write_table_xml(table, idx))?;
            }
        }

        zip.add_file("xl/styles.xml", &style_writer::write_styles(&self.styles))?;
        zip.add_file("xl/sharedStrings.xml", &sst_writer::write_sst(&self.sst))?;

        if has_props {
            zip.add_file("docProps/core.xml", &properties::write_core_xml(&self.properties))?;
            zip.add_file("docProps/app.xml", &properties::write_app_xml(&self.properties))?;
        }

        for (name, data) in &self.passthrough_entries {
            zip.add_file(name, data)?;
        }

        zip.finish()
    }

    // ── Sheet management ──

    pub fn add_worksheet(&mut self) -> &mut Worksheet {
        let name = format!("Sheet{}", self.worksheets.len() + 1);
        self.worksheets.push(Worksheet::new(&name));
        self.worksheets.last_mut().unwrap()
    }

    pub fn add_worksheet_with_name(&mut self, name: &str) -> crate::Result<&mut Worksheet> {
        if self.worksheets.iter().any(|ws| ws.name == name) {
            return Err(crate::Error::InvalidData(format!("Sheet '{name}' already exists")));
        }
        self.worksheets.push(Worksheet::new(name));
        Ok(self.worksheets.last_mut().unwrap())
    }

    pub fn worksheet(&mut self, index: usize) -> crate::Result<&mut Worksheet> {
        let ws = self.worksheets.get_mut(index)
            .ok_or_else(|| crate::Error::SheetNotFound(format!("index {index}")))?;
        // Lazy deserialization for edit mode
        if ws.raw_xml.is_some() && ws.read_cells.is_none() {
            let raw = ws.raw_xml.as_ref().unwrap();
            let cells = crate::reader::sheet_reader::read_sheet_cells(
                raw, &self.sst, &crate::reader::style_parser::ParsedStyles::default(),
            )?;
            ws.read_cells = Some(cells);
        }
        Ok(ws)
    }

    pub fn worksheet_by_name(&mut self, name: &str) -> crate::Result<&mut Worksheet> {
        let idx = self.worksheets.iter().position(|ws| ws.name == name)
            .ok_or_else(|| crate::Error::SheetNotFound(name.to_string()))?;
        self.worksheet(idx)
    }

    /// Remove a worksheet by index.
    pub fn remove_worksheet(&mut self, index: usize) -> crate::Result<()> {
        if index >= self.worksheets.len() {
            return Err(crate::Error::SheetNotFound(format!("index {index}")));
        }
        if self.worksheets.len() == 1 {
            return Err(crate::Error::InvalidData("Cannot remove the last worksheet".into()));
        }
        self.worksheets.remove(index);
        Ok(())
    }

    /// Rename a worksheet by index.
    pub fn rename_worksheet(&mut self, index: usize, name: &str) -> crate::Result<()> {
        if self.worksheets.iter().any(|ws| ws.name == name) {
            return Err(crate::Error::InvalidData(format!("Sheet '{name}' already exists")));
        }
        let ws = self.worksheets.get_mut(index)
            .ok_or_else(|| crate::Error::SheetNotFound(format!("index {index}")))?;
        ws.name = name.to_string();
        Ok(())
    }

    /// Move a worksheet from one index to another.
    pub fn move_worksheet(&mut self, from: usize, to: usize) -> crate::Result<()> {
        if from >= self.worksheets.len() || to >= self.worksheets.len() {
            return Err(crate::Error::SheetNotFound("index out of range".into()));
        }
        let ws = self.worksheets.remove(from);
        self.worksheets.insert(to, ws);
        Ok(())
    }

    pub fn sheet_names(&self) -> Vec<&str> {
        self.worksheets.iter().map(|ws| ws.name.as_str()).collect()
    }

    pub fn sheet_count(&self) -> usize { self.worksheets.len() }

    // ── Defined names ──

    pub fn define_name(&mut self, name: &str, formula: &str) -> &mut Self {
        self.defined_names.push((name.to_string(), formula.to_string()));
        self
    }

    pub fn defined_names(&self) -> &[(String, String)] { &self.defined_names }

    // ── Properties ──

    pub fn set_properties(&mut self, props: DocProperties) -> &mut Self {
        self.properties = props;
        self
    }

    pub fn properties(&self) -> &DocProperties { &self.properties }

    /// Protect the workbook structure (prevent adding/removing/renaming sheets).
    pub fn protect(&mut self) -> &mut Self {
        self.workbook_protection = Some(WorkbookProtection { password_hash: None });
        self
    }

    /// Protect the workbook structure with a password.
    pub fn protect_with_password(&mut self, password: &str) -> &mut Self {
        self.workbook_protection = Some(WorkbookProtection {
            password_hash: Some(crate::worksheet::hash_password_public(password)),
        });
        self
    }

    // ── Internal ──

    fn write_workbook_xml(&self) -> Vec<u8> {
        let mut w = XmlWriter::new();
        w.declaration();
        w.start_tag("workbook", &[
            ("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main"),
            ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
        ]);
        w.start_tag("sheets", &[]);
        for (i, ws) in self.worksheets.iter().enumerate() {
            let id = (i + 1).to_string();
            let rid = format!("rId{}", i + 1);
            w.empty_tag("sheet", &[("name", &ws.name), ("sheetId", &id), ("r:id", &rid)]);
        }
        w.end_tag("sheets");

        // workbookProtection
        if let Some(ref prot) = self.workbook_protection {
            let mut attrs: Vec<(&str, &str)> = vec![("lockStructure", "1")];
            let pw;
            if let Some(ref hash) = prot.password_hash {
                pw = hash.clone();
                attrs.push(("workbookPassword", &pw));
            }
            w.empty_tag("workbookProtection", &attrs);
        }

        if !self.defined_names.is_empty() {
            w.start_tag("definedNames", &[]);
            for (name, formula) in &self.defined_names {
                w.text_element("definedName", &[("name", name.as_str())], formula);
            }
            w.end_tag("definedNames");
        }

        w.end_tag("workbook");
        w.into_bytes()
    }
}

impl Default for Workbook {
    fn default() -> Self { Self::new() }
}

fn write_content_types_full(sheet_count: usize, has_props: bool, chart_count: usize, table_count: usize, sheets_with_drawings: &[usize], image_extensions: &[String], has_vba: bool, is_xlsm: bool) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("Types", &[("xmlns", "http://schemas.openxmlformats.org/package/2006/content-types")]);
    w.empty_tag("Default", &[("Extension", "rels"), ("ContentType", "application/vnd.openxmlformats-package.relationships+xml")]);
    w.empty_tag("Default", &[("Extension", "xml"), ("ContentType", "application/xml")]);
    if has_vba {
        w.empty_tag("Default", &[("Extension", "bin"), ("ContentType", "application/vnd.ms-office.vbaProject")]);
    }

    // Image defaults
    let mut seen_ext = std::collections::HashSet::new();
    for ext in image_extensions {
        if seen_ext.insert(ext.as_str()) {
            let ct = if ext == "png" { "image/png" } else { "image/jpeg" };
            w.empty_tag("Default", &[("Extension", ext), ("ContentType", ct)]);
        }
    }

    let wb_ct = if is_xlsm || has_vba {
        "application/vnd.ms-excel.sheet.macroEnabled.main+xml"
    } else {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"
    };
    w.empty_tag("Override", &[("PartName", "/xl/workbook.xml"), ("ContentType", wb_ct)]);
    for i in 1..=sheet_count {
        let part = format!("/xl/worksheets/sheet{i}.xml");
        w.empty_tag("Override", &[("PartName", &part), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml")]);
    }
    w.empty_tag("Override", &[("PartName", "/xl/styles.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml")]);
    w.empty_tag("Override", &[("PartName", "/xl/sharedStrings.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml")]);

    for &si in sheets_with_drawings {
        let part = format!("/xl/drawings/drawing{}.xml", si + 1);
        w.empty_tag("Override", &[("PartName", &part), ("ContentType", "application/vnd.openxmlformats-officedocument.drawing+xml")]);
    }
    for i in 1..=chart_count {
        let part = format!("/xl/charts/chart{i}.xml");
        w.empty_tag("Override", &[("PartName", &part), ("ContentType", "application/vnd.openxmlformats-officedocument.drawingml.chart+xml")]);
    }
    for i in 1..=table_count {
        let part = format!("/xl/tables/table{i}.xml");
        w.empty_tag("Override", &[("PartName", &part), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.table+xml")]);
    }

    if has_props {
        w.empty_tag("Override", &[("PartName", "/docProps/core.xml"), ("ContentType", "application/vnd.openxmlformats-package.core-properties+xml")]);
        w.empty_tag("Override", &[("PartName", "/docProps/app.xml"), ("ContentType", "application/vnd.openxmlformats-officedocument.extended-properties+xml")]);
    }
    w.end_tag("Types");
    w.into_bytes()
}

fn write_root_rels(has_props: bool) -> Vec<u8> {
    let mut rels = vec![
        ("rId1", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", "xl/workbook.xml"),
    ];
    if has_props {
        rels.push(("rId2", "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", "docProps/core.xml"));
        rels.push(("rId3", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties", "docProps/app.xml"));
    }
    rel_writer::write_rels(&rels)
}

#[derive(Debug, Clone)]
pub struct WorkbookProtection {
    pub(crate) password_hash: Option<String>,
}
