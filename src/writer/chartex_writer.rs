//! Writer for ChartEx (treemap, sunburst, etc.) — Excel 2016+ cx: namespace.

use crate::features::treemap::TreemapChart;
use crate::xml::xml_writer::XmlWriter;

/// Write a chartEx XML file for a treemap chart.
pub fn write_chartex_xml(chart: &TreemapChart, _chart_id: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("cx:chartSpace", &[
        ("xmlns:cx", "http://schemas.microsoft.com/office/drawing/2014/chartex"),
        ("xmlns:a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
    ]);

    // ── Chart Data ──
    w.start_tag("cx:chartData", &[]);
    w.start_tag("cx:data", &[("id", "0")]);

    // String dimension (categories)
    w.start_tag("cx:strDim", &[("type", "cat")]);
    if let Some(ref cr) = chart.cat_range {
        w.text_element("cx:f", &[], cr);
    }
    let pt_count = chart.categories.len().to_string();
    w.start_tag("cx:lvl", &[("ptCount", &pt_count)]);
    for (i, cat) in chart.categories.iter().enumerate() {
        let idx = i.to_string();
        w.start_tag("cx:pt", &[("idx", &idx)]);
        w.text(cat);
        w.end_tag("cx:pt");
    }
    w.end_tag("cx:lvl");
    w.end_tag("cx:strDim");

    // Numeric dimension (values/sizes)
    w.start_tag("cx:numDim", &[("type", "size")]);
    if let Some(ref vr) = chart.val_range {
        w.text_element("cx:f", &[], vr);
    }
    w.start_tag("cx:lvl", &[("ptCount", &pt_count), ("formatCode", "General")]);
    for (i, val) in chart.values.iter().enumerate() {
        let idx = i.to_string();
        let vs = format!("{val}");
        w.start_tag("cx:pt", &[("idx", &idx)]);
        w.text(&vs);
        w.end_tag("cx:pt");
    }
    w.end_tag("cx:lvl");
    w.end_tag("cx:numDim");

    w.end_tag("cx:data");
    w.end_tag("cx:chartData");

    // ── Chart ──
    w.start_tag("cx:chart", &[]);

    // Title
    if let Some(ref title) = chart.title {
        w.start_tag("cx:title", &[("pos", "t"), ("align", "ctr"), ("overlay", "0")]);
        w.start_tag("cx:tx", &[]);
        w.start_tag("cx:rich", &[]);
        w.empty_tag("a:bodyPr", &[]);
        w.empty_tag("a:lstStyle", &[]);
        w.start_tag("a:p", &[]);
        w.start_tag("a:r", &[]);
        w.text_element("a:t", &[], title);
        w.end_tag("a:r");
        w.end_tag("a:p");
        w.end_tag("cx:rich");
        w.end_tag("cx:tx");
        w.end_tag("cx:title");
    }

    // Plot area with treemap series
    w.start_tag("cx:plotArea", &[]);
    w.start_tag("cx:plotAreaRegion", &[]);

    w.start_tag("cx:series", &[("layoutId", "treemap"), ("uniqueId", "{00000000-0000-0000-0000-000000000001}")]);

    // Series name
    if let Some(ref name) = chart.series_name {
        w.start_tag("cx:tx", &[]);
        w.start_tag("cx:txData", &[]);
        w.text_element("cx:v", &[], name);
        w.end_tag("cx:txData");
        w.end_tag("cx:tx");
    }

    // Data point colors
    for (i, color) in chart.colors.iter().enumerate() {
        if let Some(rgb) = color {
            let idx = i.to_string();
            let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
            w.start_tag("cx:dataPt", &[("idx", &idx)]);
            w.start_tag("cx:spPr", &[]);
            w.start_tag("a:solidFill", &[]);
            w.empty_tag("a:srgbClr", &[("val", &hex)]);
            w.end_tag("a:solidFill");
            w.end_tag("cx:spPr");
            w.end_tag("cx:dataPt");
        }
    }

    w.empty_tag("cx:dataId", &[("val", "0")]);
    w.start_tag("cx:layoutPr", &[]);
    w.empty_tag("cx:parentLabelLayout", &[("val", "overlapping")]);
    w.end_tag("cx:layoutPr");

    w.end_tag("cx:series");
    w.end_tag("cx:plotAreaRegion");
    w.end_tag("cx:plotArea");

    // Legend
    w.start_tag("cx:legend", &[("pos", "b"), ("align", "ctr"), ("overlay", "0")]);
    w.end_tag("cx:legend");

    w.end_tag("cx:chart");
    w.end_tag("cx:chartSpace");
    w.into_bytes()
}
