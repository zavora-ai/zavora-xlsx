use std::collections::{BTreeMap, BTreeSet};
use crate::cell::CellValue;
use crate::features::pivot::{PivotTable, PivotAggregation, PivotLayout};
use crate::xml::xml_writer::XmlWriter;

/// Scanned cache data from source worksheet.
pub(crate) struct PivotCacheData {
    pub headers: Vec<String>,
    pub fields: Vec<CacheFieldData>,
    pub records: Vec<Vec<CacheValue>>,
}

pub(crate) enum CacheFieldData {
    String { unique_values: Vec<String> },
    Number { min: f64, max: f64, has_int: bool },
    Mixed,
}

pub(crate) enum CacheValue {
    StringIndex(u32),
    Number(f64),
    Blank,
}

/// Scan source data from worksheet cells to build cache.
pub(crate) fn scan_source_data(
    cells: &std::collections::BTreeMap<u32, std::collections::BTreeMap<u16, (crate::cell::CellType, u32)>>,
    sst: &crate::model::shared_strings::SharedStringTable,
    r1: u32, c1: u16, r2: u32, c2: u16,
) -> PivotCacheData {
    let col_count = (c2 - c1 + 1) as usize;

    // Read headers from first row
    let headers: Vec<String> = (c1..=c2).map(|c| {
        cells.get(&r1).and_then(|row| row.get(&c)).map(|(cell, _)| {
            match cell {
                crate::cell::CellType::InlineString(s) | crate::cell::CellType::Error(s) => s.clone(),
                crate::cell::CellType::SharedString(idx) => sst.get(*idx).unwrap_or("").to_string(),
                crate::cell::CellType::Number(n) => format!("{n}"),
                _ => format!("Col{}", c - c1 + 1),
            }
        }).unwrap_or_else(|| format!("Col{}", c - c1 + 1))
    }).collect();

    // Collect values per column
    let mut col_values: Vec<Vec<CellValue>> = vec![Vec::new(); col_count];
    for r in (r1 + 1)..=r2 {
        for (ci, c) in (c1..=c2).enumerate() {
            let val = cells.get(&r).and_then(|row| row.get(&c)).map(|(cell, _)| {
                match cell {
                    crate::cell::CellType::Number(n) => CellValue::Number(*n),
                    crate::cell::CellType::InlineString(s) => CellValue::String(s.clone()),
                    crate::cell::CellType::SharedString(idx) => {
                        CellValue::String(sst.get(*idx).unwrap_or("").to_string())
                    }
                    crate::cell::CellType::Bool(b) => CellValue::String(if *b { "TRUE" } else { "FALSE" }.into()),
                    _ => CellValue::Empty,
                }
            }).unwrap_or(CellValue::Empty);
            col_values[ci].push(val);
        }
    }

    // Build field metadata + records
    let mut fields = Vec::with_capacity(col_count);
    let mut records: Vec<Vec<CacheValue>> = (0..(r2 - r1) as usize).map(|_| Vec::with_capacity(col_count)).collect();

    for (ci, values) in col_values.iter().enumerate() {
        let all_numeric = values.iter().all(|v| matches!(v, CellValue::Number(_) | CellValue::Empty));

        if all_numeric {
            let mut min = f64::MAX;
            let mut max = f64::MIN;
            let mut has_int = true;
            for v in values {
                if let CellValue::Number(n) = v {
                    min = min.min(*n); max = max.max(*n);
                    if *n != n.floor() { has_int = false; }
                }
            }
            if min == f64::MAX { min = 0.0; max = 0.0; }
            fields.push(CacheFieldData::Number { min, max, has_int });
            for (ri, v) in values.iter().enumerate() {
                records[ri].push(match v {
                    CellValue::Number(n) => CacheValue::Number(*n),
                    _ => CacheValue::Blank,
                });
            }
        } else {
            let mut unique: Vec<String> = Vec::new();
            let mut seen: BTreeSet<String> = BTreeSet::new();
            for v in values {
                let s = match v {
                    CellValue::String(s) => s.clone(),
                    CellValue::Number(n) => format!("{n}"),
                    _ => String::new(),
                };
                if !s.is_empty() && seen.insert(s.clone()) { unique.push(s); }
            }
            let index_map: BTreeMap<&str, u32> = unique.iter().enumerate().map(|(i, s)| (s.as_str(), i as u32)).collect();
            for (ri, v) in values.iter().enumerate() {
                let s = match v {
                    CellValue::String(s) => s.as_str(),
                    CellValue::Number(n) => { records[ri].push(CacheValue::Number(*n)); continue; }
                    _ => { records[ri].push(CacheValue::Blank); continue; }
                };
                records[ri].push(CacheValue::StringIndex(*index_map.get(s).unwrap_or(&0)));
            }
            fields.push(CacheFieldData::String { unique_values: unique });
        }
    }

    PivotCacheData { headers, fields, records }
}

/// Write pivotCacheDefinition XML.
pub(crate) fn write_cache_definition(cache: &PivotCacheData, source_ref: &str, source_sheet: &str, _cache_id: usize) -> Vec<u8> {
    let clean_ref = source_ref.replace('$', "");
    let mut w = XmlWriter::new();
    w.declaration();
    let rec_count = cache.records.len().to_string();
    w.start_tag("pivotCacheDefinition", &[
        ("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
        ("r:id", "rId1"),
        ("refreshedBy", "zavora-xlsx"),
        ("createdVersion", "8"), ("refreshedVersion", "8"), ("minRefreshableVersion", "3"),
        ("recordCount", &rec_count),
    ]);

    w.start_tag("cacheSource", &[("type", "worksheet")]);
    w.empty_tag("worksheetSource", &[("ref", &clean_ref), ("sheet", source_sheet)]);
    w.end_tag("cacheSource");

    let fc = cache.fields.len().to_string();
    w.start_tag("cacheFields", &[("count", &fc)]);
    for (i, field) in cache.fields.iter().enumerate() {
        w.start_tag("cacheField", &[("name", &cache.headers[i]), ("numFmtId", "0")]);
        match field {
            CacheFieldData::String { unique_values } => {
                let count = unique_values.len().to_string();
                w.start_tag("sharedItems", &[("count", &count)]);
                for v in unique_values { w.empty_tag("s", &[("v", v)]); }
                w.end_tag("sharedItems");
            }
            CacheFieldData::Number { min, max, has_int } => {
                let min_s = format!("{min}"); let max_s = format!("{max}");
                let mut attrs: Vec<(&str, &str)> = vec![
                    ("containsSemiMixedTypes", "0"), ("containsString", "0"),
                    ("containsNumber", "1"),
                ];
                if *has_int { attrs.push(("containsInteger", "1")); }
                attrs.push(("minValue", &min_s));
                attrs.push(("maxValue", &max_s));
                w.empty_tag("sharedItems", &attrs);
            }
            CacheFieldData::Mixed => {
                w.empty_tag("sharedItems", &[("containsMixedTypes", "1")]);
            }
        }
        w.end_tag("cacheField");
    }
    w.end_tag("cacheFields");
    w.end_tag("pivotCacheDefinition");
    w.into_bytes()
}

/// Write pivotCacheRecords XML.
pub(crate) fn write_cache_records(cache: &PivotCacheData) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    let count = cache.records.len().to_string();
    w.start_tag("pivotCacheRecords", &[
        ("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
        ("count", &count),
    ]);
    for record in &cache.records {
        w.start_tag("r", &[]);
        for val in record {
            match val {
                CacheValue::StringIndex(idx) => { let s = idx.to_string(); w.empty_tag("x", &[("v", &s)]); }
                CacheValue::Number(n) => { let s = format!("{n}"); w.empty_tag("n", &[("v", &s)]); }
                CacheValue::Blank => { w.empty_tag("m", &[]); }
            }
        }
        w.end_tag("r");
    }
    w.end_tag("pivotCacheRecords");
    w.into_bytes()
}

/// Write pivotTableDefinition XML.
pub(crate) fn write_pivot_table(pt: &PivotTable, cache: &PivotCacheData, cache_id: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();

    let cache_id_s = cache_id.to_string();
    let mut pt_attrs: Vec<(&str, &str)> = vec![
        ("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main"),
        ("name", &pt.name), ("cacheId", &cache_id_s),
        ("dataCaption", "Values"),
        ("updatedVersion", "8"), ("createdVersion", "8"), ("minRefreshableVersion", "3"),
        ("useAutoFormatting", "1"), ("itemPrintTitles", "1"),
        ("indent", "0"), ("outline", "1"), ("outlineData", "1"),
        ("multipleFieldFilters", "0"),
    ];
    if !pt.show_row_grand_total { pt_attrs.push(("rowGrandTotals", "0")); }
    if !pt.show_col_grand_total { pt_attrs.push(("colGrandTotals", "0")); }
    let compact = matches!(pt.layout, PivotLayout::Compact);
    if !compact { pt_attrs.push(("compact", "0")); pt_attrs.push(("compactData", "0")); }
    w.start_tag("pivotTableDefinition", &pt_attrs);

    // Compute location
    let data_cols = pt.value_fields.len().max(1);
    let n_col_items = pt.column_fields.iter().filter_map(|f| {
        let idx = cache.headers.iter().position(|h| h == f)?;
        match &cache.fields[idx] { CacheFieldData::String { unique_values } => Some(unique_values.len()), _ => None }
    }).product::<usize>().max(1);
    let total_cols = pt.row_fields.len() + n_col_items * data_cols;
    let page_rows = if pt.filter_fields.is_empty() { 0 } else { pt.filter_fields.len() + 1 };
    let loc_ref = format!("{}{}:{}{}", crate::utility::col_to_letter(pt.col),
        pt.row + 1 + page_rows as u32,
        crate::utility::col_to_letter(pt.col + total_cols as u16), pt.row + 5 + page_rows as u32);
    let first_data_col = pt.row_fields.len().to_string();
    let mut loc_attrs: Vec<(&str, &str)> = vec![("ref", &loc_ref), ("firstHeaderRow", "1"), ("firstDataRow", "1"), ("firstDataCol", &first_data_col)];
    let page_count;
    if !pt.filter_fields.is_empty() {
        page_count = pt.filter_fields.len().to_string();
        loc_attrs.push(("rowPageCount", &page_count));
        loc_attrs.push(("colPageCount", "1"));
    }
    w.empty_tag("location", &loc_attrs);

    // pivotFields — one per source column
    let fc = cache.fields.len().to_string();
    w.start_tag("pivotFields", &[("count", &fc)]);
    for (i, header) in cache.headers.iter().enumerate() {
        let is_row = pt.row_fields.contains(header);
        let is_col = pt.column_fields.contains(header);
        let is_filter = pt.filter_fields.contains(header);
        let is_data = pt.value_fields.iter().any(|vf| vf.source_field == *header);

        let mut attrs: Vec<(&str, &str)> = Vec::new();
        if is_row { attrs.push(("axis", "axisRow")); }
        else if is_col { attrs.push(("axis", "axisCol")); }
        else if is_filter { attrs.push(("axis", "axisPage")); }
        if is_data { attrs.push(("dataField", "1")); }
        if !compact && (is_row || is_col) {
            attrs.push(("compact", "0")); attrs.push(("outline", "0"));
        }
        attrs.push(("showAll", "0"));

        let has_items = is_row || is_col || is_filter;
        if has_items {
            if let CacheFieldData::String { unique_values } = &cache.fields[i] {
                w.start_tag("pivotField", &attrs);
                let item_count = (unique_values.len() + 1).to_string();
                w.start_tag("items", &[("count", &item_count)]);
                for j in 0..unique_values.len() {
                    let js = j.to_string();
                    w.empty_tag("item", &[("x", &js)]);
                }
                w.empty_tag("item", &[("t", "default")]);
                w.end_tag("items");
                w.end_tag("pivotField");
            } else {
                w.empty_tag("pivotField", &attrs);
            }
        } else {
            w.empty_tag("pivotField", &attrs);
        }
    }
    w.end_tag("pivotFields");

    // rowFields
    if !pt.row_fields.is_empty() {
        let rc = pt.row_fields.len().to_string();
        w.start_tag("rowFields", &[("count", &rc)]);
        for f in &pt.row_fields {
            if let Some(idx) = cache.headers.iter().position(|h| h == f) {
                let s = idx.to_string();
                w.empty_tag("field", &[("x", &s)]);
            }
        }
        w.end_tag("rowFields");
    }

    // rowItems — one per unique value in first row field + grand total
    if !pt.row_fields.is_empty() {
        if let Some(first_row_field) = pt.row_fields.first() {
            if let Some(idx) = cache.headers.iter().position(|h| h == first_row_field) {
                if let CacheFieldData::String { unique_values } = &cache.fields[idx] {
                    let count = (unique_values.len() + 1).to_string();
                    w.start_tag("rowItems", &[("count", &count)]);
                    for vi in 0..unique_values.len() {
                        w.start_tag("i", &[]);
                        let vs = vi.to_string();
                        w.empty_tag("x", &[("v", &vs)]);
                        w.end_tag("i");
                    }
                    w.start_tag("i", &[("t", "grand")]);
                    w.empty_tag("x", &[]);
                    w.end_tag("i");
                    w.end_tag("rowItems");
                } else {
                    w.start_tag("rowItems", &[("count", "1")]);
                    w.start_tag("i", &[("t", "grand")]); w.empty_tag("x", &[]); w.end_tag("i");
                    w.end_tag("rowItems");
                }
            }
        }
    } else {
        w.start_tag("rowItems", &[("count", "1")]);
        w.start_tag("i", &[("t", "grand")]); w.empty_tag("x", &[]); w.end_tag("i");
        w.end_tag("rowItems");
    }

    // colFields
    if !pt.column_fields.is_empty() || pt.value_fields.len() > 1 {
        let mut col_count = pt.column_fields.len();
        if pt.value_fields.len() > 1 { col_count += 1; }
        let cc = col_count.to_string();
        w.start_tag("colFields", &[("count", &cc)]);
        for f in &pt.column_fields {
            if let Some(idx) = cache.headers.iter().position(|h| h == f) {
                let s = idx.to_string();
                w.empty_tag("field", &[("x", &s)]);
            }
        }
        if pt.value_fields.len() > 1 {
            w.empty_tag("field", &[("x", "-2")]); // -2 = data field
        }
        w.end_tag("colFields");
    }

    // colItems — one per unique value in column field + grand total
    if !pt.column_fields.is_empty() {
        if let Some(col_field) = pt.column_fields.first() {
            if let Some(idx) = cache.headers.iter().position(|h| h == col_field) {
                if let CacheFieldData::String { unique_values } = &cache.fields[idx] {
                    let count = (unique_values.len() + 1).to_string();
                    w.start_tag("colItems", &[("count", &count)]);
                    for vi in 0..unique_values.len() {
                        w.start_tag("i", &[]);
                        let vs = vi.to_string();
                        w.empty_tag("x", &[("v", &vs)]);
                        w.end_tag("i");
                    }
                    w.start_tag("i", &[("t", "grand")]);
                    w.empty_tag("x", &[]);
                    w.end_tag("i");
                    w.end_tag("colItems");
                } else {
                    goto_default_col_items(&mut w, &pt.value_fields);
                }
            } else {
                goto_default_col_items(&mut w, &pt.value_fields);
            }
        }
    } else {
        goto_default_col_items(&mut w, &pt.value_fields);
    }

    // pageFields (filters)
    if !pt.filter_fields.is_empty() {
        let pfc = pt.filter_fields.len().to_string();
        w.start_tag("pageFields", &[("count", &pfc)]);
        for f in &pt.filter_fields {
            if let Some(idx) = cache.headers.iter().position(|h| h == f) {
                let s = idx.to_string();
                w.empty_tag("pageField", &[("fld", &s), ("hier", "-1")]);
            }
        }
        w.end_tag("pageFields");
    }

    // dataFields
    let dfc = pt.value_fields.len().to_string();
    w.start_tag("dataFields", &[("count", &dfc)]);
    for vf in &pt.value_fields {
        if let Some(idx) = cache.headers.iter().position(|h| h == &vf.source_field) {
            let fld = idx.to_string();
            let mut attrs: Vec<(&str, &str)> = vec![("name", &vf.name), ("fld", &fld)];
            let sub = vf.aggregation.xml_str();
            if !matches!(vf.aggregation, PivotAggregation::Sum) { attrs.push(("subtotal", sub)); }
            attrs.push(("baseField", "0"));
            attrs.push(("baseItem", "0"));
            let nf;
            if let Some(ref fmt) = vf.num_format {
                nf = fmt.clone();
                attrs.push(("numFmtId", "0")); // Excel will apply on refresh
            }
            w.empty_tag("dataField", &attrs);
        }
    }
    w.end_tag("dataFields");

    // Style
    let style_name = match &pt.style { PivotStyle::Named(n) => n.as_str() };
    w.empty_tag("pivotTableStyleInfo", &[
        ("name", style_name),
        ("showRowHeaders", if pt.show_row_headers { "1" } else { "0" }),
        ("showColHeaders", if pt.show_col_headers { "1" } else { "0" }),
        ("showRowStripes", if pt.show_row_stripes { "1" } else { "0" }),
        ("showColStripes", if pt.show_col_stripes { "1" } else { "0" }),
        ("showLastColumn", "1"),
    ]);

    w.end_tag("pivotTableDefinition");
    w.into_bytes()
}

use crate::features::pivot::PivotStyle;

fn goto_default_col_items(w: &mut XmlWriter, value_fields: &[crate::features::pivot::PivotValueField]) {
    let n = value_fields.len().max(1);
    let cc = n.to_string();
    w.start_tag("colItems", &[("count", &cc)]);
    for vi in 0..n {
        let vs = vi.to_string();
        if vi == 0 { w.start_tag("i", &[]); } else { w.start_tag("i", &[("i", &vs)]); }
        w.empty_tag("x", &[("v", &vs)]);
        w.end_tag("i");
    }
    w.end_tag("colItems");
}
