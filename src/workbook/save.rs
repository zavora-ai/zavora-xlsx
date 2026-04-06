// save_to_buffer — the main serialization pipeline

use std::collections::BTreeMap;
use crate::properties;
use crate::writer::sheet_writer::{self, SheetCells};
use crate::writer::{chart_writer, comment_writer, drawing_writer, rel_writer, sst_writer, style_writer, table_writer};
use crate::zip::zip_writer::ZipOutput;
use super::{Workbook, write_content_types_full, write_root_rels};

impl Workbook {
    /// Save the workbook to an in-memory buffer.
    pub fn save_to_buffer(&mut self) -> crate::Result<Vec<u8>> {
        for ws in &mut self.worksheets {
            if ws.raw_xml.is_some() && ws.read_cells.is_none() && !ws.dirty { continue; }
            if ws.raw_xml.is_some() && ws.dirty { ws.raw_xml = None; }
            ws.finalize(&mut self.sst, &mut self.styles);
        }

        for ws in &mut self.worksheets {
            for cf in &mut ws.conditional_formats {
                if let Some(fmt) = cf.rule.dxf_format() {
                    let fmt_clone = fmt.clone();
                    cf.dxf_id = Some(self.styles.register_dxf(&fmt_clone));
                }
            }
        }

        // Mark pivot tables that have associated pivot charts
        for ws in &mut self.worksheets {
            let chart_pivot_names: Vec<String> = ws.charts.iter()
                .filter_map(|c| c.pivot_source.as_ref().map(|ps| ps.pivot_table_name.clone()))
                .collect();
            for pt in &mut ws.pivot_tables {
                if chart_pivot_names.iter().any(|n| n == &pt.name) {
                    pt.has_chart = true;
                }
            }
        }

        let mut col_format_xfs: Vec<BTreeMap<u16, u32>> = Vec::new();
        let mut row_format_xfs: Vec<BTreeMap<u32, u32>> = Vec::new();
        for ws in &self.worksheets {
            let mut cf_map = BTreeMap::new();
            for (col, fmt) in &ws.col_formats { cf_map.insert(*col, self.styles.register_format(fmt)); }
            col_format_xfs.push(cf_map);
            let mut rf_map = BTreeMap::new();
            for (row, fmt) in &ws.row_formats { rf_map.insert(*row, self.styles.register_format(fmt)); }
            row_format_xfs.push(rf_map);
        }

        let mut zip = ZipOutput::new();
        let sheet_count = self.worksheets.len();
        let has_props = self.properties.title.is_some() || self.properties.author.is_some()
            || self.properties.subject.is_some() || self.properties.company.is_some();

        let mut total_charts = 0usize;
        let mut total_tables = 0usize;
        let mut sheets_with_drawings = Vec::new();
        let mut sheets_with_comments = Vec::new();
        let mut image_extensions: Vec<String> = Vec::new();

        for (i, ws) in self.worksheets.iter().enumerate() {
            let has_drawing = !ws.charts.is_empty() || !ws.images.is_empty() || ws.original_drawing_rid.is_some();
            if has_drawing { sheets_with_drawings.push(i); }
            if !ws.comments.is_empty() || ws.original_legacy_drawing_rid.is_some() { sheets_with_comments.push(i); }
            total_charts += ws.charts.len();
            total_tables += ws.tables.len();
            for img in &ws.images { image_extensions.push(img.image_type.extension().to_string()); }
        }
        for (name, _) in &self.passthrough_entries {
            if name.starts_with("xl/charts/chart") && name.ends_with(".xml") { total_charts += 1; }
        }

        // Count pivot tables
        let mut total_pivots = 0usize;
        let mut sheets_with_pivots: Vec<(usize, usize)> = Vec::new(); // (sheet_idx, pivot_count)
        for (i, ws) in self.worksheets.iter().enumerate() {
            if !ws.pivot_tables.is_empty() {
                sheets_with_pivots.push((i, ws.pivot_tables.len()));
                total_pivots += ws.pivot_tables.len();
            }
        }

        let has_vba = self.passthrough_entries.iter().any(|(n, _)| n.eq_ignore_ascii_case("xl/vbaProject.bin"));

        zip.add_file("[Content_Types].xml", &write_content_types_full(
            sheet_count, has_props, total_charts, total_tables, &sheets_with_drawings, &image_extensions, has_vba, self.is_xlsm, &sheets_with_comments, total_pivots,
        ))?;
        zip.add_file("_rels/.rels", &write_root_rels(has_props))?;
        zip.add_file("xl/_rels/workbook.xml.rels", &rel_writer::write_workbook_rels(sheet_count, has_vba, total_pivots))?;
        zip.add_file("xl/workbook.xml", &self.write_workbook_xml())?;

        // Pre-compute per-sheet metadata
        struct SheetMeta {
            drawing_rid: Option<String>,
            table_rids: Vec<String>,
            sheet_rels: Vec<(String, String, String)>,
            legacy_drawing_rid: Option<String>,
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
            } else if ws.original_drawing_rid.is_some() {
                ws.original_drawing_rid.clone()
            } else { None };

            let mut table_rids = Vec::new();
            for t_idx in 0..ws.tables.len() {
                let rid = format!("rId{next_rid}");
                let tid = global_table_idx + t_idx + 1;
                sheet_rels.push((rid.clone(), "http://schemas.openxmlformats.org/officeDocument/2006/relationships/table".into(), format!("../tables/table{tid}.xml")));
                table_rids.push(rid);
                next_rid += 1;
            }

            for (hi, hl) in ws.hyperlinks.iter().enumerate() {
                if hl.location.is_none() && !hl.url.is_empty() {
                    let rid = format!("rId_hl{}", hi + 1);
                    sheet_rels.push((rid, "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink".into(), hl.url.clone()));
                }
            }

            let mut legacy_drawing_rid = None;
            if !ws.comments.is_empty() {
                let rid = format!("rId{next_rid}");
                sheet_rels.push((rid, "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments".into(), format!("../comments{}.xml", i + 1)));
                next_rid += 1;
                let rid2 = format!("rId{next_rid}");
                sheet_rels.push((rid2.clone(), "http://schemas.openxmlformats.org/officeDocument/2006/relationships/vmlDrawing".into(), format!("../drawings/vmlDrawing{}.vml", i + 1)));
                legacy_drawing_rid = Some(rid2);
                let _ = next_rid;
            } else if ws.original_legacy_drawing_rid.is_some() {
                legacy_drawing_rid = ws.original_legacy_drawing_rid.clone();
            }

            // Pivot table rels
            for (pi, _pt) in ws.pivot_tables.iter().enumerate() {
                let rid = format!("rId{next_rid}");
                let pt_idx = pi + 1;
                sheet_rels.push((rid, "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotTable".into(), format!("../pivotTables/pivotTable{pt_idx}.xml")));
                next_rid += 1;
            }

            metas.push(SheetMeta {
                drawing_rid, table_rids, sheet_rels, legacy_drawing_rid,
                global_chart_start: global_chart_idx, global_image_start: global_image_idx, global_table_start: global_table_idx,
            });
            global_chart_idx += ws.charts.len();
            global_image_idx += ws.images.len();
            global_table_idx += ws.tables.len();
        }

        // Parallel sheet XML generation
        let sheet_xmls: Vec<Option<Vec<u8>>> = std::thread::scope(|s| {
            let handles: Vec<_> = self.worksheets.iter().zip(metas.iter()).enumerate().map(|(idx, (ws, meta))| {
                let active = self.active_sheet.map_or(idx == 0, |a| a == idx);
                let cf_xfs = &col_format_xfs[idx];
                let rf_xfs = &row_format_xfs[idx];
                s.spawn(move || {
                    if ws.raw_xml.is_some() && !ws.dirty { return None; }
                    let sc = SheetCells {
                        cells: &ws.cells, merge_ranges: &ws.merge_ranges,
                        col_widths: &ws.col_widths, row_heights: &ws.row_heights,
                        freeze_row: ws.freeze_row, freeze_col: ws.freeze_col,
                        drawing_rid: meta.drawing_rid.clone(), table_parts: meta.table_rids.clone(),
                        conditional_formats: &ws.conditional_formats,
                        validations: &ws.validations, sparklines: &ws.sparklines,
                        protection: ws.protection.as_ref(),
                        print_settings: ws.print_settings.as_ref(),
                        hidden_rows: &ws.hidden_rows, hidden_cols: &ws.hidden_cols,
                        autofilter: ws.autofilter,
                        hyperlinks: &ws.hyperlinks, hyperlink_rels: &[],
                        row_outline_levels: &ws.row_outline_levels,
                        col_outline_levels: &ws.col_outline_levels,
                        legacy_drawing_rid: meta.legacy_drawing_rid.clone(),
                        zoom: ws.zoom, show_gridlines: ws.show_gridlines,
                        show_headings: ws.show_headings, right_to_left: ws.right_to_left,
                        tab_color: ws.tab_color,
                        is_active: active, selection: ws.selection, top_left_cell: ws.top_left_cell,
                        default_row_height: ws.default_row_height,
                        col_formats: cf_xfs, row_formats: rf_xfs,
                        ignored_errors: &ws.ignored_errors, autofilter_columns: &ws.autofilter_columns,
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
                None => { if let Some(ref raw) = ws.raw_xml { zip.add_file(&sheet_path, raw)?; } }
                Some(xml) => { zip.add_file(&sheet_path, xml)?; }
            }

            if !meta.sheet_rels.is_empty() {
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", i + 1);
                let refs: Vec<(&str, &str, &str)> = meta.sheet_rels.iter().map(|(a, b, c)| (a.as_str(), b.as_str(), c.as_str())).collect();
                zip.add_file(&rels_path, &rel_writer::write_rels(&refs))?;
            } else if ws.dirty && ws.original_rels.is_some() {
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", i + 1);
                zip.add_file(&rels_path, ws.original_rels.as_ref().unwrap())?;
            }

            if !ws.charts.is_empty() || !ws.images.is_empty() {
                let drawing_path = format!("xl/drawings/drawing{}.xml", i + 1);
                zip.add_file(&drawing_path, &drawing_writer::write_drawing_xml(&ws.charts, &ws.images, i))?;
                let img_types: Vec<&str> = ws.images.iter().map(|img| img.image_type.extension()).collect();
                let drawing_rels_path = format!("xl/drawings/_rels/drawing{}.xml.rels", i + 1);
                zip.add_file(&drawing_rels_path, &drawing_writer::write_drawing_rels(ws.charts.len(), ws.images.len(), &img_types, meta.global_chart_start, meta.global_image_start))?;
            }

            for (ci, chart) in ws.charts.iter().enumerate() {
                let idx = meta.global_chart_start + ci + 1;
                zip.add_file(&format!("xl/charts/chart{idx}.xml"), &chart_writer::write_chart_xml(chart, idx))?;
            }
            for (ii, img) in ws.images.iter().enumerate() {
                let idx = meta.global_image_start + ii + 1;
                zip.add_file(&format!("xl/media/image{}.{}", idx, img.image_type.extension()), &img.data)?;
            }
            for (ti, table) in ws.tables.iter().enumerate() {
                let idx = meta.global_table_start + ti + 1;
                zip.add_file(&format!("xl/tables/table{idx}.xml"), &table_writer::write_table_xml(table, idx))?;
            }
            if !ws.comments.is_empty() {
                zip.add_file(&format!("xl/comments{}.xml", i + 1), &comment_writer::write_comments_xml(&ws.comments))?;
                zip.add_file(&format!("xl/drawings/vmlDrawing{}.vml", i + 1), &comment_writer::write_vml_drawing(&ws.comments))?;
            }

            // Pivot tables
            for (pi, pt) in ws.pivot_tables.iter().enumerate() {
                let pt_idx = pi + 1; // per-sheet for now, global handled below
                // Parse source range to get sheet name and cell range
                let (src_sheet, src_ref) = parse_pivot_source(&pt.source_range);
                // Find source worksheet and scan data
                let src_ws = self.worksheets.iter().find(|w| w.name == src_sheet);
                if let Some(src) = src_ws {
                    if let Some((r1, c1, r2, c2)) = crate::utility::parse_range(&src_ref.replace('$', "")) {
                        let cache = crate::writer::pivot_writer::scan_source_data(&src.cells, &self.sst, r1, c1, r2, c2);
                        // cacheId must match the rId in workbook rels
                        // workbook rels: sheets(N) + styles + sharedStrings + theme + [vba] + pivotCache
                        let cache_rid = sheet_count + 3 + if has_vba { 1 } else { 0 } + pt_idx;
                        let cache_id = cache_rid;
                        zip.add_file(&format!("xl/pivotCache/pivotCacheDefinition{pt_idx}.xml"),
                            &crate::writer::pivot_writer::write_cache_definition(&cache, &src_ref, &src_sheet, cache_id))?;
                        zip.add_file(&format!("xl/pivotCache/pivotCacheRecords{pt_idx}.xml"),
                            &crate::writer::pivot_writer::write_cache_records(&cache))?;
                        zip.add_file(&format!("xl/pivotTables/pivotTable{pt_idx}.xml"),
                            &crate::writer::pivot_writer::write_pivot_table(pt, &cache, cache_id))?;
                        // Pivot table rels → cache definition
                        zip.add_file(&format!("xl/pivotTables/_rels/pivotTable{pt_idx}.xml.rels"),
                            &rel_writer::write_rels(&[("rId1", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheDefinition", &format!("../pivotCache/pivotCacheDefinition{pt_idx}.xml"))]))?;
                        // Cache definition rels → cache records
                        zip.add_file(&format!("xl/pivotCache/_rels/pivotCacheDefinition{pt_idx}.xml.rels"),
                            &rel_writer::write_rels(&[("rId1", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheRecords", &format!("pivotCacheRecords{pt_idx}.xml"))]))?;
                    }
                }
            }
        }

        zip.add_file("xl/styles.xml", &style_writer::write_styles(&self.styles))?;
        zip.add_file("xl/sharedStrings.xml", &sst_writer::write_sst(&self.sst))?;
        zip.add_file("xl/theme/theme1.xml", &crate::writer::theme_writer::write_theme())?;

        if has_props {
            zip.add_file("docProps/core.xml", &properties::write_core_xml(&self.properties))?;
            zip.add_file("docProps/app.xml", &properties::write_app_xml(&self.properties))?;
        }

        for (name, data) in &self.passthrough_entries { zip.add_file(name, data)?; }

        zip.finish()
    }
}

fn parse_pivot_source(source_range: &str) -> (String, String) {
    // "Sheet1!$A$1:$E$100" → ("Sheet1", "$A$1:$E$100")
    if let Some(pos) = source_range.find('!') {
        let sheet = source_range[..pos].trim_matches('\'').to_string();
        let range = source_range[pos + 1..].to_string();
        (sheet, range)
    } else {
        ("Sheet1".into(), source_range.into())
    }
}
