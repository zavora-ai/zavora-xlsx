//! Workbook module — split into logical submodules.

mod save;
mod xml;

use std::path::Path;
use std::sync::Arc;

use crate::model::shared_strings::SharedStringTable;
use crate::model::style_registry::StyleRegistry;
use crate::properties::DocProperties;
use crate::reader::cf_reader;
use crate::reader::chart_reader;
use crate::reader::comment_reader;
use crate::reader::slicer_reader;
use crate::reader::sparkline_reader;
use crate::reader::style_parser::ParsedStyles;
use crate::reader::table_reader;
use crate::reader::validation_reader;
use crate::reader::xlsx_reader;
use crate::worksheet::Worksheet;

pub(crate) use xml::{write_content_types_full, write_root_rels};

/// The scope of a defined name: either workbook-global or local to a specific sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinedNameScope {
    /// The name is visible across the entire workbook.
    Workbook,
    /// The name is scoped to a specific sheet (0-based index).
    Sheet(usize),
}

/// A defined name in an Excel workbook, with its scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinedName {
    /// The name identifier (e.g. "TaxRate", "_xlnm.Print_Area").
    pub name: String,
    /// The formula or reference the name resolves to (e.g. "Sheet1!$A$1:$B$10").
    pub formula: String,
    /// Whether this name is workbook-scoped or sheet-scoped.
    pub scope: DefinedNameScope,
}

/// An Excel workbook.
pub struct Workbook {
    pub(crate) worksheets: Vec<Worksheet>,
    pub(crate) chart_sheets: Vec<ChartSheet>,
    pub(crate) sst: SharedStringTable,
    pub(crate) styles: StyleRegistry,
    pub(crate) defined_names: Vec<(String, String)>,
    pub(crate) scoped_defined_names: Vec<DefinedName>,
    pub(crate) properties: DocProperties,
    pub(crate) passthrough_entries: Vec<(String, Vec<u8>)>,
    pub(crate) workbook_protection: Option<WorkbookProtection>,
    pub(crate) is_xlsm: bool,
    pub(crate) calc_mode: Option<CalcMode>,
    pub(crate) active_sheet: Option<usize>,
    pub(crate) parsed_styles: Option<Arc<ParsedStyles>>,
    pub(crate) sst_threshold: Option<usize>,
    pub(crate) custom_properties: Vec<crate::properties::CustomProperty>,
    pub(crate) custom_xml_parts: Vec<(String, Vec<u8>)>,
}

/// A chart sheet — a dedicated sheet that contains only a chart (no cells).
#[derive(Debug, Clone)]
pub struct ChartSheet {
    pub(crate) name: String,
    pub(crate) chart: crate::features::chart::Chart,
}

#[derive(Debug, Clone, Copy)]
pub enum CalcMode {
    Auto,
    Manual,
    AutoNoTable,
}

/// The content type variant used when saving a workbook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkbookContentType {
    /// Standard `.xlsx` workbook
    Xlsx,
    /// Macro-enabled `.xlsm` workbook
    Xlsm,
    /// Template `.xltx`
    Template,
    /// Macro-enabled template `.xltm`
    TemplateMacro,
}

#[derive(Debug, Clone)]
pub struct WorkbookProtection {
    pub(crate) password_hash: Option<String>,
}

impl Workbook {
    pub fn new() -> Self {
        Self {
            worksheets: vec![Worksheet::new("Sheet1")],
            chart_sheets: Vec::new(),
            sst: SharedStringTable::new(),
            styles: StyleRegistry::new(),
            defined_names: Vec::new(),
            scoped_defined_names: Vec::new(),
            properties: DocProperties::default(),
            passthrough_entries: Vec::new(),
            workbook_protection: None,
            is_xlsm: false,
            calc_mode: None,
            active_sheet: None,
            parsed_styles: None,
            sst_threshold: None,
            custom_properties: Vec::new(),
            custom_xml_parts: Vec::new(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    /// Open a password-protected (encrypted) xlsx file.
    ///
    /// Encrypted Excel files use OLE2 compound document packaging with
    /// ECMA-376 Standard or Agile encryption. This method detects the OLE2
    /// magic bytes and attempts decryption.
    ///
    /// Currently returns `UnsupportedFormat` — full implementation requires
    /// the `crypto` feature flag with AES/SHA dependencies.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_with_password(path: impl AsRef<Path>, password: &str) -> crate::Result<Self> {
        let data = std::fs::read(path.as_ref())?;
        if crate::crypto::is_ole2(&data) {
            let decrypted = crate::crypto::decrypt(&data, password)?;
            Self::open_from_buffer(&decrypted)
        } else {
            // Not encrypted — try opening normally
            Self::open(path)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(path: impl AsRef<Path>) -> crate::Result<Self> {
        let (data, mut zip) = xlsx_reader::read_xlsx(path.as_ref())?;
        let is_xlsm = path
            .as_ref()
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("xlsm"));
        let mut wb = Self::from_xlsx_data_edit(data, &mut zip)?;
        wb.is_xlsm = is_xlsm;
        Ok(wb)
    }

    /// Open a workbook using memory-mapped I/O for the initial file read.
    ///
    /// This can be faster for very large files because the OS handles paging.
    /// Requires the `mmap` feature flag.
    #[cfg(feature = "mmap")]
    pub fn open_mmap(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let cursor = std::io::Cursor::new(&mmap[..]);
        let mut zip = crate::zip::zip_reader::ZipReader::new(cursor)?;
        let data = xlsx_reader::read_xlsx_from_zip(&mut zip)?;
        Self::from_xlsx_data(data, &mut zip, false)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&mut self, path: impl AsRef<Path>) -> crate::Result<()> {
        let bytes = self.save_to_buffer()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Save the workbook as a password-protected (encrypted) xlsx file.
    ///
    /// Encrypted Excel files use OLE2 compound document packaging with
    /// ECMA-376 Agile encryption (AES-256-CBC, SHA-512, HMAC).
    ///
    /// Currently returns `UnsupportedFormat` — full implementation requires
    /// the `crypto` feature flag with AES/SHA dependencies.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_encrypted(
        &mut self,
        _path: impl AsRef<Path>,
        _password: &str,
    ) -> crate::Result<()> {
        Err(crate::Error::UnsupportedFormat(
            "File encryption is not yet implemented. \
             Enable the 'crypto' feature flag when available."
                .to_string(),
        ))
    }

    /// Save the workbook as a macro-enabled `.xlsm` file.
    ///
    /// The `vba_project` parameter must contain the raw bytes of a `vbaProject.bin`
    /// file (e.g. extracted from an existing `.xlsm`). The method sets the
    /// appropriate macro-enabled content types and includes the VBA binary in
    /// the output ZIP.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_as_xlsm(
        &mut self,
        path: impl AsRef<Path>,
        vba_project: &[u8],
    ) -> crate::Result<()> {
        if vba_project.is_empty() {
            return Err(crate::Error::InvalidData(
                "VBA project data is empty".into(),
            ));
        }
        // Temporarily set XLSM flags
        let prev_xlsm = self.is_xlsm;
        self.is_xlsm = true;
        // Add vbaProject.bin as a passthrough entry
        self.passthrough_entries
            .retain(|(n, _)| !n.eq_ignore_ascii_case("xl/vbaProject.bin"));
        self.passthrough_entries
            .push(("xl/vbaProject.bin".to_string(), vba_project.to_vec()));
        let result = self.save(path);
        // Restore previous state
        self.is_xlsm = prev_xlsm;
        if !prev_xlsm {
            self.passthrough_entries
                .retain(|(n, _)| !n.eq_ignore_ascii_case("xl/vbaProject.bin"));
        }
        result
    }

    /// Save the workbook as an Excel template (`.xltx`).
    ///
    /// This sets the content type to `spreadsheetml.template.main+xml`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_as_template(&mut self, path: impl AsRef<Path>) -> crate::Result<()> {
        let bytes = self.save_to_buffer_with_content_type(WorkbookContentType::Template)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Save the workbook as a macro-enabled Excel template (`.xltm`).
    ///
    /// The `vba_project` parameter must contain the raw bytes of a `vbaProject.bin`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_as_template_macro(
        &mut self,
        path: impl AsRef<Path>,
        vba_project: &[u8],
    ) -> crate::Result<()> {
        if vba_project.is_empty() {
            return Err(crate::Error::InvalidData(
                "VBA project data is empty".into(),
            ));
        }
        self.passthrough_entries
            .retain(|(n, _)| !n.eq_ignore_ascii_case("xl/vbaProject.bin"));
        self.passthrough_entries
            .push(("xl/vbaProject.bin".to_string(), vba_project.to_vec()));
        let bytes = self.save_to_buffer_with_content_type(WorkbookContentType::TemplateMacro)?;
        std::fs::write(path, bytes)?;
        self.passthrough_entries
            .retain(|(n, _)| !n.eq_ignore_ascii_case("xl/vbaProject.bin"));
        Ok(())
    }

    // ── Open helpers ──

    fn from_xlsx_data<R: std::io::Read + std::io::Seek>(
        data: xlsx_reader::XlsxData,
        zip: &mut crate::zip::zip_reader::ZipReader<R>,
        _edit: bool,
    ) -> crate::Result<Self> {
        let parsed_styles = Arc::new(data.styles);
        let mut worksheets = Vec::with_capacity(data.sheets.len());
        for sheet_info in &data.sheets {
            let mut ws = Worksheet::new(&sheet_info.name);
            let raw = zip
                .read_entry(&sheet_info.path)
                .ok_or_else(|| crate::Error::SheetNotFound(sheet_info.path.clone()))??;
            let (cells, meta) =
                crate::reader::sheet_reader::read_sheet_full(&raw, &data.sst, &parsed_styles)?;
            let map = cells
                .iter()
                .map(|rc| ((rc.row, rc.col), rc.value.clone()))
                .collect();
            ws.read_cells_map = Some(map);
            ws.read_cells = Some(cells);
            ws.merge_ranges = meta.merge_ranges;
            for (c, w) in meta.col_widths {
                ws.col_widths.insert(c, w);
            }
            for (r, h) in meta.row_heights {
                ws.row_heights.insert(r, h);
            }
            ws.freeze_row = meta.freeze_row;
            ws.freeze_col = meta.freeze_col;
            ws.parsed_styles = Some(Arc::clone(&parsed_styles));
            // Populate print settings from sheet XML
            ws.print_settings = meta.print_settings;
            // Populate sheet protection from sheet XML
            ws.protection = meta.protection;
            // Populate row/column outline levels from sheet XML
            ws.row_outline_levels = meta.row_outline_levels;
            ws.col_outline_levels = meta.col_outline_levels;
            // Parse conditional formatting rules from the sheet XML
            ws.conditional_formats =
                cf_reader::parse_conditional_formats(&raw, &parsed_styles.dxf_records);
            // Parse data validation rules from the sheet XML
            ws.validations = validation_reader::parse_data_validations(&raw);
            // Parse sparklines from the extLst section of the sheet XML
            ws.sparklines = sparkline_reader::parse_sparklines(&raw);
            // Read comments via sheet relationships
            {
                let sheet_idx = worksheets.len() + 1;
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", sheet_idx);
                if let Some(Ok(sheet_rels_data)) = zip.read_entry(&rels_path) {
                    read_comments_for_worksheet(&mut ws, zip, &sheet_rels_data);
                }
            }
            // Read charts from drawing relationships
            if let Some(ref drawing_rid) = meta.drawing_rid {
                let sheet_idx = worksheets.len() + 1;
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", sheet_idx);
                if let Some(Ok(sheet_rels_data)) = zip.read_entry(&rels_path) {
                    read_charts_for_worksheet(&mut ws, zip, drawing_rid, &sheet_rels_data);
                }
            }
            // Read tables from sheet relationships
            {
                let sheet_idx = worksheets.len() + 1;
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", sheet_idx);
                if let Some(Ok(sheet_rels_data)) = zip.read_entry(&rels_path) {
                    read_tables_for_worksheet(&mut ws, zip, &sheet_rels_data);
                }
            }
            // Read slicers from sheet relationships
            {
                let sheet_idx = worksheets.len() + 1;
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", sheet_idx);
                if let Some(Ok(sheet_rels_data)) = zip.read_entry(&rels_path) {
                    read_slicers_for_worksheet(&mut ws, zip, &sheet_rels_data);
                }
            }
            // Read timelines from sheet relationships
            {
                let sheet_idx = worksheets.len() + 1;
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", sheet_idx);
                if let Some(Ok(sheet_rels_data)) = zip.read_entry(&rels_path) {
                    read_timelines_for_worksheet(&mut ws, zip, &sheet_rels_data);
                }
            }
            // Resolve hyperlinks from parsed sheet metadata + sheet rels
            {
                let sheet_idx = worksheets.len() + 1;
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", sheet_idx);
                let sheet_rels_data = zip.read_entry(&rels_path).and_then(|r| r.ok());
                resolve_hyperlinks_for_worksheet(
                    &mut ws,
                    &meta.hyperlinks,
                    sheet_rels_data.as_deref(),
                );
            }
            ws.visibility = match sheet_info.visibility {
                1 => crate::worksheet::SheetVisibility::Hidden,
                2 => crate::worksheet::SheetVisibility::VeryHidden,
                _ => crate::worksheet::SheetVisibility::Visible,
            };
            worksheets.push(ws);
        }
        // Parse _xlnm.Print_Titles defined names to populate repeat rows/columns
        apply_print_titles(&data.scoped_defined_names, &mut worksheets);
        Ok(Self {
            worksheets,
            chart_sheets: Vec::new(),
            sst: data.sst,
            // Seeded from the file's own style table, so the indices every cell already
            // carries still mean what they meant. Without this, saving rebuilt styles.xml from
            // an empty registry and the workbook lost all its formatting.
            styles: {
                let mut registry = StyleRegistry::new();
                registry.import_parsed(&parsed_styles);
                registry
            },
            defined_names: data.defined_names,
            scoped_defined_names: data.scoped_defined_names,
            properties: data.properties,
            passthrough_entries: Vec::new(),
            workbook_protection: None,
            is_xlsm: false,
            calc_mode: None,
            active_sheet: None,
            parsed_styles: Some(parsed_styles),
            sst_threshold: None,
            custom_properties: Vec::new(),
            custom_xml_parts: Vec::new(),
        })
    }

    fn from_xlsx_data_edit<R: std::io::Read + std::io::Seek>(
        data: xlsx_reader::XlsxData,
        zip: &mut crate::zip::zip_reader::ZipReader<R>,
    ) -> crate::Result<Self> {
        let parsed_styles = Arc::new(data.styles);
        let mut worksheets = Vec::with_capacity(data.sheets.len());
        for (i, sheet_info) in data.sheets.iter().enumerate() {
            let mut ws = Worksheet::new(&sheet_info.name);
            if let Some(raw) = zip.read_entry(&sheet_info.path) {
                ws.raw_xml = Some(raw?);
            }
            let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", i + 1);
            if let Some(Ok(rels_data)) = zip.read_entry(&rels_path) {
                ws.original_rels = Some(rels_data);
            }
            ws.parsed_styles = Some(Arc::clone(&parsed_styles));
            ws.visibility = match sheet_info.visibility {
                1 => crate::worksheet::SheetVisibility::Hidden,
                2 => crate::worksheet::SheetVisibility::VeryHidden,
                _ => crate::worksheet::SheetVisibility::Visible,
            };
            worksheets.push(ws);
        }
        let known_prefixes = [
            "xl/worksheets/",
            "xl/workbook.xml",
            "xl/sharedStrings.xml",
            "xl/styles.xml",
            "xl/theme/",
            "[Content_Types].xml",
            "_rels/",
            "xl/_rels/workbook.xml.rels",
            "docProps/",
        ];
        let mut passthrough = Vec::new();
        let entry_names: Vec<String> = (0..zip.archive.len())
            .filter_map(|i| {
                zip.archive
                    .by_index_raw(i)
                    .ok()
                    .map(|e| e.name().to_string())
            })
            .collect();
        for name in &entry_names {
            if !known_prefixes
                .iter()
                .any(|p| name.starts_with(p) || name == *p)
                && let Some(Ok(bytes)) = zip.read_entry(name)
            {
                passthrough.push((name.clone(), bytes));
            }
        }
        Ok(Self {
            worksheets,
            chart_sheets: Vec::new(),
            sst: data.sst,
            // Seeded from the file's own style table, so the indices every cell already
            // carries still mean what they meant. Without this, saving rebuilt styles.xml from
            // an empty registry and the workbook lost all its formatting.
            styles: {
                let mut registry = StyleRegistry::new();
                registry.import_parsed(&parsed_styles);
                registry
            },
            defined_names: data.defined_names,
            scoped_defined_names: data.scoped_defined_names,
            properties: data.properties,
            passthrough_entries: passthrough,
            workbook_protection: None,
            is_xlsm: false,
            calc_mode: None,
            active_sheet: None,
            parsed_styles: Some(parsed_styles),
            sst_threshold: None,
            custom_properties: Vec::new(),
            custom_xml_parts: Vec::new(),
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
            return Err(crate::Error::InvalidData(format!(
                "Sheet '{name}' already exists"
            )));
        }
        self.worksheets.push(Worksheet::new(name));
        Ok(self.worksheets.last_mut().unwrap())
    }

    /// Add a chart sheet — a dedicated sheet that contains only a chart (no cells).
    pub fn add_chart_sheet(
        &mut self,
        name: &str,
        chart: crate::features::chart::Chart,
    ) -> crate::Result<()> {
        crate::worksheet::validate_sheet_name(name)?;
        if self.worksheets.iter().any(|ws| ws.name == name)
            || self.chart_sheets.iter().any(|cs| cs.name == name)
        {
            return Err(crate::Error::InvalidData(format!(
                "Sheet '{name}' already exists"
            )));
        }
        self.chart_sheets.push(ChartSheet {
            name: name.to_string(),
            chart,
        });
        Ok(())
    }

    pub fn worksheet(&mut self, index: usize) -> crate::Result<&mut Worksheet> {
        // Borrow parsed_styles before the mutable borrow of worksheets
        let styles_for_parse: Arc<ParsedStyles> = self
            .parsed_styles
            .as_ref()
            .cloned()
            .unwrap_or_else(|| Arc::new(ParsedStyles::default()));
        let ws = self
            .worksheets
            .get_mut(index)
            .ok_or_else(|| crate::Error::SheetNotFound(format!("index {index}")))?;
        // `!ws.deserialized` is the part that matters: without it this re-parses a sheet whose
        // cells have already been taken into memory and edited, undoing the edit.
        if ws.raw_xml.is_some() && ws.read_cells.is_none() && !ws.deserialized {
            let raw = ws.raw_xml.as_ref().unwrap();
            let (cells, meta) =
                crate::reader::sheet_reader::read_sheet_full(raw, &self.sst, &styles_for_parse)?;
            let map = cells
                .iter()
                .map(|rc| ((rc.row, rc.col), rc.value.clone()))
                .collect();
            ws.read_cells_map = Some(map);
            ws.read_cells = Some(cells);
            ws.merge_ranges = meta.merge_ranges;
            for (c, w) in meta.col_widths {
                ws.col_widths.insert(c, w);
            }
            for (r, h) in meta.row_heights {
                ws.row_heights.insert(r, h);
            }
            ws.freeze_row = meta.freeze_row;
            ws.freeze_col = meta.freeze_col;
            ws.original_drawing_rid = meta.drawing_rid;
            ws.original_legacy_drawing_rid = meta.legacy_drawing_rid;
            // Populate print settings from sheet XML
            ws.print_settings = meta.print_settings;
            // Populate sheet protection from sheet XML
            ws.protection = meta.protection;
            // Populate row/column outline levels from sheet XML
            ws.row_outline_levels = meta.row_outline_levels;
            ws.col_outline_levels = meta.col_outline_levels;
            // Parse conditional formatting rules from the sheet XML
            ws.conditional_formats =
                cf_reader::parse_conditional_formats(raw, &styles_for_parse.dxf_records);
            // Parse data validation rules from the sheet XML
            ws.validations = validation_reader::parse_data_validations(raw);
            // Parse sparklines from the extLst section of the sheet XML
            ws.sparklines = sparkline_reader::parse_sparklines(raw);
            // Resolve hyperlinks from parsed sheet metadata + original rels
            let rels_data = ws.original_rels.clone();
            resolve_hyperlinks_for_worksheet(&mut *ws, &meta.hyperlinks, rels_data.as_deref());
        }
        Ok(ws)
    }

    pub fn worksheet_by_name(&mut self, name: &str) -> crate::Result<&mut Worksheet> {
        let idx = self
            .worksheets
            .iter()
            .position(|ws| ws.name == name)
            .ok_or_else(|| crate::Error::SheetNotFound(name.to_string()))?;
        self.worksheet(idx)
    }

    pub fn remove_worksheet(&mut self, index: usize) -> crate::Result<()> {
        if index >= self.worksheets.len() {
            return Err(crate::Error::SheetNotFound(format!("index {index}")));
        }
        if self.worksheets.len() == 1 {
            return Err(crate::Error::InvalidData(
                "Cannot remove the last worksheet".into(),
            ));
        }
        self.worksheets.remove(index);
        Ok(())
    }

    pub fn rename_worksheet(&mut self, index: usize, name: &str) -> crate::Result<()> {
        crate::worksheet::validate_sheet_name(name)?;
        if self.worksheets.iter().any(|ws| ws.name == name) {
            return Err(crate::Error::InvalidData(format!(
                "Sheet '{name}' already exists"
            )));
        }
        self.worksheets
            .get_mut(index)
            .ok_or_else(|| crate::Error::SheetNotFound(format!("index {index}")))?
            .name = name.to_string();
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

    pub fn sheet_names(&self) -> Vec<&str> {
        self.worksheets.iter().map(|ws| ws.name.as_str()).collect()
    }
    pub fn sheet_count(&self) -> usize {
        self.worksheets.len()
    }
    pub fn worksheet_ref(&self, index: usize) -> crate::Result<&Worksheet> {
        self.worksheets
            .get(index)
            .ok_or(crate::Error::SheetNotFound(format!("index {index}")))
    }

    // ── Properties & settings ──

    pub fn define_name(&mut self, name: &str, formula: &str) -> &mut Self {
        self.defined_names
            .push((name.to_string(), formula.to_string()));
        self.scoped_defined_names.push(DefinedName {
            name: name.to_string(),
            formula: formula.to_string(),
            scope: DefinedNameScope::Workbook,
        });
        self
    }

    /// Define a name scoped to a specific sheet (0-based index).
    pub fn define_name_scoped(
        &mut self,
        name: &str,
        formula: &str,
        sheet_index: usize,
    ) -> &mut Self {
        self.scoped_defined_names.push(DefinedName {
            name: name.to_string(),
            formula: formula.to_string(),
            scope: DefinedNameScope::Sheet(sheet_index),
        });
        self
    }

    /// Add a named range with the given scope.
    ///
    /// This is the preferred CRUD method for managing defined names.
    /// Returns an error if a name with the same name and scope already exists.
    pub fn add_named_range(
        &mut self,
        name: &str,
        formula: &str,
        scope: DefinedNameScope,
    ) -> crate::Result<&mut Self> {
        // Check for duplicates with same name and scope
        let exists = self
            .scoped_defined_names
            .iter()
            .any(|dn| dn.name == name && dn.scope == scope);
        if exists {
            return Err(crate::Error::InvalidData(format!(
                "Defined name '{}' already exists with the same scope",
                name
            )));
        }
        self.scoped_defined_names.push(DefinedName {
            name: name.to_string(),
            formula: formula.to_string(),
            scope: scope.clone(),
        });
        // Keep legacy list in sync for workbook-scoped names
        if scope == DefinedNameScope::Workbook {
            self.defined_names
                .push((name.to_string(), formula.to_string()));
        }
        Ok(self)
    }

    /// Update the formula of an existing named range.
    ///
    /// Updates the first defined name matching the given name (regardless of scope).
    /// Returns an error if no defined name with that name exists.
    pub fn update_named_range(
        &mut self,
        name: &str,
        new_formula: &str,
    ) -> crate::Result<&mut Self> {
        let found = self
            .scoped_defined_names
            .iter_mut()
            .find(|dn| dn.name == name);
        match found {
            Some(dn) => {
                dn.formula = new_formula.to_string();
                // Keep legacy list in sync
                if let Some(legacy) = self.defined_names.iter_mut().find(|(n, _)| n == name) {
                    legacy.1 = new_formula.to_string();
                }
                Ok(self)
            }
            None => Err(crate::Error::InvalidData(format!(
                "Defined name '{}' not found",
                name
            ))),
        }
    }

    /// Remove a named range by name and scope.
    ///
    /// Returns an error if no defined name with that name and scope exists.
    pub fn remove_named_range(
        &mut self,
        name: &str,
        scope: &DefinedNameScope,
    ) -> crate::Result<&mut Self> {
        let pos = self
            .scoped_defined_names
            .iter()
            .position(|dn| dn.name == name && &dn.scope == scope);
        match pos {
            Some(idx) => {
                self.scoped_defined_names.remove(idx);
                // Keep legacy list in sync for workbook-scoped names
                if *scope == DefinedNameScope::Workbook
                    && let Some(legacy_pos) = self.defined_names.iter().position(|(n, _)| n == name)
                {
                    self.defined_names.remove(legacy_pos);
                }
                Ok(self)
            }
            None => Err(crate::Error::InvalidData(format!(
                "Defined name '{}' not found with the specified scope",
                name
            ))),
        }
    }

    /// Returns the legacy defined names as (name, formula) pairs (workbook-scoped only).
    pub fn defined_names(&self) -> &[(String, String)] {
        &self.defined_names
    }

    /// Returns all defined names with scope information.
    pub fn defined_names_with_scope(&self) -> &[DefinedName] {
        &self.scoped_defined_names
    }
    pub fn set_properties(&mut self, props: DocProperties) -> &mut Self {
        self.properties = props;
        self
    }
    pub fn properties(&self) -> &DocProperties {
        &self.properties
    }

    pub fn protect(&mut self) -> &mut Self {
        self.workbook_protection = Some(WorkbookProtection {
            password_hash: None,
        });
        self
    }
    pub fn protect_with_password(&mut self, password: &str) -> &mut Self {
        self.workbook_protection = Some(WorkbookProtection {
            password_hash: Some(crate::worksheet::hash_password_public(password)),
        });
        self
    }

    pub fn set_calc_mode(&mut self, mode: CalcMode) -> &mut Self {
        self.calc_mode = Some(mode);
        self
    }

    /// Set the SST deduplication threshold.
    ///
    /// Strings that appear fewer than `n` times will be inlined in the cell
    /// rather than stored in the shared string table. This can reduce file size
    /// when many strings are unique. Default is 1 (all strings go to SST).
    pub fn set_sst_threshold(&mut self, n: usize) -> &mut Self {
        self.sst_threshold = Some(n);
        self
    }
    pub fn set_active_sheet(&mut self, index: usize) -> &mut Self {
        self.active_sheet = Some(index);
        self
    }

    /// Save the workbook using parallel sheet serialization.
    ///
    /// This is equivalent to `save()` — the existing save pipeline already
    /// serializes sheets in parallel using `std::thread::scope`. This method
    /// is provided as an explicit API for callers who want to signal intent.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_parallel(&mut self, path: impl AsRef<Path>) -> crate::Result<()> {
        // The existing save_to_buffer already uses std::thread::scope for
        // parallel sheet XML generation. This method delegates to it.
        self.save(path)
    }

    /// Recalculate all formula cells across all worksheets.
    ///
    /// Parses every formula cell into an AST, builds a dependency graph,
    /// detects circular references, and evaluates cells in topological order.
    /// Volatile functions (`RAND`, `NOW`, `TODAY`, `INDIRECT`) and their
    /// dependents are re-evaluated automatically.
    ///
    /// Returns the number of cells evaluated, or an error if a circular
    /// reference is detected.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zavora_xlsx::Workbook;
    ///
    /// let mut wb = Workbook::new();
    /// let ws = wb.worksheet(0).unwrap();
    /// ws.write(0, 0, 10.0).unwrap();
    /// ws.write(0, 1, 20.0).unwrap();
    /// ws.write_formula(0, 2, "A1+B1").unwrap();
    ///
    /// let count = wb.recalculate().unwrap();
    /// assert_eq!(count, 1);
    ///
    /// let ws = wb.worksheet(0).unwrap();
    /// let val = ws.read_cell(0, 2);
    /// // val is now CellValue::Number(30.0) (via cached formula result)
    /// ```
    pub fn recalculate(&mut self) -> crate::Result<usize> {
        use crate::cell::CellType;
        use crate::formula_engine::recalc;
        use crate::formula_engine::{CellAddr, parse, tokenize};

        // Ensure all worksheets are deserialized so we can read their cells
        for i in 0..self.worksheets.len() {
            // Access each worksheet mutably to trigger lazy deserialization
            let _ = self.worksheet(i);
        }

        // Collect all formula cells across all sheets
        let mut formulas = Vec::new();
        for (sheet_idx, ws) in self.worksheets.iter().enumerate() {
            for (&row, cols) in &ws.cells {
                for (&col, (cell, _)) in cols {
                    let formula_text = match cell {
                        CellType::Formula { text, .. } => Some(text.as_str()),
                        CellType::ArrayFormula { text, .. } => Some(text.as_str()),
                        CellType::DynamicFormula { text, .. } => Some(text.as_str()),
                        _ => None,
                    };
                    if let Some(text) = formula_text
                        && let Ok(tokens) = tokenize(text)
                        && let Ok(ast) = parse(&tokens)
                    {
                        formulas.push((
                            CellAddr {
                                sheet: sheet_idx,
                                row,
                                col,
                            },
                            ast,
                        ));
                    }
                }
            }
        }

        if formulas.is_empty() {
            return Ok(0);
        }

        // Build a mutable context that reads/writes from the worksheets
        let mut ctx = WorkbookCellContext {
            worksheets: &mut self.worksheets,
        };

        recalc::recalculate(&formulas, &mut ctx)
            .map_err(|e| crate::Error::InvalidData(e.to_string()))
    }

    /// Access the passthrough entries (preserved ZIP entries from the original file).
    pub fn passthrough_entries(&self) -> &[(String, Vec<u8>)] {
        &self.passthrough_entries
    }

    /// Add a raw passthrough entry (for preserving custom parts).
    pub fn add_passthrough_entry(&mut self, name: &str, data: Vec<u8>) -> &mut Self {
        self.passthrough_entries.push((name.to_string(), data));
        self
    }

    // ── Custom Properties (Task 76) ──

    /// Set a custom document property.
    pub fn set_custom_property(
        &mut self,
        name: &str,
        value: crate::properties::CustomPropertyValue,
    ) -> &mut Self {
        // Replace existing property with same name
        self.custom_properties.retain(|p| p.name != name);
        self.custom_properties
            .push(crate::properties::CustomProperty {
                name: name.to_string(),
                value,
            });
        self
    }

    /// Get all custom document properties.
    pub fn custom_properties(&self) -> &[crate::properties::CustomProperty] {
        &self.custom_properties
    }

    // ── Custom XML Parts (Task 75) ──

    /// Add a custom XML part with a namespace identifier and content.
    pub fn add_custom_xml(&mut self, namespace: &str, content: &[u8]) -> &mut Self {
        self.custom_xml_parts
            .push((namespace.to_string(), content.to_vec()));
        self
    }

    /// Read a custom XML part by namespace.
    pub fn read_custom_xml(&self, namespace: &str) -> Option<&[u8]> {
        self.custom_xml_parts
            .iter()
            .find(|(ns, _)| ns == namespace)
            .map(|(_, data)| data.as_slice())
    }

    /// Get all custom XML parts as (namespace, content) pairs.
    pub fn custom_xml_parts(&self) -> &[(String, Vec<u8>)] {
        &self.custom_xml_parts
    }

    // ── Digital Signatures (Task 74) ──

    /// Sign the workbook with a certificate (skeleton — returns UnsupportedFormat).
    ///
    /// Digital signature support requires the `crypto` feature flag and is not
    /// yet implemented. This method is provided as a placeholder.
    pub fn sign(&mut self, _certificate: &[u8]) -> crate::Result<()> {
        Err(crate::Error::UnsupportedFormat(
            "Digital signatures are not yet implemented. Enable the 'crypto' feature flag when available.".to_string()
        ))
    }

    /// Sign the VBA project with a certificate (skeleton — returns UnsupportedFormat).
    ///
    /// VBA project signing generates a digital signature stored in
    /// `xl/vbaProjectSignature.bin`. This requires the `crypto` feature flag
    /// and is not yet implemented.
    pub fn sign_vba(&mut self, _certificate: &[u8]) -> crate::Result<()> {
        Err(crate::Error::UnsupportedFormat(
            "VBA project signing is not yet implemented. Enable the 'crypto' feature flag when available.".to_string()
        ))
    }

    /// Verify the workbook's digital signature (skeleton — returns UnsupportedFormat).
    ///
    /// Digital signature verification requires the `crypto` feature flag and is
    /// not yet implemented. This method is provided as a placeholder.
    pub fn verify_signature(&self) -> crate::Result<bool> {
        Err(crate::Error::UnsupportedFormat(
            "Digital signature verification is not yet implemented. Enable the 'crypto' feature flag when available.".to_string()
        ))
    }

    /// Add an external data connection to the workbook.
    ///
    /// This creates a connection entry in `xl/connections.xml`. The connection
    /// is stored in passthrough entries and will be preserved during save.
    pub fn add_connection(&mut self, connection_string: &str, command: &str) -> &mut Self {
        let conn_id = self
            .passthrough_entries
            .iter()
            .filter(|(n, _)| n == "xl/connections.xml")
            .count()
            + 1;
        let xml = generate_connections_xml(conn_id, connection_string, command);
        // Remove existing connections.xml if present, then add updated one
        self.passthrough_entries
            .retain(|(n, _)| n != "xl/connections.xml");
        self.passthrough_entries
            .push(("xl/connections.xml".to_string(), xml));
        self
    }

    pub fn pictures<R: std::io::Read + std::io::Seek>(
        zip: &mut crate::zip::zip_reader::ZipReader<R>,
    ) -> Vec<(String, Vec<u8>)> {
        let mut pics = Vec::new();
        let names: Vec<String> = (0..zip.archive.len())
            .filter_map(|i| {
                zip.archive
                    .by_index_raw(i)
                    .ok()
                    .map(|e| e.name().to_string())
            })
            .collect();
        for name in names {
            if name.starts_with("xl/media/")
                && let Some(Ok(data)) = zip.read_entry(&name)
            {
                pics.push((name.rsplit('/').next().unwrap_or(&name).to_string(), data));
            }
        }
        pics
    }

    // ── Serde Integration (Task 83) ──

    /// Serialize a slice of structs to worksheet rows.
    ///
    /// Row 0 gets the header (field names), and each struct becomes a subsequent row.
    /// Requires the `serde-support` feature flag.
    #[cfg(feature = "serde-support")]
    pub fn write_rows<T: serde::Serialize>(
        &mut self,
        sheet: usize,
        data: &[T],
    ) -> crate::Result<()> {
        let ws = self.worksheet(sheet)?;
        crate::serde_support::ser::write_rows(ws, data)
    }

    /// Deserialize worksheet rows into a Vec<T>.
    ///
    /// Row 0 is treated as the header row (field names). Each subsequent row
    /// is deserialized into a T using the header-to-field mapping.
    /// Requires the `serde-support` feature flag.
    #[cfg(feature = "serde-support")]
    pub fn read_rows<T: serde::de::DeserializeOwned>(
        &mut self,
        sheet: usize,
    ) -> crate::Result<Vec<T>> {
        // Ensure the sheet is deserialized first
        let ws = self.worksheet(sheet)?;
        crate::serde_support::de::read_rows(ws)
    }

    // ── Async I/O (Task 85) ──

    /// Open a workbook asynchronously using tokio's blocking thread pool.
    ///
    /// Requires the `async-tokio` feature flag.
    #[cfg(feature = "async-tokio")]
    pub async fn open_async(path: impl AsRef<Path> + Send + 'static) -> crate::Result<Self> {
        let path = path.as_ref().to_path_buf();
        tokio::task::spawn_blocking(move || Self::open(path))
            .await
            .map_err(|e| crate::Error::InvalidData(format!("async task failed: {}", e)))?
    }

    /// Save the workbook asynchronously using tokio's blocking thread pool.
    ///
    /// Requires the `async-tokio` feature flag.
    #[cfg(feature = "async-tokio")]
    pub async fn save_async(&mut self, path: impl AsRef<Path>) -> crate::Result<()> {
        let bytes = self.save_to_buffer()?;
        let path = path.as_ref().to_path_buf();
        tokio::task::spawn_blocking(move || std::fs::write(path, bytes))
            .await
            .map_err(|e| crate::Error::InvalidData(format!("async task failed: {}", e)))??;
        Ok(())
    }
}

impl Default for Workbook {
    fn default() -> Self {
        Self::new()
    }
}

// ── Recalculation context ──────────────────────────────────────────────────────

/// A cell context that reads/writes directly from/to Workbook worksheets.
/// Used by `Workbook::recalculate()` to bridge the formula engine to live cell data.
struct WorkbookCellContext<'a> {
    worksheets: &'a mut Vec<Worksheet>,
}

impl crate::formula_engine::CellContext for WorkbookCellContext<'_> {
    fn get_cell(&self, sheet: usize, row: u32, col: u16) -> crate::formula_engine::Value {
        if let Some(ws) = self.worksheets.get(sheet) {
            let cv = ws.read_cell(row, col);
            cell_value_to_engine_value(&cv)
        } else {
            crate::formula_engine::Value::Error(crate::formula_engine::ErrorKind::Ref)
        }
    }
}

impl crate::formula_engine::MutableCellContext for WorkbookCellContext<'_> {
    fn set_cell(&mut self, sheet: usize, row: u32, col: u16, val: crate::formula_engine::Value) {
        if let Some(ws) = self.worksheets.get_mut(sheet) {
            match val {
                crate::formula_engine::Value::Number(n) => {
                    // Update the cached value in the formula cell
                    if let Some(cols) = ws.cells.get_mut(&row)
                        && let Some((cell, _)) = cols.get_mut(&col)
                        && let crate::cell::CellType::Formula { cached_number, .. } = cell
                    {
                        *cached_number = Some(n);
                        return;
                    }
                    let _ = ws.write(row, col, n);
                }
                crate::formula_engine::Value::String(s) => {
                    let _ = ws.write(row, col, s.as_str());
                }
                crate::formula_engine::Value::Bool(b) => {
                    let _ = ws.write(row, col, b);
                }
                _ => {}
            }
        }
    }
}

/// Convert a `CellValue` (from worksheet) to a formula engine `Value`.
fn cell_value_to_engine_value(cv: &crate::cell::CellValue) -> crate::formula_engine::Value {
    match cv {
        crate::cell::CellValue::Number(n) => crate::formula_engine::Value::Number(*n),
        crate::cell::CellValue::String(s) => crate::formula_engine::Value::String(s.clone()),
        crate::cell::CellValue::Bool(b) => crate::formula_engine::Value::Bool(*b),
        crate::cell::CellValue::DateTime(dt) => crate::formula_engine::Value::Number(dt.serial()),
        crate::cell::CellValue::Error(_) => {
            crate::formula_engine::Value::Error(crate::formula_engine::ErrorKind::Value)
        }
        crate::cell::CellValue::Formula { cached_value, .. } => {
            cell_value_to_engine_value(cached_value)
        }
        crate::cell::CellValue::RichText(rt) => {
            crate::formula_engine::Value::String(rt.plain_text())
        }
        crate::cell::CellValue::Empty => crate::formula_engine::Value::Empty,
    }
}

/// Parse `_xlnm.Print_Titles` defined names and populate repeat rows/columns
/// on the corresponding worksheets.
///
/// The formula for Print_Titles can contain row references like `Sheet1!$1:$3`
/// and/or column references like `Sheet1!$A:$B`, separated by commas.
fn apply_print_titles(defined_names: &[DefinedName], worksheets: &mut [Worksheet]) {
    for dn in defined_names {
        if !dn.name.eq_ignore_ascii_case("_xlnm.Print_Titles") {
            continue;
        }
        let sheet_idx = match &dn.scope {
            DefinedNameScope::Sheet(idx) => *idx,
            DefinedNameScope::Workbook => {
                // Try to infer sheet index from the formula reference
                // e.g. "Sheet1!$1:$3" → find sheet named "Sheet1"
                if let Some(sheet_name) = extract_sheet_name(&dn.formula) {
                    match worksheets.iter().position(|ws| ws.name == sheet_name) {
                        Some(idx) => idx,
                        None => continue,
                    }
                } else {
                    continue;
                }
            }
        };
        if sheet_idx >= worksheets.len() {
            continue;
        }
        let ws = &mut worksheets[sheet_idx];
        let ps = ws
            .print_settings
            .get_or_insert_with(crate::worksheet::PrintSettings::default);

        // Parse the formula parts (comma-separated)
        for part in dn.formula.split(',') {
            let part = part.trim();
            // Strip sheet name prefix (e.g. "Sheet1!" or "'Sheet 1'!")
            let ref_part = if let Some(idx) = part.rfind('!') {
                &part[idx + 1..]
            } else {
                part
            };

            // Check if it's a row reference like $1:$3
            if let Some((first, last)) = parse_row_range(ref_part) {
                ps.repeat_rows = Some((first, last));
            }
            // Check if it's a column reference like $A:$B
            else if let Some((first, last)) = parse_col_range(ref_part) {
                ps.repeat_cols = Some((first, last));
            }
        }
    }
}

/// Extract sheet name from a formula reference like "Sheet1!$1:$3" or "'My Sheet'!$A:$B"
fn extract_sheet_name(formula: &str) -> Option<String> {
    // Take the first part before comma
    let part = formula.split(',').next()?;
    let bang_idx = part.rfind('!')?;
    let name = &part[..bang_idx];
    // Remove surrounding quotes if present
    let name = name.trim_matches('\'');
    Some(name.to_string())
}

/// Parse a row range like "$1:$3" → (0, 2) (0-based)
fn parse_row_range(s: &str) -> Option<(u32, u32)> {
    let s = s.replace('$', "");
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let first: u32 = parts[0].parse().ok()?;
    let last: u32 = parts[1].parse().ok()?;
    // Verify these are pure numbers (row references)
    if first == 0 || last == 0 {
        return None;
    }
    Some((first - 1, last - 1))
}

/// Parse a column range like "$A:$B" → (0, 1) (0-based)
fn parse_col_range(s: &str) -> Option<(u16, u16)> {
    let s = s.replace('$', "");
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    // Verify these are pure letters (column references)
    if !parts[0].chars().all(|c| c.is_ascii_alphabetic())
        || !parts[1].chars().all(|c| c.is_ascii_alphabetic())
    {
        return None;
    }
    let first = col_letter_to_num(parts[0])?;
    let last = col_letter_to_num(parts[1])?;
    Some((first, last))
}

/// Convert column letter(s) to 0-based column number: "A" → 0, "B" → 1, "AA" → 26
fn col_letter_to_num(s: &str) -> Option<u16> {
    if s.is_empty() {
        return None;
    }
    let mut num: u16 = 0;
    for c in s.chars() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        num = num
            .checked_mul(26)?
            .checked_add(c.to_ascii_uppercase() as u16 - b'A' as u16 + 1)?;
    }
    Some(num - 1)
}

/// Read charts associated with a worksheet by following drawing relationships.
///
/// 1. Resolve the drawing rId to a drawing path via the sheet rels
/// 2. Parse the drawing XML to find chart relationship IDs
/// 3. Resolve chart rIds to chart part paths via the drawing rels
/// 4. Read and parse each chart XML into Chart / TreemapChart structs
fn read_charts_for_worksheet<R: std::io::Read + std::io::Seek>(
    ws: &mut Worksheet,
    zip: &mut crate::zip::zip_reader::ZipReader<R>,
    drawing_rid: &str,
    sheet_rels_data: &[u8],
) {
    // Step 1: Find the drawing path from the sheet rels
    let sheet_rels = match crate::reader::rel_parser::parse_rels(sheet_rels_data) {
        Ok(r) => r,
        Err(_) => return,
    };
    let drawing_target = match sheet_rels.iter().find(|r| r.id == drawing_rid) {
        Some(rel) => &rel.target,
        None => return,
    };
    let drawing_path = if let Some(stripped) = drawing_target.strip_prefix("../") {
        format!("xl/{}", stripped)
    } else if let Some(stripped) = drawing_target.strip_prefix("/xl/") {
        stripped.to_string()
    } else if drawing_target.starts_with("xl/") {
        drawing_target.to_string()
    } else {
        format!("xl/drawings/{drawing_target}")
    };

    // Step 2: Read the drawing XML and parse chart references
    let drawing_data = match zip.read_entry(&drawing_path) {
        Some(Ok(d)) => d,
        _ => return,
    };
    let chart_refs = chart_reader::parse_drawing_chart_refs(&drawing_data);
    if chart_refs.is_empty() {
        return;
    }

    // Step 3: Read the drawing rels to resolve chart paths
    // Drawing rels path: e.g. xl/drawings/_rels/drawing1.xml.rels
    let drawing_filename = drawing_path.rsplit('/').next().unwrap_or("");
    let drawing_dir = drawing_path
        .rsplit_once('/')
        .map(|(d, _)| d)
        .unwrap_or("xl/drawings");
    let drawing_rels_path = format!("{drawing_dir}/_rels/{drawing_filename}.rels");
    let drawing_rels_data = match zip.read_entry(&drawing_rels_path) {
        Some(Ok(d)) => d,
        _ => return,
    };
    let chart_paths = chart_reader::resolve_chart_paths(&chart_refs, &drawing_rels_data);

    // Step 4: Read and parse each chart
    for (chart_path, is_chartex) in &chart_paths {
        if let Some(Ok(chart_data)) = zip.read_entry(chart_path) {
            if *is_chartex {
                if let Ok(tc) = chart_reader::read_chartex(&chart_data) {
                    ws.treemap_charts.push(tc);
                }
            } else if let Ok(chart) = chart_reader::read_chart(&chart_data) {
                ws.charts.push(chart);
            }
        }
    }
}

/// Read tables associated with a worksheet by following table relationships
/// in the sheet rels file.
fn read_tables_for_worksheet<R: std::io::Read + std::io::Seek>(
    ws: &mut Worksheet,
    zip: &mut crate::zip::zip_reader::ZipReader<R>,
    sheet_rels_data: &[u8],
) {
    let rels = match crate::reader::rel_parser::parse_rels(sheet_rels_data) {
        Ok(r) => r,
        Err(_) => return,
    };

    let table_rel_type =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/table";
    for rel in &rels {
        if rel.rel_type != table_rel_type {
            continue;
        }
        let target = &rel.target;
        let full_path = if let Some(stripped) = target.strip_prefix("../") {
            format!("xl/{}", stripped)
        } else if let Some(stripped) = target.strip_prefix("/xl/") {
            stripped.to_string()
        } else if target.starts_with("xl/") {
            target.to_string()
        } else {
            format!("xl/tables/{target}")
        };
        if let Some(Ok(table_data)) = zip.read_entry(&full_path)
            && let Ok(table) = table_reader::read_table(&table_data)
        {
            ws.tables.push(table);
        }
    }
}

/// Resolve parsed hyperlinks from sheet XML into Hyperlink structs on the worksheet.
///
/// External hyperlinks have an `r:id` attribute that references a relationship
/// in the sheet rels file. Internal hyperlinks have a `location` attribute directly.
fn resolve_hyperlinks_for_worksheet(
    ws: &mut Worksheet,
    parsed: &[crate::reader::sheet_reader::ParsedHyperlink],
    sheet_rels_data: Option<&[u8]>,
) {
    use std::collections::HashMap;

    if parsed.is_empty() {
        return;
    }

    // Build a map of rId → target URL from the sheet rels
    let rels_map: HashMap<String, String> = if let Some(data) = sheet_rels_data {
        match crate::reader::rel_parser::parse_rels(data) {
            Ok(rels) => rels.into_iter().map(|r| (r.id, r.target)).collect(),
            Err(_) => HashMap::new(),
        }
    } else {
        HashMap::new()
    };

    for ph in parsed {
        let (row, col) = match crate::utility::parse_cell_ref(&ph.cell_ref) {
            Ok(rc) => rc,
            Err(_) => continue,
        };

        let url = if let Some(ref rid) = ph.rid {
            rels_map.get(rid).cloned().unwrap_or_default()
        } else {
            String::new()
        };

        ws.hyperlinks.push(crate::worksheet::Hyperlink {
            row,
            col,
            url,
            location: ph.location.clone(),
            tooltip: ph.tooltip.clone(),
        });
    }
}

/// Read comments associated with a worksheet by following comment relationships
/// in the sheet rels file.
fn read_comments_for_worksheet<R: std::io::Read + std::io::Seek>(
    ws: &mut Worksheet,
    zip: &mut crate::zip::zip_reader::ZipReader<R>,
    sheet_rels_data: &[u8],
) {
    if let Some(comments_path) = comment_reader::find_comments_path_from_rels(sheet_rels_data)
        && let Some(Ok(comments_data)) = zip.read_entry(&comments_path)
    {
        let parsed = comment_reader::parse_comments(&comments_data);
        ws.comments = parsed
            .into_iter()
            .map(|pc| crate::worksheet::Comment {
                row: pc.row,
                col: pc.col,
                text: pc.text,
                author: pc.author,
            })
            .collect();
    }
}

/// Read slicers associated with a worksheet by following slicer relationships
/// in the sheet rels file.
fn read_slicers_for_worksheet<R: std::io::Read + std::io::Seek>(
    ws: &mut Worksheet,
    zip: &mut crate::zip::zip_reader::ZipReader<R>,
    sheet_rels_data: &[u8],
) {
    let slicer_paths = slicer_reader::find_slicer_paths_from_rels(sheet_rels_data);
    for path in slicer_paths {
        if let Some(Ok(slicer_data)) = zip.read_entry(&path) {
            let parsed = slicer_reader::parse_slicers(&slicer_data);
            ws.slicers.extend(parsed);
        }
    }
}

/// Read timelines associated with a worksheet by following timeline relationships
/// in the sheet rels file.
fn read_timelines_for_worksheet<R: std::io::Read + std::io::Seek>(
    ws: &mut Worksheet,
    zip: &mut crate::zip::zip_reader::ZipReader<R>,
    sheet_rels_data: &[u8],
) {
    let timeline_paths =
        crate::reader::timeline_reader::find_timeline_paths_from_rels(sheet_rels_data);
    for path in timeline_paths {
        if let Some(Ok(timeline_data)) = zip.read_entry(&path) {
            let parsed = crate::reader::timeline_reader::parse_timelines(&timeline_data);
            ws.timelines.extend(parsed);
        }
    }
}

/// Generate xl/connections.xml content for an external data connection.
fn generate_connections_xml(id: usize, connection_string: &str, command: &str) -> Vec<u8> {
    use crate::xml::xml_writer::XmlWriter;
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "connections",
        &[(
            "xmlns",
            "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
        )],
    );
    let id_s = id.to_string();
    w.start_tag(
        "connection",
        &[
            ("id", &id_s),
            ("name", &format!("Connection{id}")),
            ("type", "1"),
        ],
    );
    w.empty_tag(
        "dbPr",
        &[("connection", connection_string), ("command", command)],
    );
    w.end_tag("connection");
    w.end_tag("connections");
    w.into_bytes()
}
