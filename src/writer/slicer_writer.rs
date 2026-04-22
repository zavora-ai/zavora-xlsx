use crate::features::slicer::Slicer;
use crate::xml::xml_writer::XmlWriter;

/// Write the slicer XML part (xl/slicers/slicer{N}.xml).
pub fn write_slicer_xml(slicers: &[Slicer]) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "slicers",
        &[
            (
                "xmlns",
                "http://schemas.microsoft.com/office/spreadsheetml/2009/9/main",
            ),
            (
                "xmlns:mc",
                "http://schemas.openxmlformats.org/markup-compatibility/2006",
            ),
            ("mc:Ignorable", "x"),
            (
                "xmlns:x",
                "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
            ),
        ],
    );

    for slicer in slicers {
        let cache_name = slicer_cache_name(&slicer.name);
        w.empty_tag(
            "slicer",
            &[
                ("name", &slicer.name),
                ("cache", &cache_name),
                ("caption", &slicer.caption),
                ("startItem", "0"),
                ("columnCount", "1"),
                ("showCaption", "1"),
                ("style", &slicer.style),
            ],
        );
    }

    w.end_tag("slicers");
    w.into_bytes()
}

/// Write the slicer cache XML part (xl/slicerCaches/slicerCache{N}.xml).
pub fn write_slicer_cache_xml(slicer: &Slicer, cache_index: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();

    let cache_name = slicer_cache_name(&slicer.name);

    w.start_tag(
        "slicerCacheDefinition",
        &[
            (
                "xmlns",
                "http://schemas.microsoft.com/office/spreadsheetml/2009/9/main",
            ),
            (
                "xmlns:mc",
                "http://schemas.openxmlformats.org/markup-compatibility/2006",
            ),
            ("mc:Ignorable", "x"),
            (
                "xmlns:x",
                "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
            ),
            ("name", &cache_name),
            ("sourceName", &slicer.source_name),
        ],
    );

    // Extension list with table slicer cache info
    w.start_tag("extLst", &[]);
    w.start_tag(
        "ext",
        &[
            ("uri", "{2F2917AC-EB37-4324-AD4E-5DD8C200BD13}"),
            (
                "xmlns:x15",
                "http://schemas.microsoft.com/office/spreadsheetml/2010/11/main",
            ),
        ],
    );

    if let Some(ref _pivot_cache) = slicer.pivot_cache_name {
        // Pivot table slicer cache
        let cache_id_s = cache_index.to_string();
        w.empty_tag("x15:pivotCacheId", &[("val", &cache_id_s)]);
    } else {
        // Table slicer cache
        let table_id = slicer.table_id.unwrap_or(1);
        let col_idx = slicer.column_index.unwrap_or(1);
        let table_id_s = table_id.to_string();
        let col_idx_s = col_idx.to_string();
        w.empty_tag(
            "x15:tableSlicerCache",
            &[("tableId", &table_id_s), ("column", &col_idx_s)],
        );
    }

    w.end_tag("ext");
    w.end_tag("extLst");
    w.end_tag("slicerCacheDefinition");
    w.into_bytes()
}

/// Generate the cache name for a slicer (used as the link between slicer and cache).
pub fn slicer_cache_name(slicer_name: &str) -> String {
    format!("Slicer_{}", slicer_name.replace(' ', "_"))
}
