use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::model::shared_strings::SharedStringTable;
use crate::properties::{self, DocProperties};
use crate::reader::style_parser::ParsedStyles;
use crate::reader::{rel_parser, sheet_reader, sst_parser, style_parser};
use crate::xml::xml_reader::get_attr;
use crate::zip::zip_reader::ZipReader;

use crate::workbook::{DefinedName, DefinedNameScope};

pub use crate::reader::sheet_reader::RawCell;

pub struct SheetInfo {
    pub name: String,
    pub path: String,
    pub visibility: u8, // 0=visible, 1=hidden, 2=veryHidden
}

pub struct XlsxData {
    pub sheets: Vec<SheetInfo>,
    pub sst: SharedStringTable,
    pub styles: ParsedStyles,
    #[allow(dead_code)]
    pub is_1904: bool,
    pub defined_names: Vec<(String, String)>,
    pub scoped_defined_names: Vec<DefinedName>,
    pub properties: DocProperties,
}

pub fn read_xlsx(
    path: &Path,
) -> crate::Result<(XlsxData, ZipReader<std::io::BufReader<std::fs::File>>)> {
    let mut zip = ZipReader::open(path)?;
    let data = read_xlsx_from_zip(&mut zip)?;
    Ok((data, zip))
}

pub fn read_xlsx_from_zip<R: std::io::Read + std::io::Seek>(
    zip: &mut ZipReader<R>,
) -> crate::Result<XlsxData> {
    let rels_map = if let Some(data) = zip.read_entry("xl/_rels/workbook.xml.rels") {
        let data = data?;
        let rels = rel_parser::parse_rels(&data)?;
        rels.into_iter()
            .map(|r| (r.id, r.target))
            .collect::<HashMap<_, _>>()
    } else {
        HashMap::new()
    };

    let mut sheets = Vec::new();
    let mut is_1904 = false;
    let mut defined_names = Vec::new();
    let mut scoped_defined_names = Vec::new();

    if let Some(data) = zip.read_entry("xl/workbook.xml") {
        let data = data?;
        let mut reader = Reader::from_reader(data.as_slice());
        reader.config_mut().check_end_names = false;
        reader.config_mut().expand_empty_elements = true;
        let mut buf = Vec::with_capacity(512);
        let mut in_defined_name = false;
        let mut dn_name = String::new();
        let mut dn_value = String::new();
        let mut dn_local_sheet_id: Option<usize> = None;

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) | Event::Empty(e) => match e.local_name().as_ref() {
                    b"workbookPr" => {
                        if let Some(v) = get_attr(e.attributes(), b"date1904") {
                            is_1904 = v == b"1" || v == b"true";
                        }
                    }
                    b"sheet" => {
                        let name = get_attr(e.attributes(), b"name")
                            .and_then(|v| std::str::from_utf8(v).ok())
                            .unwrap_or("")
                            .to_string();
                        let rid = get_attr(e.attributes(), b"r:id")
                            .and_then(|v| std::str::from_utf8(v).ok())
                            .unwrap_or("")
                            .to_string();
                        if let Some(target) = rels_map.get(&rid) {
                            let vis = get_attr(e.attributes(), b"state")
                                .map(|v| {
                                    if v == b"hidden" {
                                        1
                                    } else if v == b"veryHidden" {
                                        2
                                    } else {
                                        0
                                    }
                                })
                                .unwrap_or(0);
                            sheets.push(SheetInfo {
                                name,
                                path: normalize_sheet_path(target),
                                visibility: vis,
                            });
                        }
                    }
                    b"definedName" => {
                        dn_name = get_attr(e.attributes(), b"name")
                            .and_then(|v| std::str::from_utf8(v).ok())
                            .unwrap_or("")
                            .to_string();
                        dn_local_sheet_id = get_attr(e.attributes(), b"localSheetId")
                            .and_then(|v| std::str::from_utf8(v).ok())
                            .and_then(|v| v.parse::<usize>().ok());
                        dn_value.clear();
                        in_defined_name = true;
                    }
                    _ => {}
                },
                Event::Text(e) if in_defined_name => {
                    if let Ok(t) = e.unescape() {
                        dn_value.push_str(&t);
                    }
                }
                Event::End(e) if e.local_name().as_ref() == b"definedName" => {
                    if !dn_name.is_empty() {
                        // Populate legacy defined_names (name, formula) for backward compat
                        defined_names.push((dn_name.clone(), dn_value.clone()));
                        // Populate scoped defined names
                        let scope = match dn_local_sheet_id {
                            Some(idx) => DefinedNameScope::Sheet(idx),
                            None => DefinedNameScope::Workbook,
                        };
                        scoped_defined_names.push(DefinedName {
                            name: dn_name.clone(),
                            formula: dn_value.clone(),
                            scope,
                        });
                    }
                    in_defined_name = false;
                    dn_local_sheet_id = None;
                }
                Event::Eof => break,
                _ => {}
            }
        }
    }

    let sst = if let Some(data) = zip.read_entry("xl/sharedStrings.xml") {
        sst_parser::parse_sst(&data?)?
    } else {
        SharedStringTable::new()
    };

    let styles = if let Some(data) = zip.read_entry("xl/styles.xml") {
        style_parser::parse_styles(&data?)?
    } else {
        ParsedStyles::default()
    };

    let doc_props = if let Some(data) = zip.read_entry("docProps/core.xml") {
        properties::parse_core_xml(&data?)
    } else {
        DocProperties::default()
    };

    let xlsx_data = XlsxData {
        sheets,
        sst,
        styles,
        is_1904,
        defined_names,
        scoped_defined_names,
        properties: doc_props,
    };
    Ok(xlsx_data)
}

#[allow(dead_code)]
pub fn read_sheet_data<R: Read + Seek>(
    zip: &mut ZipReader<R>,
    sheet_path: &str,
    sst: &SharedStringTable,
    styles: &ParsedStyles,
) -> crate::Result<Vec<RawCell>> {
    let data = zip
        .read_entry(sheet_path)
        .ok_or_else(|| crate::Error::SheetNotFound(sheet_path.to_string()))??;
    sheet_reader::read_sheet_cells(&data, sst, styles)
}

fn normalize_sheet_path(target: &str) -> String {
    if target.starts_with("/xl/") {
        target[1..].to_string()
    } else if target.starts_with("xl/") {
        target.to_string()
    } else {
        format!("xl/{target}")
    }
}
