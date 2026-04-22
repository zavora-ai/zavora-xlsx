// XML generation: workbook.xml, content types, root rels

use super::{CalcMode, Workbook};
use crate::utility::col_to_letter;
use crate::writer::rel_writer;
use crate::xml::xml_writer::XmlWriter;

impl Workbook {
    pub(crate) fn write_workbook_xml(&self) -> Vec<u8> {
        let mut w = XmlWriter::new();
        w.declaration();
        w.start_tag(
            "workbook",
            &[
                (
                    "xmlns",
                    "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
                ),
                (
                    "xmlns:r",
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                ),
            ],
        );
        if let Some(ref prot) = self.workbook_protection {
            let mut attrs: Vec<(&str, &str)> = vec![("lockStructure", "1")];
            let pw;
            if let Some(ref hash) = prot.password_hash {
                pw = hash.clone();
                attrs.push(("workbookPassword", &pw));
            }
            w.empty_tag("workbookProtection", &attrs);
        }

        if let Some(idx) = self.active_sheet {
            let idx_s = idx.to_string();
            w.start_tag("bookViews", &[]);
            w.empty_tag("workbookView", &[("activeTab", &idx_s)]);
            w.end_tag("bookViews");
        }

        w.start_tag("sheets", &[]);
        for (i, ws) in self.worksheets.iter().enumerate() {
            let id = (i + 1).to_string();
            let rid = format!("rId{}", i + 1);
            use crate::worksheet::SheetVisibility;
            let mut attrs: Vec<(&str, &str)> =
                vec![("name", &ws.name), ("sheetId", &id), ("r:id", &rid)];
            match ws.visibility {
                SheetVisibility::Hidden => attrs.push(("state", "hidden")),
                SheetVisibility::VeryHidden => attrs.push(("state", "veryHidden")),
                SheetVisibility::Visible => {}
            }
            w.empty_tag("sheet", &attrs);
        }
        for (i, cs) in self.chart_sheets.iter().enumerate() {
            let id = (self.worksheets.len() + i + 1).to_string();
            let rid = format!("rId{}", self.worksheets.len() + i + 1);
            w.empty_tag(
                "sheet",
                &[("name", &cs.name), ("sheetId", &id), ("r:id", &rid)],
            );
        }
        w.end_tag("sheets");

        let mut all_names: Vec<(String, String, Option<usize>)> = Vec::new();
        // Use scoped_defined_names if available (includes scope info), otherwise fall back to legacy
        if !self.scoped_defined_names.is_empty() {
            for dn in &self.scoped_defined_names {
                let local_sheet = match dn.scope {
                    super::DefinedNameScope::Workbook => None,
                    super::DefinedNameScope::Sheet(idx) => Some(idx),
                };
                all_names.push((dn.name.clone(), dn.formula.clone(), local_sheet));
            }
        } else {
            for (name, formula) in &self.defined_names {
                all_names.push((name.clone(), formula.clone(), None));
            }
        }
        for (i, ws) in self.worksheets.iter().enumerate() {
            if let Some(ref ps) = ws.print_settings {
                if let Some((r1, c1, r2, c2)) = ps.print_area {
                    let val = format!(
                        "'{}'!${}${}:${}${}",
                        ws.name,
                        col_to_letter(c1),
                        r1 + 1,
                        col_to_letter(c2),
                        r2 + 1
                    );
                    all_names.push(("_xlnm.Print_Area".into(), val, Some(i)));
                }
                if let Some((first, last)) = ps.repeat_rows {
                    let row_part = format!("'{}'!${}:${}", ws.name, first + 1, last + 1);
                    if let Some((fc, lc)) = ps.repeat_cols {
                        let col_part = format!(
                            "'{}'!${}:${}",
                            ws.name,
                            col_to_letter(fc),
                            col_to_letter(lc)
                        );
                        all_names.push((
                            "_xlnm.Print_Titles".into(),
                            format!("{col_part},{row_part}"),
                            Some(i),
                        ));
                    } else {
                        all_names.push(("_xlnm.Print_Titles".into(), row_part, Some(i)));
                    }
                } else if let Some((fc, lc)) = ps.repeat_cols {
                    let val = format!(
                        "'{}'!${}:${}",
                        ws.name,
                        col_to_letter(fc),
                        col_to_letter(lc)
                    );
                    all_names.push(("_xlnm.Print_Titles".into(), val, Some(i)));
                }
            }
        }

        if !all_names.is_empty() {
            w.start_tag("definedNames", &[]);
            for (name, formula, local_sheet) in &all_names {
                if let Some(idx) = local_sheet {
                    let ids = idx.to_string();
                    w.text_element(
                        "definedName",
                        &[("name", name.as_str()), ("localSheetId", &ids)],
                        formula,
                    );
                } else {
                    w.text_element("definedName", &[("name", name.as_str())], formula);
                }
            }
            w.end_tag("definedNames");
        }

        if let Some(mode) = &self.calc_mode {
            let val = match mode {
                CalcMode::Auto => "auto",
                CalcMode::Manual => "manual",
                CalcMode::AutoNoTable => "autoNoTable",
            };
            w.empty_tag("calcPr", &[("calcMode", val), ("fullCalcOnLoad", "1")]);
        } else {
            w.empty_tag("calcPr", &[("calcId", "0"), ("fullCalcOnLoad", "1")]);
        }

        // pivotCaches — links cacheId to workbook rel
        let mut pivot_count = 0usize;
        for ws in &self.worksheets {
            pivot_count += ws.pivot_tables.len();
        }
        if pivot_count > 0 {
            let sheet_count = self.worksheets.len();
            let chartsheet_count = self.chart_sheets.len();
            let has_vba = self
                .passthrough_entries
                .iter()
                .any(|(n, _)| n.eq_ignore_ascii_case("xl/vbaProject.bin"));
            w.start_tag("pivotCaches", &[]);
            for i in 0..pivot_count {
                let cache_id =
                    (sheet_count + chartsheet_count + 3 + if has_vba { 1 } else { 0 } + i + 1)
                        .to_string();
                let rid = format!("rId{cache_id}");
                w.empty_tag("pivotCache", &[("cacheId", &cache_id), ("r:id", &rid)]);
            }
            w.end_tag("pivotCaches");
        }

        w.end_tag("workbook");
        w.into_bytes()
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn write_content_types_full(
    sheet_count: usize,
    has_props: bool,
    chart_count: usize,
    table_count: usize,
    sheets_with_drawings: &[usize],
    image_extensions: &[String],
    has_vba: bool,
    is_xlsm: bool,
    sheets_with_comments: &[usize],
    pivot_count: usize,
    chartex_count: usize,
    chart_sheets: &[super::ChartSheet],
    slicer_sheet_count: usize,
    slicer_cache_count: usize,
    timeline_sheet_count: usize,
    timeline_cache_count: usize,
    wb_content_type: super::WorkbookContentType,
    sheets_with_threaded_comments: &[usize],
    has_persons: bool,
    custom_xml_count: usize,
    has_custom_props: bool,
) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "Types",
        &[(
            "xmlns",
            "http://schemas.openxmlformats.org/package/2006/content-types",
        )],
    );
    w.empty_tag(
        "Default",
        &[
            ("Extension", "rels"),
            (
                "ContentType",
                "application/vnd.openxmlformats-package.relationships+xml",
            ),
        ],
    );
    w.empty_tag(
        "Default",
        &[("Extension", "xml"), ("ContentType", "application/xml")],
    );
    if has_vba {
        w.empty_tag(
            "Default",
            &[
                ("Extension", "bin"),
                ("ContentType", "application/vnd.ms-office.vbaProject"),
            ],
        );
    }

    let mut seen_ext = std::collections::HashSet::new();
    for ext in image_extensions {
        if seen_ext.insert(ext.as_str()) {
            let ct = if ext == "png" {
                "image/png"
            } else {
                "image/jpeg"
            };
            w.empty_tag("Default", &[("Extension", ext), ("ContentType", ct)]);
        }
    }

    let wb_ct = match wb_content_type {
        super::WorkbookContentType::Xlsm => "application/vnd.ms-excel.sheet.macroEnabled.main+xml",
        super::WorkbookContentType::Template => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.template.main+xml"
        }
        super::WorkbookContentType::TemplateMacro => {
            "application/vnd.ms-excel.template.macroEnabled.main+xml"
        }
        super::WorkbookContentType::Xlsx => {
            if is_xlsm || has_vba {
                "application/vnd.ms-excel.sheet.macroEnabled.main+xml"
            } else {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"
            }
        }
    };
    w.empty_tag(
        "Override",
        &[("PartName", "/xl/workbook.xml"), ("ContentType", wb_ct)],
    );
    for i in 1..=sheet_count {
        let part = format!("/xl/worksheets/sheet{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml",
                ),
            ],
        );
    }
    w.empty_tag(
        "Override",
        &[
            ("PartName", "/xl/styles.xml"),
            (
                "ContentType",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml",
            ),
        ],
    );
    w.empty_tag(
        "Override",
        &[
            ("PartName", "/xl/theme/theme1.xml"),
            (
                "ContentType",
                "application/vnd.openxmlformats-officedocument.theme+xml",
            ),
        ],
    );
    w.empty_tag(
        "Override",
        &[
            ("PartName", "/xl/sharedStrings.xml"),
            (
                "ContentType",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml",
            ),
        ],
    );

    for &si in sheets_with_drawings {
        let part = format!("/xl/drawings/drawing{}.xml", si + 1);
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.drawing+xml",
                ),
            ],
        );
    }
    for i in 1..=chart_count {
        let part = format!("/xl/charts/chart{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.drawingml.chart+xml",
                ),
            ],
        );
    }
    for i in 1..=table_count {
        let part = format!("/xl/tables/table{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.table+xml",
                ),
            ],
        );
    }
    for &si in sheets_with_comments {
        let part = format!("/xl/comments{}.xml", si + 1);
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.comments+xml",
                ),
            ],
        );
    }
    if !sheets_with_comments.is_empty() {
        w.empty_tag(
            "Default",
            &[
                ("Extension", "vml"),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.vmlDrawing",
                ),
            ],
        );
    }
    if has_props {
        w.empty_tag(
            "Override",
            &[
                ("PartName", "/docProps/core.xml"),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-package.core-properties+xml",
                ),
            ],
        );
        w.empty_tag(
            "Override",
            &[
                ("PartName", "/docProps/app.xml"),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.extended-properties+xml",
                ),
            ],
        );
    }
    for i in 1..=pivot_count {
        let pt = format!("/xl/pivotTables/pivotTable{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &pt),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.pivotTable+xml",
                ),
            ],
        );
        let cd = format!("/xl/pivotCache/pivotCacheDefinition{i}.xml");
        w.empty_tag("Override", &[("PartName", &cd), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.pivotCacheDefinition+xml")]);
        let cr = format!("/xl/pivotCache/pivotCacheRecords{i}.xml");
        w.empty_tag("Override", &[("PartName", &cr), ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.pivotCacheRecords+xml")]);
    }
    for i in 1..=chartex_count {
        let ce = format!("/xl/charts/chartEx{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &ce),
                ("ContentType", "application/vnd.ms-office.chartEx+xml"),
            ],
        );
        let st = format!("/xl/charts/style{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &st),
                ("ContentType", "application/vnd.ms-office.chartstyle+xml"),
            ],
        );
        let co = format!("/xl/charts/colors{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &co),
                (
                    "ContentType",
                    "application/vnd.ms-office.chartcolorstyle+xml",
                ),
            ],
        );
    }
    // Slicer parts
    for i in 1..=slicer_sheet_count {
        let part = format!("/xl/slicers/slicer{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                ("ContentType", "application/vnd.ms-excel.slicer+xml"),
            ],
        );
    }
    for i in 1..=slicer_cache_count {
        let part = format!("/xl/slicerCaches/slicerCache{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                ("ContentType", "application/vnd.ms-excel.slicerCache+xml"),
            ],
        );
    }
    // Timeline parts
    for i in 1..=timeline_sheet_count {
        let part = format!("/xl/timelines/timeline{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                ("ContentType", "application/vnd.ms-excel.timeline+xml"),
            ],
        );
    }
    for i in 1..=timeline_cache_count {
        let part = format!("/xl/timelineCaches/timelineCache{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                ("ContentType", "application/vnd.ms-excel.timelineCache+xml"),
            ],
        );
    }
    // Chart sheets
    for (i, _cs) in chart_sheets.iter().enumerate() {
        let cs_idx = i + 1;
        let part = format!("/xl/chartsheets/sheet{cs_idx}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.chartsheet+xml",
                ),
            ],
        );
        // Drawing for chart sheet
        let drawing_idx = sheets_with_drawings.len() + cs_idx;
        let drawing_part = format!("/xl/drawings/drawing{drawing_idx}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &drawing_part),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.drawing+xml",
                ),
            ],
        );
        // Chart for chart sheet
        let chart_idx = chart_count + cs_idx;
        let chart_part = format!("/xl/charts/chart{chart_idx}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &chart_part),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.drawingml.chart+xml",
                ),
            ],
        );
    }
    // Threaded comments
    for &si in sheets_with_threaded_comments {
        let part = format!("/xl/threadedComments/threadedComment{}.xml", si + 1);
        w.empty_tag(
            "Override",
            &[
                ("PartName", &part),
                (
                    "ContentType",
                    "application/vnd.ms-excel.threadedcomments+xml",
                ),
            ],
        );
    }
    if has_persons {
        w.empty_tag(
            "Override",
            &[
                ("PartName", "/xl/persons/person.xml"),
                ("ContentType", "application/vnd.ms-excel.person+xml"),
            ],
        );
    }
    // Custom XML parts
    for i in 1..=custom_xml_count {
        let part = format!("/customXml/item{i}.xml");
        w.empty_tag(
            "Override",
            &[("PartName", &part), ("ContentType", "application/xml")],
        );
        let props_part = format!("/customXml/itemProps{i}.xml");
        w.empty_tag(
            "Override",
            &[
                ("PartName", &props_part),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.customXmlProperties+xml",
                ),
            ],
        );
    }
    // Custom document properties
    if has_custom_props {
        w.empty_tag(
            "Override",
            &[
                ("PartName", "/docProps/custom.xml"),
                (
                    "ContentType",
                    "application/vnd.openxmlformats-officedocument.custom-properties+xml",
                ),
            ],
        );
    }
    w.end_tag("Types");
    w.into_bytes()
}

pub(crate) fn write_root_rels(has_props: bool) -> Vec<u8> {
    // Note: custom properties relationship is added via passthrough or content types.
    // The root rels only needs the core relationships.
    let mut rels = vec![(
        "rId1",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
        "xl/workbook.xml",
    )];
    if has_props {
        rels.push((
            "rId2",
            "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties",
            "docProps/core.xml",
        ));
        rels.push(("rId3", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties", "docProps/app.xml"));
    }
    rel_writer::write_rels(&rels)
}
