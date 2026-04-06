//! Workbook module — split into logical submodules.

mod save;
mod xml;

use std::path::Path;

use crate::model::shared_strings::SharedStringTable;
use crate::model::style_registry::StyleRegistry;
use crate::properties::DocProperties;
use crate::reader::xlsx_reader;
use crate::worksheet::Worksheet;

pub(crate) use xml::{write_content_types_full, write_root_rels};

/// An Excel workbook.
pub struct Workbook {
    pub(crate) worksheets: Vec<Worksheet>,
    pub(crate) sst: SharedStringTable,
    pub(crate) styles: StyleRegistry,
    pub(crate) defined_names: Vec<(String, String)>,
    pub(crate) properties: DocProperties,
    pub(crate) passthrough_entries: Vec<(String, Vec<u8>)>,
    pub(crate) workbook_protection: Option<WorkbookProtection>,
    pub(crate) is_xlsm: bool,
    pub(crate) calc_mode: Option<CalcMode>,
    pub(crate) active_sheet: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum CalcMode { Auto, Manual, AutoNoTable }

#[derive(Debug, Clone)]
pub struct WorkbookProtection {
    pub(crate) password_hash: Option<String>,
}

impl Workbook {
    pub fn new() -> Self {
        Self {
            worksheets: vec![Worksheet::new("Sheet1")],
            sst: SharedStringTable::new(), styles: StyleRegistry::new(),
            defined_names: Vec::new(), properties: DocProperties::default(),
            passthrough_entries: Vec::new(), workbook_protection: None,
            is_xlsm: false, calc_mode: None, active_sheet: None,
        }
    }

    pub fn open_readonly(path: impl AsRef<Path>) -> crate::Result<Self> {
        let (data, mut zip) = xlsx_reader::read_xlsx(path.as_ref())?;
        Self::from_xlsx_data(data, &mut zip, false)
    }

    pub fn open_readonly_from_buffer(bytes: &[u8]) -> crate::Result<Self> {
        let cursor = std::io::Cursor::new(bytes);
        let mut zip = crate::zip::zip_reader::ZipReader::new(cursor)?;
        let data = xlsx_reader::read_xlsx_from_zip(&mut zip)?;
        Self::from_xlsx_data(data, &mut zip, false)
    }

    pub fn open_from_buffer(bytes: &[u8]) -> crate::Result<Self> {
        let cursor = std::io::Cursor::new(bytes);
        let mut zip = crate::zip::zip_reader::ZipReader::new(cursor)?;
        let data = xlsx_reader::read_xlsx_from_zip(&mut zip)?;
        Self::from_xlsx_data_edit(data, &mut zip)
    }

    pub fn open(path: impl AsRef<Path>) -> crate::Result<Self> {
        let (data, mut zip) = xlsx_reader::read_xlsx(path.as_ref())?;
        let is_xlsm = path.as_ref().extension().map_or(false, |e| e.eq_ignore_ascii_case("xlsm"));
        let mut wb = Self::from_xlsx_data_edit(data, &mut zip)?;
        wb.is_xlsm = is_xlsm;
        Ok(wb)
    }

    pub fn save(&mut self, path: impl AsRef<Path>) -> crate::Result<()> {
        let bytes = self.save_to_buffer()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    // ── Open helpers ──

    fn from_xlsx_data<R: std::io::Read + std::io::Seek>(data: xlsx_reader::XlsxData, zip: &mut crate::zip::zip_reader::ZipReader<R>, _edit: bool) -> crate::Result<Self> {
        let mut worksheets = Vec::with_capacity(data.sheets.len());
        for sheet_info in &data.sheets {
            let mut ws = Worksheet::new(&sheet_info.name);
            let raw = zip.read_entry(&sheet_info.path)
                .ok_or_else(|| crate::Error::SheetNotFound(sheet_info.path.clone()))??;
            let (cells, meta) = crate::reader::sheet_reader::read_sheet_full(&raw, &data.sst, &data.styles)?;
            let map = cells.iter().map(|rc| ((rc.row, rc.col), rc.value.clone())).collect();
            ws.read_cells_map = Some(map);
            ws.read_cells = Some(cells);
            ws.merge_ranges = meta.merge_ranges;
            for (c, w) in meta.col_widths { ws.col_widths.insert(c, w); }
            for (r, h) in meta.row_heights { ws.row_heights.insert(r, h); }
            ws.freeze_row = meta.freeze_row;
            ws.freeze_col = meta.freeze_col;
            // Read comments
            let comments_path = format!("xl/comments{}.xml", worksheets.len() + 1);
            if let Some(Ok(comments_data)) = zip.read_entry(&comments_path) {
                ws.comments = parse_comments(&comments_data);
            }
            ws.visibility = match sheet_info.visibility {
                1 => crate::worksheet::SheetVisibility::Hidden,
                2 => crate::worksheet::SheetVisibility::VeryHidden,
                _ => crate::worksheet::SheetVisibility::Visible,
            };
            worksheets.push(ws);
        }
        Ok(Self {
            worksheets, sst: data.sst, styles: StyleRegistry::new(),
            defined_names: data.defined_names, properties: data.properties,
            passthrough_entries: Vec::new(), workbook_protection: None,
            is_xlsm: false, calc_mode: None, active_sheet: None,
        })
    }

    fn from_xlsx_data_edit<R: std::io::Read + std::io::Seek>(data: xlsx_reader::XlsxData, zip: &mut crate::zip::zip_reader::ZipReader<R>) -> crate::Result<Self> {
        let mut worksheets = Vec::with_capacity(data.sheets.len());
        for (i, sheet_info) in data.sheets.iter().enumerate() {
            let mut ws = Worksheet::new(&sheet_info.name);
            if let Some(raw) = zip.read_entry(&sheet_info.path) { ws.raw_xml = Some(raw?); }
            let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", i + 1);
            if let Some(Ok(rels_data)) = zip.read_entry(&rels_path) { ws.original_rels = Some(rels_data); }
            ws.visibility = match sheet_info.visibility {
                1 => crate::worksheet::SheetVisibility::Hidden,
                2 => crate::worksheet::SheetVisibility::VeryHidden,
                _ => crate::worksheet::SheetVisibility::Visible,
            };
            worksheets.push(ws);
        }
        let known_prefixes = ["xl/worksheets/", "xl/workbook.xml", "xl/sharedStrings.xml",
            "xl/styles.xml", "xl/theme/", "[Content_Types].xml", "_rels/", "xl/_rels/workbook.xml.rels", "docProps/"];
        let mut passthrough = Vec::new();
        let entry_names: Vec<String> = (0..zip.archive.len())
            .filter_map(|i| zip.archive.by_index_raw(i).ok().map(|e| e.name().to_string())).collect();
        for name in &entry_names {
            if !known_prefixes.iter().any(|p| name.starts_with(p) || name == *p) {
                if let Some(Ok(bytes)) = zip.read_entry(name) { passthrough.push((name.clone(), bytes)); }
            }
        }
        Ok(Self {
            worksheets, sst: data.sst, styles: StyleRegistry::new(),
            defined_names: data.defined_names, properties: data.properties,
            passthrough_entries: passthrough, workbook_protection: None,
            is_xlsm: false, calc_mode: None, active_sheet: None,
        })
    }

    // ── Sheet management ──

    pub fn add_worksheet(&mut self) -> &mut Worksheet {
        let name = format!("Sheet{}", self.worksheets.len() + 1);
        self.worksheets.push(Worksheet::new(&name));
        self.worksheets.last_mut().unwrap()
    }

    pub fn add_worksheet_with_name(&mut self, name: &str) -> crate::Result<&mut Worksheet> {
        crate::worksheet::validate_sheet_name(name)?;
        if self.worksheets.iter().any(|ws| ws.name == name) {
            return Err(crate::Error::InvalidData(format!("Sheet '{name}' already exists")));
        }
        self.worksheets.push(Worksheet::new(name));
        Ok(self.worksheets.last_mut().unwrap())
    }

    pub fn worksheet(&mut self, index: usize) -> crate::Result<&mut Worksheet> {
        let ws = self.worksheets.get_mut(index)
            .ok_or_else(|| crate::Error::SheetNotFound(format!("index {index}")))?;
        if ws.raw_xml.is_some() && ws.read_cells.is_none() {
            let raw = ws.raw_xml.as_ref().unwrap();
            let (cells, meta) = crate::reader::sheet_reader::read_sheet_full(
                raw, &self.sst, &crate::reader::style_parser::ParsedStyles::default(),
            )?;
            let map = cells.iter().map(|rc| ((rc.row, rc.col), rc.value.clone())).collect();
            ws.read_cells_map = Some(map);
            ws.read_cells = Some(cells);
            ws.merge_ranges = meta.merge_ranges;
            for (c, w) in meta.col_widths { ws.col_widths.insert(c, w); }
            for (r, h) in meta.row_heights { ws.row_heights.insert(r, h); }
            ws.freeze_row = meta.freeze_row;
            ws.freeze_col = meta.freeze_col;
            ws.original_drawing_rid = meta.drawing_rid;
            ws.original_legacy_drawing_rid = meta.legacy_drawing_rid;
        }
        Ok(ws)
    }

    pub fn worksheet_by_name(&mut self, name: &str) -> crate::Result<&mut Worksheet> {
        let idx = self.worksheets.iter().position(|ws| ws.name == name)
            .ok_or_else(|| crate::Error::SheetNotFound(name.to_string()))?;
        self.worksheet(idx)
    }

    pub fn remove_worksheet(&mut self, index: usize) -> crate::Result<()> {
        if index >= self.worksheets.len() { return Err(crate::Error::SheetNotFound(format!("index {index}"))); }
        if self.worksheets.len() == 1 { return Err(crate::Error::InvalidData("Cannot remove the last worksheet".into())); }
        self.worksheets.remove(index);
        Ok(())
    }

    pub fn rename_worksheet(&mut self, index: usize, name: &str) -> crate::Result<()> {
        crate::worksheet::validate_sheet_name(name)?;
        if self.worksheets.iter().any(|ws| ws.name == name) {
            return Err(crate::Error::InvalidData(format!("Sheet '{name}' already exists")));
        }
        self.worksheets.get_mut(index).ok_or_else(|| crate::Error::SheetNotFound(format!("index {index}")))?.name = name.to_string();
        Ok(())
    }

    pub fn move_worksheet(&mut self, from: usize, to: usize) -> crate::Result<()> {
        if from >= self.worksheets.len() || to >= self.worksheets.len() {
            return Err(crate::Error::SheetNotFound("index out of range".into()));
        }
        let ws = self.worksheets.remove(from);
        self.worksheets.insert(to, ws);
        Ok(())
    }

    pub fn sheet_names(&self) -> Vec<&str> { self.worksheets.iter().map(|ws| ws.name.as_str()).collect() }
    pub fn sheet_count(&self) -> usize { self.worksheets.len() }
    pub fn worksheet_ref(&self, index: usize) -> crate::Result<&Worksheet> {
        self.worksheets.get(index).ok_or(crate::Error::SheetNotFound(format!("index {index}")))
    }

    // ── Properties & settings ──

    pub fn define_name(&mut self, name: &str, formula: &str) -> &mut Self {
        self.defined_names.push((name.to_string(), formula.to_string())); self
    }
    pub fn defined_names(&self) -> &[(String, String)] { &self.defined_names }
    pub fn set_properties(&mut self, props: DocProperties) -> &mut Self { self.properties = props; self }
    pub fn properties(&self) -> &DocProperties { &self.properties }

    pub fn protect(&mut self) -> &mut Self {
        self.workbook_protection = Some(WorkbookProtection { password_hash: None }); self
    }
    pub fn protect_with_password(&mut self, password: &str) -> &mut Self {
        self.workbook_protection = Some(WorkbookProtection {
            password_hash: Some(crate::worksheet::hash_password_public(password)),
        }); self
    }

    pub fn set_calc_mode(&mut self, mode: CalcMode) -> &mut Self { self.calc_mode = Some(mode); self }
    pub fn set_active_sheet(&mut self, index: usize) -> &mut Self { self.active_sheet = Some(index); self }

    pub fn pictures<R: std::io::Read + std::io::Seek>(zip: &mut crate::zip::zip_reader::ZipReader<R>) -> Vec<(String, Vec<u8>)> {
        let mut pics = Vec::new();
        let names: Vec<String> = (0..zip.archive.len())
            .filter_map(|i| zip.archive.by_index_raw(i).ok().map(|e| e.name().to_string())).collect();
        for name in names {
            if name.starts_with("xl/media/") {
                if let Some(Ok(data)) = zip.read_entry(&name) {
                    pics.push((name.rsplit('/').next().unwrap_or(&name).to_string(), data));
                }
            }
        }
        pics
    }
}

impl Default for Workbook {
    fn default() -> Self { Self::new() }
}

/// Parse comments from xl/comments{N}.xml.
fn parse_comments(data: &[u8]) -> Vec<crate::worksheet::Comment> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;
    use crate::xml::xml_reader::get_attr;

    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::new();
    let mut authors: Vec<String> = Vec::new();
    let mut comments = Vec::new();
    let mut in_author = false;
    let mut in_comment = false;
    let mut in_text = false;
    let mut cur_row = 0u32;
    let mut cur_col = 0u16;
    let mut cur_author_id = 0usize;
    let mut cur_text = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"author" => { in_author = true; }
                b"comment" => {
                    in_comment = true;
                    if let Some(r) = get_attr(e.attributes(), b"ref").and_then(|v| std::str::from_utf8(v).ok()) {
                        if let Ok((r, c)) = crate::utility::parse_cell_ref(r) { cur_row = r; cur_col = c; }
                    }
                    cur_author_id = get_attr(e.attributes(), b"authorId")
                        .and_then(|v| atoi_simd::parse::<usize>(v).ok()).unwrap_or(0);
                    cur_text.clear();
                }
                b"t" if in_comment => { in_text = true; }
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if in_author { if let Ok(t) = e.unescape() { authors.push(t.to_string()); } in_author = false; }
                if in_text { if let Ok(t) = e.unescape() { cur_text.push_str(&t); } }
            },
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"comment" => {
                    let author = authors.get(cur_author_id).cloned().unwrap_or_default();
                    comments.push(crate::worksheet::Comment { row: cur_row, col: cur_col, text: cur_text.clone(), author });
                    in_comment = false;
                }
                b"t" => { in_text = false; }
                b"author" => { in_author = false; }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    comments
}
