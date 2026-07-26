// save_to_buffer — the main serialization pipeline

use super::{Workbook, write_content_types_full, write_root_rels};
use crate::properties;
use crate::writer::sheet_writer::{self, SheetCells};
use crate::writer::{
    chart_writer, comment_writer, drawing_writer, rel_writer, slicer_writer, sst_writer,
    style_writer, table_writer, timeline_writer,
};
use crate::zip::zip_writer::ZipOutput;
use std::collections::BTreeMap;

impl Workbook {
    /// Save the workbook to an in-memory buffer.
    pub fn save_to_buffer(&mut self) -> crate::Result<Vec<u8>> {
        let ct = if self.is_xlsm {
            super::WorkbookContentType::Xlsm
        } else {
            super::WorkbookContentType::Xlsx
        };
        self.save_to_buffer_with_content_type(ct)
    }

    /// Save the workbook to an in-memory buffer with a specific content type.
    pub(crate) fn save_to_buffer_with_content_type(
        &mut self,
        wb_content_type: super::WorkbookContentType,
    ) -> crate::Result<Vec<u8>> {
        for ws in &mut self.worksheets {
            if ws.raw_xml.is_some() && ws.read_cells.is_none() && !ws.dirty {
                continue;
            }
            if ws.raw_xml.is_some() && ws.dirty {
                ws.raw_xml = None;
            }
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

        // Mark pivot tables that have associated pivot charts + pre-compute chart series
        // First pass: collect pivot cache data per pivot table name
        let mut pivot_caches: std::collections::HashMap<
            String,
            crate::writer::pivot_writer::PivotCacheData,
        > = std::collections::HashMap::new();
        for ws in &self.worksheets {
            for pt in &ws.pivot_tables {
                let (src_sheet, src_ref) = parse_pivot_source(&pt.source_range);
                let src_ws = self.worksheets.iter().find(|w| w.name == src_sheet);
                if let Some(src) = src_ws
                    && let Some((r1, c1, r2, c2)) =
                        crate::utility::parse_range(&src_ref.replace('$', ""))
                {
                    let cache = crate::writer::pivot_writer::scan_source_data(
                        &src.cells, &self.sst, r1, c1, r2, c2,
                    );
                    pivot_caches.insert(pt.name.clone(), cache);
                }
            }
        }
        // Second pass: mark has_chart and attach pre-computed series to charts
        for ws in &mut self.worksheets {
            let chart_pivot_names: Vec<String> = ws
                .charts
                .iter()
                .filter_map(|c| {
                    c.pivot_source
                        .as_ref()
                        .map(|ps| ps.pivot_table_name.clone())
                })
                .collect();
            for pt in &mut ws.pivot_tables {
                if chart_pivot_names.iter().any(|n| n == &pt.name) {
                    pt.has_chart = true;
                }
            }
            for chart in &mut ws.charts {
                if let Some(ref ps) = chart.pivot_source
                    && let Some(pt) = ws
                        .pivot_tables
                        .iter()
                        .find(|p| p.name == ps.pivot_table_name)
                    && let Some(cache) = pivot_caches.get(&pt.name)
                {
                    chart.pivot_series =
                        crate::writer::pivot_writer::compute_pivot_chart_series(pt, cache);
                }
            }
        }

        let mut col_format_xfs: Vec<BTreeMap<u16, u32>> = Vec::new();
        let mut row_format_xfs: Vec<BTreeMap<u32, u32>> = Vec::new();
        for ws in &self.worksheets {
            let mut cf_map = BTreeMap::new();
            for (col, fmt) in &ws.col_formats {
                cf_map.insert(*col, self.styles.register_format(fmt));
            }
            col_format_xfs.push(cf_map);
            let mut rf_map = BTreeMap::new();
            for (row, fmt) in &ws.row_formats {
                rf_map.insert(*row, self.styles.register_format(fmt));
            }
            row_format_xfs.push(rf_map);
        }

        let mut zip = ZipOutput::new();
        let sheet_count = self.worksheets.len();
        let has_props = self.properties.title.is_some()
            || self.properties.author.is_some()
            || self.properties.subject.is_some()
            || self.properties.company.is_some();

        let mut total_charts = 0usize;
        let mut total_tables = 0usize;
        let mut sheets_with_drawings = Vec::new();
        let mut sheets_with_comments = Vec::new();
        let mut image_extensions: Vec<String> = Vec::new();

        for (i, ws) in self.worksheets.iter().enumerate() {
            let has_drawing = !ws.charts.is_empty()
                || !ws.images.is_empty()
                || !ws.treemap_charts.is_empty()
                || !ws.chartex_charts.is_empty()
                || !ws.shapes.is_empty()
                || ws.original_drawing_rid.is_some();
            if has_drawing {
                sheets_with_drawings.push(i);
            }
            if !ws.comments.is_empty()
                || !ws.form_controls.is_empty()
                || ws.original_legacy_drawing_rid.is_some()
            {
                sheets_with_comments.push(i);
            }
            total_charts += ws.charts.len();
            total_tables += ws.tables.len();
            for img in &ws.images {
                image_extensions.push(img.image_type.extension().to_string());
            }
        }
        for (name, _) in &self.passthrough_entries {
            if name.starts_with("xl/charts/chart") && name.ends_with(".xml") {
                total_charts += 1;
            }
        }

        // Count treemap (chartEx) charts
        let mut total_chartex = 0usize;
        for ws in &self.worksheets {
            total_chartex += ws.treemap_charts.len() + ws.chartex_charts.len();
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

        let has_vba = self
            .passthrough_entries
            .iter()
            .any(|(n, _)| n.eq_ignore_ascii_case("xl/vbaProject.bin"));

        // Count slicers
        let mut total_slicer_sheets = 0usize;
        let mut total_slicer_caches = 0usize;
        for ws in &self.worksheets {
            if !ws.slicers.is_empty() {
                total_slicer_sheets += 1;
                total_slicer_caches += ws.slicers.len();
            }
        }

        // Count timelines
        let mut total_timeline_sheets = 0usize;
        let mut total_timeline_caches = 0usize;
        for ws in &self.worksheets {
            if !ws.timelines.is_empty() {
                total_timeline_sheets += 1;
                total_timeline_caches += ws.timelines.len();
            }
        }

        // Count threaded comments, custom XML, custom properties
        let mut sheets_with_threaded_comments: Vec<usize> = Vec::new();
        for (i, ws) in self.worksheets.iter().enumerate() {
            if !ws.threaded_comments.is_empty() {
                sheets_with_threaded_comments.push(i);
            }
        }
        let has_persons = !sheets_with_threaded_comments.is_empty();
        let custom_xml_count = self.custom_xml_parts.len();
        let has_custom_props = !self.custom_properties.is_empty();

        zip.add_file(
            "[Content_Types].xml",
            &write_content_types_full(
                sheet_count,
                has_props,
                total_charts,
                total_tables,
                &sheets_with_drawings,
                &image_extensions,
                has_vba,
                self.is_xlsm,
                &sheets_with_comments,
                total_pivots,
                total_chartex,
                &self.chart_sheets,
                total_slicer_sheets,
                total_slicer_caches,
                total_timeline_sheets,
                total_timeline_caches,
                wb_content_type,
                &sheets_with_threaded_comments,
                has_persons,
                custom_xml_count,
                has_custom_props,
            ),
        )?;
        zip.add_file("_rels/.rels", &write_root_rels(has_props))?;
        zip.add_file(
            "xl/_rels/workbook.xml.rels",
            &rel_writer::write_workbook_rels_with_chartsheets(
                sheet_count,
                self.chart_sheets.len(),
                has_vba,
                total_pivots,
            ),
        )?;
        zip.add_file("xl/workbook.xml", &self.write_workbook_xml())?;

        // Pre-compute per-sheet metadata
        struct SheetMeta {
            drawing_rid: Option<String>,
            table_rids: Vec<String>,
            sheet_rels: Vec<(String, String, String)>,
            legacy_drawing_rid: Option<String>,
            global_chart_start: usize,
            global_chartex_start: usize,
            global_image_start: usize,
            global_table_start: usize,
        }

        let mut metas = Vec::with_capacity(sheet_count);
        let mut global_chart_idx = 0usize;
        let mut global_chartex_idx = 0usize;
        let mut global_image_idx = 0usize;
        let mut global_table_idx = 0usize;

        for (i, ws) in self.worksheets.iter().enumerate() {
            let has_drawing = !ws.charts.is_empty()
                || !ws.images.is_empty()
                || !ws.treemap_charts.is_empty()
                || !ws.chartex_charts.is_empty()
                || !ws.shapes.is_empty();
            let mut sheet_rels: Vec<(String, String, String)> = Vec::new();
            let mut next_rid = 1;

            let drawing_rid = if has_drawing {
                let rid = format!("rId{next_rid}");
                sheet_rels.push((
                    rid.clone(),
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing"
                        .into(),
                    format!("../drawings/drawing{}.xml", i + 1),
                ));
                next_rid += 1;
                Some(rid)
            } else if ws.original_drawing_rid.is_some() {
                ws.original_drawing_rid.clone()
            } else {
                None
            };

            let mut table_rids = Vec::new();
            for t_idx in 0..ws.tables.len() {
                let rid = format!("rId{next_rid}");
                let tid = global_table_idx + t_idx + 1;
                sheet_rels.push((
                    rid.clone(),
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/table"
                        .into(),
                    format!("../tables/table{tid}.xml"),
                ));
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
            if !ws.comments.is_empty() || !ws.form_controls.is_empty() {
                let rid = format!("rId{next_rid}");
                sheet_rels.push((
                    rid,
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments"
                        .into(),
                    format!("../comments{}.xml", i + 1),
                ));
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

            // Slicer rels (extLst in sheet XML references the slicer part)
            // We don't add a direct relationship here; slicers are referenced via extLst.
            // But we do need a drawing relationship for the slicer visual.
            let _ = next_rid;

            metas.push(SheetMeta {
                drawing_rid,
                table_rids,
                sheet_rels,
                legacy_drawing_rid,
                global_chart_start: global_chart_idx,
                global_chartex_start: global_chartex_idx,
                global_image_start: global_image_idx,
                global_table_start: global_table_idx,
            });
            global_chart_idx += ws.charts.len();
            global_chartex_idx += ws.treemap_charts.len() + ws.chartex_charts.len();
            global_image_idx += ws.images.len();
            global_table_idx += ws.tables.len();
        }

        // Parallel sheet XML generation
        let sheet_xmls: Vec<Option<Vec<u8>>> = std::thread::scope(|s| {
            let handles: Vec<_> = self
                .worksheets
                .iter()
                .zip(metas.iter())
                .enumerate()
                .map(|(idx, (ws, meta))| {
                    let active = self.active_sheet.map_or(idx == 0, |a| a == idx);
                    let cf_xfs = &col_format_xfs[idx];
                    let rf_xfs = &row_format_xfs[idx];
                    s.spawn(move || {
                        if ws.raw_xml.is_some() && !ws.dirty {
                            return None;
                        }
                        let sc = SheetCells {
                            cells: &ws.cells,
                            merge_ranges: &ws.merge_ranges,
                            col_widths: &ws.col_widths,
                            row_heights: &ws.row_heights,
                            freeze_row: ws.freeze_row,
                            freeze_col: ws.freeze_col,
                            drawing_rid: meta.drawing_rid.clone(),
                            table_parts: meta.table_rids.clone(),
                            conditional_formats: &ws.conditional_formats,
                            validations: &ws.validations,
                            sparklines: &ws.sparklines,
                            protection: ws.protection.as_ref(),
                            print_settings: ws.print_settings.as_ref(),
                            hidden_rows: &ws.hidden_rows,
                            hidden_cols: &ws.hidden_cols,
                            autofilter: ws.autofilter,
                            hyperlinks: &ws.hyperlinks,
                            hyperlink_rels: &[],
                            row_outline_levels: &ws.row_outline_levels,
                            col_outline_levels: &ws.col_outline_levels,
                            legacy_drawing_rid: meta.legacy_drawing_rid.clone(),
                            zoom: ws.zoom,
                            show_gridlines: ws.show_gridlines,
                            show_headings: ws.show_headings,
                            right_to_left: ws.right_to_left,
                            tab_color: ws.tab_color,
                            is_active: active,
                            selection: ws.selection,
                            top_left_cell: ws.top_left_cell,
                            default_row_height: ws.default_row_height,
                            col_formats: cf_xfs,
                            row_formats: rf_xfs,
                            ignored_errors: &ws.ignored_errors,
                            autofilter_columns: &ws.autofilter_columns,
                            advanced_filter_columns: &ws.advanced_filter_columns,
                            sort_state: ws.sort_state.as_ref(),
                            phonetic_runs: &ws.phonetic_runs,
                        };
                        Some(sheet_writer::write_sheet(&sc))
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        // Sequential zip write
        for (i, (ws, meta)) in self.worksheets.iter().zip(metas.iter()).enumerate() {
            let sheet_path = format!("xl/worksheets/sheet{}.xml", i + 1);
            match &sheet_xmls[i] {
                None => {
                    if let Some(ref raw) = ws.raw_xml {
                        zip.add_file(&sheet_path, raw)?;
                    }
                }
                Some(xml) => {
                    zip.add_file(&sheet_path, xml)?;
                }
            }

            if !meta.sheet_rels.is_empty() {
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", i + 1);
                let refs: Vec<(&str, &str, &str)> = meta
                    .sheet_rels
                    .iter()
                    .map(|(a, b, c)| (a.as_str(), b.as_str(), c.as_str()))
                    .collect();
                zip.add_file(&rels_path, &rel_writer::write_rels(&refs))?;
            } else if ws.dirty && ws.original_rels.is_some() {
                let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", i + 1);
                zip.add_file(&rels_path, ws.original_rels.as_ref().unwrap())?;
            }

            if !ws.charts.is_empty()
                || !ws.images.is_empty()
                || !ws.treemap_charts.is_empty()
                || !ws.chartex_charts.is_empty()
                || !ws.shapes.is_empty()
            {
                let drawing_path = format!("xl/drawings/drawing{}.xml", i + 1);
                zip.add_file(
                    &drawing_path,
                    &drawing_writer::write_drawing_xml_with_shapes(
                        &ws.charts,
                        &ws.treemap_charts,
                        &ws.chartex_charts,
                        &ws.images,
                        &ws.shapes,
                        i,
                    ),
                )?;
                let img_types: Vec<&str> = ws
                    .images
                    .iter()
                    .map(|img| img.image_type.extension())
                    .collect();
                let drawing_rels_path = format!("xl/drawings/_rels/drawing{}.xml.rels", i + 1);
                zip.add_file(
                    &drawing_rels_path,
                    &drawing_writer::write_drawing_rels(
                        ws.charts.len(),
                        ws.treemap_charts.len() + ws.chartex_charts.len(),
                        ws.images.len(),
                        &img_types,
                        meta.global_chart_start,
                        meta.global_chartex_start,
                        meta.global_image_start,
                    ),
                )?;
            }

            for (ci, chart) in ws.charts.iter().enumerate() {
                let idx = meta.global_chart_start + ci + 1;
                zip.add_file(
                    &format!("xl/charts/chart{idx}.xml"),
                    &chart_writer::write_chart_xml(chart, idx),
                )?;
            }
            // ChartEx (treemap) files
            for (ci, tc) in ws.treemap_charts.iter().enumerate() {
                let idx = meta.global_chartex_start + ci + 1;
                zip.add_file(
                    &format!("xl/charts/chartEx{idx}.xml"),
                    &crate::writer::chartex_writer::write_chartex_xml(tc, idx),
                )?;
                // ChartEx style and color parts + rels
                zip.add_file(&format!("xl/charts/style{idx}.xml"), &chartex_style_xml())?;
                zip.add_file(&format!("xl/charts/colors{idx}.xml"), &chartex_colors_xml())?;
                zip.add_file(
                    &format!("xl/charts/_rels/chartEx{idx}.xml.rels"),
                    &chartex_rels_xml(idx),
                )?;
            }
            // ChartEx (waterfall, funnel, sunburst, histogram, box-whisker) files
            for (ci, cex) in ws.chartex_charts.iter().enumerate() {
                let idx = meta.global_chartex_start + ws.treemap_charts.len() + ci + 1;
                zip.add_file(
                    &format!("xl/charts/chartEx{idx}.xml"),
                    &crate::writer::chartex_writer::write_chartex_generic_xml(cex, idx),
                )?;
                // ChartEx style and color parts + rels
                zip.add_file(&format!("xl/charts/style{idx}.xml"), &chartex_style_xml())?;
                zip.add_file(&format!("xl/charts/colors{idx}.xml"), &chartex_colors_xml())?;
                zip.add_file(
                    &format!("xl/charts/_rels/chartEx{idx}.xml.rels"),
                    &chartex_rels_xml(idx),
                )?;
            }
            for (ii, img) in ws.images.iter().enumerate() {
                let idx = meta.global_image_start + ii + 1;
                zip.add_file(
                    &format!("xl/media/image{}.{}", idx, img.image_type.extension()),
                    &img.data,
                )?;
            }
            for (ti, table) in ws.tables.iter().enumerate() {
                let idx = meta.global_table_start + ti + 1;
                zip.add_file(
                    &format!("xl/tables/table{idx}.xml"),
                    &table_writer::write_table_xml(table, idx),
                )?;
            }
            if !ws.comments.is_empty() {
                zip.add_file(
                    &format!("xl/comments{}.xml", i + 1),
                    &comment_writer::write_comments_xml(&ws.comments),
                )?;
                zip.add_file(
                    &format!("xl/drawings/vmlDrawing{}.vml", i + 1),
                    &comment_writer::write_vml_drawing(&ws.comments),
                )?;
            }
            // Form controls VML
            if !ws.form_controls.is_empty() && ws.comments.is_empty() {
                // Write form controls VML only if no comments VML was written
                zip.add_file(
                    &format!("xl/drawings/vmlDrawing{}.vml", i + 1),
                    &comment_writer::write_form_controls_vml(&ws.form_controls),
                )?;
            }
            // Threaded comments
            if !ws.threaded_comments.is_empty() {
                zip.add_file(
                    &format!("xl/threadedComments/threadedComment{}.xml", i + 1),
                    &comment_writer::write_threaded_comments_xml(&ws.threaded_comments),
                )?;
            }

            // Pivot tables
            for (pi, pt) in ws.pivot_tables.iter().enumerate() {
                let pt_idx = pi + 1; // per-sheet for now, global handled below
                // Parse source range to get sheet name and cell range
                let (src_sheet, src_ref) = parse_pivot_source(&pt.source_range);
                // Find source worksheet and scan data
                let src_ws = self.worksheets.iter().find(|w| w.name == src_sheet);
                if let Some(src) = src_ws
                    && let Some((r1, c1, r2, c2)) =
                        crate::utility::parse_range(&src_ref.replace('$', ""))
                {
                    let cache = crate::writer::pivot_writer::scan_source_data(
                        &src.cells, &self.sst, r1, c1, r2, c2,
                    );
                    // cacheId must match the rId in workbook rels
                    // workbook rels: sheets(N) + styles + sharedStrings + theme + [vba] + pivotCache
                    let cache_rid = sheet_count + 3 + if has_vba { 1 } else { 0 } + pt_idx;
                    let cache_id = cache_rid;
                    zip.add_file(
                        &format!("xl/pivotCache/pivotCacheDefinition{pt_idx}.xml"),
                        &crate::writer::pivot_writer::write_cache_definition_with_groupings(
                            &cache,
                            &src_ref,
                            &src_sheet,
                            cache_id,
                            &pt.field_groupings,
                        ),
                    )?;
                    zip.add_file(
                        &format!("xl/pivotCache/pivotCacheRecords{pt_idx}.xml"),
                        &crate::writer::pivot_writer::write_cache_records(&cache),
                    )?;
                    zip.add_file(
                        &format!("xl/pivotTables/pivotTable{pt_idx}.xml"),
                        &crate::writer::pivot_writer::write_pivot_table(pt, &cache, cache_id),
                    )?;
                    // Pivot table rels → cache definition
                    zip.add_file(&format!("xl/pivotTables/_rels/pivotTable{pt_idx}.xml.rels"),
                            &rel_writer::write_rels(&[("rId1", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheDefinition", &format!("../pivotCache/pivotCacheDefinition{pt_idx}.xml"))]))?;
                    // Cache definition rels → cache records
                    zip.add_file(&format!("xl/pivotCache/_rels/pivotCacheDefinition{pt_idx}.xml.rels"),
                            &rel_writer::write_rels(&[("rId1", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheRecords", &format!("pivotCacheRecords{pt_idx}.xml"))]))?;
                }
            }
        }

        // Write slicer parts
        let mut slicer_sheet_idx = 0usize;
        let mut global_slicer_cache_idx = 0usize;
        for ws in &self.worksheets {
            if ws.slicers.is_empty() {
                continue;
            }
            slicer_sheet_idx += 1;

            // Resolve table IDs and column indices for each slicer
            let mut resolved_slicers: Vec<crate::features::slicer::Slicer> = Vec::new();
            for slicer in &ws.slicers {
                let mut s = slicer.clone();
                // Find the table that contains the source column
                for (ti, table) in ws.tables.iter().enumerate() {
                    if let Some(col_pos) =
                        table.columns.iter().position(|c| c.name == s.source_name)
                    {
                        s.table_id = Some((ti + 1) as u32);
                        s.column_index = Some((col_pos + 1) as u32);
                        break;
                    }
                }
                resolved_slicers.push(s);
            }

            // Write slicer XML part
            zip.add_file(
                &format!("xl/slicers/slicer{slicer_sheet_idx}.xml"),
                &slicer_writer::write_slicer_xml(&resolved_slicers),
            )?;

            // Write slicer cache XML parts (one per slicer)
            for slicer in &resolved_slicers {
                global_slicer_cache_idx += 1;
                zip.add_file(
                    &format!("xl/slicerCaches/slicerCache{global_slicer_cache_idx}.xml"),
                    &slicer_writer::write_slicer_cache_xml(slicer, global_slicer_cache_idx),
                )?;
            }
        }

        // Write timeline parts
        let mut timeline_sheet_idx = 0usize;
        let mut global_timeline_cache_idx = 0usize;
        for ws in &self.worksheets {
            if ws.timelines.is_empty() {
                continue;
            }
            timeline_sheet_idx += 1;

            // Write timeline XML part
            zip.add_file(
                &format!("xl/timelines/timeline{timeline_sheet_idx}.xml"),
                &timeline_writer::write_timeline_xml(&ws.timelines),
            )?;

            // Write timeline cache XML parts (one per timeline)
            for timeline in &ws.timelines {
                global_timeline_cache_idx += 1;
                zip.add_file(
                    &format!("xl/timelineCaches/timelineCache{global_timeline_cache_idx}.xml"),
                    &timeline_writer::write_timeline_cache_xml(timeline, global_timeline_cache_idx),
                )?;
            }
        }

        // Collect custom table styles from all worksheets
        let custom_table_styles: Vec<crate::features::table::CustomTableStyle> = self
            .worksheets
            .iter()
            .flat_map(|ws| ws.tables.iter())
            .filter_map(|t| t.custom_style.clone())
            .collect();

        zip.add_file(
            "xl/styles.xml",
            &style_writer::write_styles_with_table_styles(&self.styles, &custom_table_styles),
        )?;
        zip.add_file("xl/sharedStrings.xml", &sst_writer::write_sst(&self.sst))?;
        zip.add_file(
            "xl/theme/theme1.xml",
            &crate::writer::theme_writer::write_theme(),
        )?;

        // Chart sheets
        for (csi, cs) in self.chart_sheets.iter().enumerate() {
            let cs_idx = csi + 1;
            let chart_idx = total_charts + cs_idx;
            let drawing_idx = sheets_with_drawings.len() + cs_idx;
            // Chartsheet XML
            zip.add_file(
                &format!("xl/chartsheets/sheet{cs_idx}.xml"),
                &write_chartsheet_xml(),
            )?;
            // Chartsheet rels → drawing
            zip.add_file(
                &format!("xl/chartsheets/_rels/sheet{cs_idx}.xml.rels"),
                &rel_writer::write_rels(&[(
                    "rId1",
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing",
                    &format!("../drawings/drawing{drawing_idx}.xml"),
                )]),
            )?;
            // Drawing XML for the chart sheet
            let cs_charts = vec![cs.chart.clone()];
            zip.add_file(
                &format!("xl/drawings/drawing{drawing_idx}.xml"),
                &drawing_writer::write_drawing_xml(&cs_charts, &[], &[], &[], drawing_idx - 1),
            )?;
            // Drawing rels → chart
            zip.add_file(
                &format!("xl/drawings/_rels/drawing{drawing_idx}.xml.rels"),
                &drawing_writer::write_drawing_rels(1, 0, 0, &[], chart_idx - 1, 0, 0),
            )?;
            // Chart XML
            zip.add_file(
                &format!("xl/charts/chart{chart_idx}.xml"),
                &chart_writer::write_chart_xml(&cs.chart, chart_idx),
            )?;
        }

        if has_props {
            zip.add_file(
                "docProps/core.xml",
                &properties::write_core_xml(&self.properties),
            )?;
            zip.add_file(
                "docProps/app.xml",
                &properties::write_app_xml(&self.properties),
            )?;
        }

        // Person list for threaded comments
        {
            let all_threaded: Vec<&crate::worksheet::ThreadedComment> = self
                .worksheets
                .iter()
                .flat_map(|ws| ws.threaded_comments.iter())
                .collect();
            if !all_threaded.is_empty() {
                let owned: Vec<crate::worksheet::ThreadedComment> =
                    all_threaded.into_iter().cloned().collect();
                zip.add_file(
                    "xl/persons/person.xml",
                    &comment_writer::write_persons_xml(&owned),
                )?;
            }
        }

        // Custom XML parts (Task 75)
        for (i, (namespace, content)) in self.custom_xml_parts.iter().enumerate() {
            let idx = i + 1;
            zip.add_file(&format!("customXml/item{idx}.xml"), content)?;
            // Write item properties with namespace
            let props_xml = write_custom_xml_item_props(namespace);
            zip.add_file(&format!("customXml/itemProps{idx}.xml"), &props_xml)?;
        }

        // Custom document properties (Task 76)
        if !self.custom_properties.is_empty() {
            zip.add_file(
                "docProps/custom.xml",
                &properties::write_custom_xml(&self.custom_properties),
            )?;
        }

        // Last, and only where this save has not already written that part itself.
        for (name, data) in &self.passthrough_entries {
            zip.add_file_if_absent(name, data)?;
        }

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

/// Generate chartsheet XML.
fn write_chartsheet_xml() -> Vec<u8> {
    use crate::xml::xml_writer::XmlWriter;
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "chartsheet",
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
    w.start_tag("sheetViews", &[]);
    w.empty_tag(
        "sheetView",
        &[("tabSelected", "0"), ("workbookViewId", "0")],
    );
    w.end_tag("sheetViews");
    w.empty_tag("drawing", &[("r:id", "rId1")]);
    w.end_tag("chartsheet");
    w.into_bytes()
}

/// Generate a minimal ChartEx style XML part.
fn chartex_style_xml() -> Vec<u8> {
    include_bytes!("../writer/chartex_style.xml").to_vec()
}

/// Generate a minimal ChartEx colors XML part.
fn chartex_colors_xml() -> Vec<u8> {
    use crate::xml::xml_writer::XmlWriter;
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "cs:colorStyle",
        &[
            (
                "xmlns:cs",
                "http://schemas.microsoft.com/office/drawing/2012/chartStyle",
            ),
            (
                "xmlns:a",
                "http://schemas.openxmlformats.org/drawingml/2006/main",
            ),
            ("meth", "cycle"),
            ("id", "10"),
        ],
    );
    // Accent colors
    for i in 1..=6 {
        let val = format!("accent{i}");
        w.empty_tag("a:schemeClr", &[("val", &val)]);
    }
    // Variation entries for the color cycle
    w.empty_tag("cs:variation", &[]);
    w.start_tag("cs:variation", &[]);
    w.empty_tag("a:lumMod", &[("val", "60000")]);
    w.end_tag("cs:variation");
    w.start_tag("cs:variation", &[]);
    w.empty_tag("a:lumMod", &[("val", "80000")]);
    w.empty_tag("a:lumOff", &[("val", "20000")]);
    w.end_tag("cs:variation");
    w.start_tag("cs:variation", &[]);
    w.empty_tag("a:lumMod", &[("val", "80000")]);
    w.end_tag("cs:variation");
    w.start_tag("cs:variation", &[]);
    w.empty_tag("a:lumMod", &[("val", "60000")]);
    w.empty_tag("a:lumOff", &[("val", "40000")]);
    w.end_tag("cs:variation");
    w.start_tag("cs:variation", &[]);
    w.empty_tag("a:lumMod", &[("val", "50000")]);
    w.end_tag("cs:variation");
    w.start_tag("cs:variation", &[]);
    w.empty_tag("a:lumMod", &[("val", "70000")]);
    w.empty_tag("a:lumOff", &[("val", "30000")]);
    w.end_tag("cs:variation");
    w.end_tag("cs:colorStyle");
    w.into_bytes()
}

/// Generate relationship file for a ChartEx part pointing to its style and colors.
fn chartex_rels_xml(idx: usize) -> Vec<u8> {
    use crate::writer::rel_writer;
    rel_writer::write_rels(&[
        (
            "rId1",
            "http://schemas.microsoft.com/office/2011/relationships/chartStyle",
            &format!("style{idx}.xml"),
        ),
        (
            "rId2",
            "http://schemas.microsoft.com/office/2011/relationships/chartColorStyle",
            &format!("colors{idx}.xml"),
        ),
    ])
}

/// Generate custom XML item properties XML.
fn write_custom_xml_item_props(namespace: &str) -> Vec<u8> {
    use crate::xml::xml_writer::XmlWriter;
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "ds:datastoreItem",
        &[
            (
                "xmlns:ds",
                "http://schemas.openxmlformats.org/officeDocument/2006/customXml",
            ),
            ("ds:itemID", &format!("{{CUSTOM-{:08X}}}", namespace.len())),
        ],
    );
    w.start_tag("ds:schemaRefs", &[]);
    w.empty_tag("ds:schemaRef", &[("ds:uri", namespace)]);
    w.end_tag("ds:schemaRefs");
    w.end_tag("ds:datastoreItem");
    w.into_bytes()
}
