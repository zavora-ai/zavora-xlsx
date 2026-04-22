use crate::features::timeline::Timeline;
use crate::xml::xml_writer::XmlWriter;

/// Write the timeline XML part (xl/timelines/timeline{N}.xml).
pub fn write_timeline_xml(timelines: &[Timeline]) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "timelines",
        &[
            (
                "xmlns",
                "http://schemas.microsoft.com/office/spreadsheetml/2010/11/main",
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

    for timeline in timelines {
        let cache_name = timeline_cache_name(&timeline.name);
        let level_s = timeline.level.to_xml_value().to_string();
        w.empty_tag(
            "timeline",
            &[
                ("name", &timeline.name),
                ("cache", &cache_name),
                ("caption", &timeline.caption),
                ("level", &level_s),
                ("selectionLevel", &level_s),
                ("scrollPosition", ""),
            ],
        );
    }

    w.end_tag("timelines");
    w.into_bytes()
}

/// Write the timeline cache XML part (xl/timelineCaches/timelineCache{N}.xml).
pub fn write_timeline_cache_xml(timeline: &Timeline, _cache_index: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();

    let cache_name = timeline_cache_name(&timeline.name);

    w.start_tag(
        "timelineCacheDefinition",
        &[
            (
                "xmlns",
                "http://schemas.microsoft.com/office/spreadsheetml/2010/11/main",
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
            ("sourceName", &timeline.source_name),
        ],
    );

    // pivotTables element referencing the pivot table
    w.start_tag("pivotTables", &[]);
    w.empty_tag("pivotTable", &[("tabId", "0"), ("pivotCacheId", "1")]);
    w.end_tag("pivotTables");

    // State element
    w.start_tag(
        "state",
        &[
            ("minimalRefreshVersion", "6"),
            ("lastRefreshVersion", "6"),
            ("pivotCacheId", "1"),
            ("filterType", "dateBetween"),
        ],
    );
    w.end_tag("state");

    w.end_tag("timelineCacheDefinition");
    w.into_bytes()
}

/// Generate the cache name for a timeline (used as the link between timeline and cache).
pub fn timeline_cache_name(timeline_name: &str) -> String {
    format!("NativeTimeline_{}", timeline_name.replace(' ', "_"))
}
