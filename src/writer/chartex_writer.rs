//! Writer for ChartEx (treemap, sunburst, etc.) — Excel 2016+ cx: namespace.

use crate::features::chartex::{
    BoxWhiskerChart, ChartExChart, FunnelChart, HistogramChart, MapChart, SunburstChart,
    WaterfallChart, WaterfallPointType,
};
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

/// Write a chartEx XML file for any ChartEx chart type.
pub fn write_chartex_generic_xml(chart: &ChartExChart, _chart_id: usize) -> Vec<u8> {
    match chart {
        ChartExChart::Waterfall(c) => write_waterfall_xml(c),
        ChartExChart::Funnel(c) => write_funnel_xml(c),
        ChartExChart::Sunburst(c) => write_sunburst_xml(c),
        ChartExChart::Histogram(c) => write_histogram_xml(c),
        ChartExChart::BoxWhisker(c) => write_box_whisker_xml(c),
        ChartExChart::Map(c) => write_map_xml(c),
    }
}

fn chartex_header(w: &mut XmlWriter) {
    w.declaration();
    w.start_tag("cx:chartSpace", &[
        ("xmlns:cx", "http://schemas.microsoft.com/office/drawing/2014/chartex"),
        ("xmlns:a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
    ]);
}

fn write_chartex_title(w: &mut XmlWriter, title: &Option<String>) {
    if let Some(title) = title {
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
}

fn write_waterfall_xml(chart: &WaterfallChart) -> Vec<u8> {
    let mut w = XmlWriter::new();
    chartex_header(&mut w);

    // Chart Data
    w.start_tag("cx:chartData", &[]);
    w.start_tag("cx:data", &[("id", "0")]);

    // String dimension (categories)
    w.start_tag("cx:strDim", &[("type", "cat")]);
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

    // Numeric dimension (values)
    w.start_tag("cx:numDim", &[("type", "val")]);
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

    // Chart
    w.start_tag("cx:chart", &[]);
    write_chartex_title(&mut w, &chart.title);

    w.start_tag("cx:plotArea", &[]);
    w.start_tag("cx:plotAreaRegion", &[]);

    w.start_tag("cx:series", &[("layoutId", "waterfall"), ("uniqueId", "{00000000-0000-0000-0000-000000000002}")]);

    if let Some(ref name) = chart.series_name {
        w.start_tag("cx:tx", &[]);
        w.start_tag("cx:txData", &[]);
        w.text_element("cx:v", &[], name);
        w.end_tag("cx:txData");
        w.end_tag("cx:tx");
    }

    // Data point categorization (total, increase, decrease)
    for (i, pt_type) in chart.point_types.iter().enumerate() {
        if *pt_type == WaterfallPointType::Total {
            let idx = i.to_string();
            w.start_tag("cx:dataPt", &[("idx", &idx)]);
            w.start_tag("cx:layoutPr", &[]);
            w.empty_tag("cx:subtotal", &[]);
            w.end_tag("cx:layoutPr");
            w.end_tag("cx:dataPt");
        }
    }

    w.empty_tag("cx:dataId", &[("val", "0")]);
    w.end_tag("cx:series");
    w.end_tag("cx:plotAreaRegion");

    // Axes
    w.start_tag("cx:axis", &[("id", "0")]);
    w.empty_tag("cx:catScaling", &[]);
    w.end_tag("cx:axis");
    w.start_tag("cx:axis", &[("id", "1")]);
    w.empty_tag("cx:valScaling", &[]);
    w.end_tag("cx:axis");

    w.end_tag("cx:plotArea");
    w.end_tag("cx:chart");
    w.end_tag("cx:chartSpace");
    w.into_bytes()
}

fn write_funnel_xml(chart: &FunnelChart) -> Vec<u8> {
    let mut w = XmlWriter::new();
    chartex_header(&mut w);

    // Chart Data
    w.start_tag("cx:chartData", &[]);
    w.start_tag("cx:data", &[("id", "0")]);

    // String dimension (categories)
    w.start_tag("cx:strDim", &[("type", "cat")]);
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

    // Numeric dimension (values)
    w.start_tag("cx:numDim", &[("type", "val")]);
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

    // Chart
    w.start_tag("cx:chart", &[]);
    write_chartex_title(&mut w, &chart.title);

    w.start_tag("cx:plotArea", &[]);
    w.start_tag("cx:plotAreaRegion", &[]);

    w.start_tag("cx:series", &[("layoutId", "funnel"), ("uniqueId", "{00000000-0000-0000-0000-000000000003}")]);

    if let Some(ref name) = chart.series_name {
        w.start_tag("cx:tx", &[]);
        w.start_tag("cx:txData", &[]);
        w.text_element("cx:v", &[], name);
        w.end_tag("cx:txData");
        w.end_tag("cx:tx");
    }

    w.empty_tag("cx:dataId", &[("val", "0")]);
    w.end_tag("cx:series");
    w.end_tag("cx:plotAreaRegion");
    w.end_tag("cx:plotArea");

    w.end_tag("cx:chart");
    w.end_tag("cx:chartSpace");
    w.into_bytes()
}

fn write_sunburst_xml(chart: &SunburstChart) -> Vec<u8> {
    let mut w = XmlWriter::new();
    chartex_header(&mut w);

    // Chart Data
    w.start_tag("cx:chartData", &[]);
    w.start_tag("cx:data", &[("id", "0")]);

    // String dimension (categories) — multi-level hierarchy
    w.start_tag("cx:strDim", &[("type", "cat")]);
    // Each level is a <cx:lvl> element; outermost level first in the XML
    for level in chart.levels.iter().rev() {
        let pt_count = level.labels.len().to_string();
        w.start_tag("cx:lvl", &[("ptCount", &pt_count)]);
        for (i, label) in level.labels.iter().enumerate() {
            let idx = i.to_string();
            w.start_tag("cx:pt", &[("idx", &idx)]);
            w.text(label);
            w.end_tag("cx:pt");
        }
        w.end_tag("cx:lvl");
    }
    w.end_tag("cx:strDim");

    // Numeric dimension (values/sizes)
    let val_count = chart.values.len().to_string();
    w.start_tag("cx:numDim", &[("type", "size")]);
    w.start_tag("cx:lvl", &[("ptCount", &val_count), ("formatCode", "General")]);
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

    // Chart
    w.start_tag("cx:chart", &[]);
    write_chartex_title(&mut w, &chart.title);

    w.start_tag("cx:plotArea", &[]);
    w.start_tag("cx:plotAreaRegion", &[]);

    w.start_tag("cx:series", &[("layoutId", "sunburst"), ("uniqueId", "{00000000-0000-0000-0000-000000000004}")]);

    if let Some(ref name) = chart.series_name {
        w.start_tag("cx:tx", &[]);
        w.start_tag("cx:txData", &[]);
        w.text_element("cx:v", &[], name);
        w.end_tag("cx:txData");
        w.end_tag("cx:tx");
    }

    w.empty_tag("cx:dataId", &[("val", "0")]);
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

fn write_histogram_xml(chart: &HistogramChart) -> Vec<u8> {
    let mut w = XmlWriter::new();
    chartex_header(&mut w);

    // Chart Data
    w.start_tag("cx:chartData", &[]);
    w.start_tag("cx:data", &[("id", "0")]);

    // Numeric dimension (values)
    let pt_count = chart.values.len().to_string();
    w.start_tag("cx:numDim", &[("type", "val")]);
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

    // Chart
    w.start_tag("cx:chart", &[]);
    write_chartex_title(&mut w, &chart.title);

    w.start_tag("cx:plotArea", &[]);
    w.start_tag("cx:plotAreaRegion", &[]);

    let layout_id = if chart.is_pareto { "paretoLine" } else { "histogram" };
    w.start_tag("cx:series", &[("layoutId", layout_id), ("uniqueId", "{00000000-0000-0000-0000-000000000005}")]);

    if let Some(ref name) = chart.series_name {
        w.start_tag("cx:tx", &[]);
        w.start_tag("cx:txData", &[]);
        w.text_element("cx:v", &[], name);
        w.end_tag("cx:txData");
        w.end_tag("cx:tx");
    }

    w.empty_tag("cx:dataId", &[("val", "0")]);

    // Binning configuration
    w.start_tag("cx:layoutPr", &[]);
    let mut bin_attrs: Vec<(&str, String)> = Vec::new();
    if let Some(count) = chart.bin_count {
        bin_attrs.push(("binCount", count.to_string()));
    }
    if let Some(width) = chart.bin_width {
        bin_attrs.push(("binWidth", format!("{width}")));
    }
    if bin_attrs.is_empty() {
        w.empty_tag("cx:binning", &[]);
    } else {
        let attrs: Vec<(&str, &str)> = bin_attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();
        w.empty_tag("cx:binning", &attrs);
    }
    w.end_tag("cx:layoutPr");

    w.end_tag("cx:series");
    w.end_tag("cx:plotAreaRegion");

    // Axes
    w.start_tag("cx:axis", &[("id", "0")]);
    w.empty_tag("cx:catScaling", &[]);
    w.empty_tag("cx:tickLabels", &[]);
    w.end_tag("cx:axis");
    w.start_tag("cx:axis", &[("id", "1")]);
    w.empty_tag("cx:valScaling", &[]);
    w.empty_tag("cx:majorGridlines", &[]);
    w.empty_tag("cx:tickLabels", &[]);
    w.end_tag("cx:axis");

    w.end_tag("cx:plotArea");
    w.end_tag("cx:chart");
    w.end_tag("cx:chartSpace");
    w.into_bytes()
}

fn write_box_whisker_xml(chart: &BoxWhiskerChart) -> Vec<u8> {
    let mut w = XmlWriter::new();
    chartex_header(&mut w);

    // Chart Data — one cx:data block per data set (series)
    w.start_tag("cx:chartData", &[]);
    for (i, (cat, values)) in chart.categories.iter().zip(chart.data_sets.iter()).enumerate() {
        let id = i.to_string();
        w.start_tag("cx:data", &[("id", &id)]);

        // String dimension (category labels — shared across all)
        w.start_tag("cx:strDim", &[("type", "cat")]);
        let pt_count = chart.categories.len().to_string();
        w.start_tag("cx:lvl", &[("ptCount", &pt_count)]);
        for (j, c) in chart.categories.iter().enumerate() {
            let idx = j.to_string();
            w.start_tag("cx:pt", &[("idx", &idx)]);
            w.text(c);
            w.end_tag("cx:pt");
        }
        w.end_tag("cx:lvl");
        w.end_tag("cx:strDim");

        // Numeric dimension (values for this series)
        let val_count = values.len().to_string();
        w.start_tag("cx:numDim", &[("type", "val")]);
        w.start_tag("cx:lvl", &[("ptCount", &val_count), ("formatCode", "General")]);
        for (j, val) in values.iter().enumerate() {
            let idx = j.to_string();
            let vs = if *val == val.trunc() { format!("{}", *val as i64) } else { format!("{val}") };
            w.start_tag("cx:pt", &[("idx", &idx)]);
            w.text(&vs);
            w.end_tag("cx:pt");
        }
        w.end_tag("cx:lvl");
        w.end_tag("cx:numDim");

        w.end_tag("cx:data");
    }
    w.end_tag("cx:chartData");

    // Chart
    w.start_tag("cx:chart", &[]);
    if let Some(ref title) = chart.title {
        w.start_tag("cx:title", &[("pos", "t"), ("align", "ctr"), ("overlay", "0")]);
        w.end_tag("cx:title");
    }

    w.start_tag("cx:plotArea", &[]);
    w.start_tag("cx:plotAreaRegion", &[]);

    // One series per data set
    for (i, cat) in chart.categories.iter().enumerate() {
        let id = i.to_string();
        let uid = format!("{{00000000-0000-0000-0000-{:012X}}}", i + 1);
        w.start_tag("cx:series", &[("layoutId", "boxWhisker"), ("uniqueId", &uid)]);

        // Series name
        w.start_tag("cx:tx", &[]);
        w.start_tag("cx:txData", &[]);
        w.text_element("cx:v", &[], cat);
        w.end_tag("cx:txData");
        w.end_tag("cx:tx");

        w.empty_tag("cx:dataId", &[("val", &id)]);

        w.start_tag("cx:layoutPr", &[]);
        w.empty_tag("cx:statistics", &[("quartileMethod", "exclusive")]);
        w.end_tag("cx:layoutPr");

        w.end_tag("cx:series");
    }

    w.end_tag("cx:plotAreaRegion");

    // Axes
    w.start_tag("cx:axis", &[("id", "0")]);
    w.empty_tag("cx:catScaling", &[("gapWidth", "1")]);
    w.empty_tag("cx:tickLabels", &[]);
    w.end_tag("cx:axis");
    w.start_tag("cx:axis", &[("id", "1")]);
    w.empty_tag("cx:valScaling", &[]);
    w.empty_tag("cx:majorGridlines", &[]);
    w.empty_tag("cx:tickLabels", &[]);
    w.end_tag("cx:axis");

    w.end_tag("cx:plotArea");
    w.end_tag("cx:chart");
    w.end_tag("cx:chartSpace");
    w.into_bytes()
}

fn write_map_xml(chart: &MapChart) -> Vec<u8> {
    let mut w = XmlWriter::new();
    chartex_header(&mut w);

    // Chart Data
    w.start_tag("cx:chartData", &[]);
    w.start_tag("cx:data", &[("id", "0")]);

    // String dimension (geographic categories)
    w.start_tag("cx:strDim", &[("type", "cat")]);
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

    // Numeric dimension (colorVal — map charts use color values)
    w.start_tag("cx:numDim", &[("type", "colorVal")]);
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

    // Chart
    w.start_tag("cx:chart", &[]);
    write_chartex_title(&mut w, &chart.title);

    w.start_tag("cx:plotArea", &[]);
    w.start_tag("cx:plotAreaRegion", &[]);

    w.start_tag("cx:series", &[
        ("layoutId", "regionMap"),
        ("uniqueId", "{00000000-0000-0000-0000-000000000007}"),
    ]);

    if let Some(ref name) = chart.series_name {
        w.start_tag("cx:tx", &[]);
        w.start_tag("cx:txData", &[]);
        w.text_element("cx:v", &[], name);
        w.end_tag("cx:txData");
        w.end_tag("cx:tx");
    }

    w.empty_tag("cx:dataId", &[("val", "0")]);

    // Geography layout — tells Excel to use Bing Maps
    w.start_tag("cx:layoutPr", &[]);
    w.start_tag("cx:geography", &[
        ("cultureLanguage", "en-US"),
        ("cultureRegion", "US"),
        ("attribution", "Powered by Bing"),
    ]);
    // No geoCache — Excel will fetch from Bing on open
    w.end_tag("cx:geography");
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
