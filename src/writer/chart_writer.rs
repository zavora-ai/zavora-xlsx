use crate::features::chart::{Chart, ChartType, LegendPosition};
use crate::xml::xml_writer::XmlWriter;

pub fn write_chart_xml(chart: &Chart, _chart_id: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("c:chartSpace", &[
        ("xmlns:c", "http://schemas.openxmlformats.org/drawingml/2006/chart"),
        ("xmlns:a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
    ]);
    w.empty_tag("c:lang", &[("val", "en-US")]);
    w.start_tag("c:chart", &[]);

    // Title
    if let Some(ref title) = chart.title {
        write_title(&mut w, title);
    }
    w.empty_tag("c:autoTitleDeleted", &[("val", if chart.title.is_some() { "0" } else { "1" })]);

    // Plot area
    w.start_tag("c:plotArea", &[]);
    w.empty_tag("c:layout", &[]);

    let has_secondary = chart.series.iter().any(|s| s.secondary_axis);
    let is_combo = chart.series.iter().any(|s| s.chart_type_override.is_some());

    if is_combo {
        write_combo_chart(&mut w, chart, has_secondary);
    } else {
        write_single_chart_type(&mut w, chart, has_secondary);
    }

    // Axes (not for pie/doughnut)
    let base_type = chart.chart_type;
    if !matches!(base_type, ChartType::Pie | ChartType::Doughnut) {
        write_axes(&mut w, chart, has_secondary);
    }

    w.end_tag("c:plotArea");

    // Legend
    if !matches!(chart.legend_pos, LegendPosition::None) {
        let pos = match chart.legend_pos {
            LegendPosition::Top => "t", LegendPosition::Bottom => "b",
            LegendPosition::Left => "l", LegendPosition::Right => "r",
            LegendPosition::None => unreachable!(),
        };
        w.start_tag("c:legend", &[]);
        w.empty_tag("c:legendPos", &[("val", pos)]);
        w.empty_tag("c:overlay", &[("val", "0")]);
        w.end_tag("c:legend");
    }

    w.empty_tag("c:plotVisOnly", &[("val", "1")]);
    w.end_tag("c:chart");
    w.start_tag("c:printSettings", &[]);
    w.empty_tag("c:headerFooter", &[]);
    w.empty_tag("c:pageMargins", &[("b", "0.75"), ("l", "0.7"), ("r", "0.7"), ("t", "0.75"), ("header", "0.3"), ("footer", "0.3")]);
    w.empty_tag("c:pageSetup", &[]);
    w.end_tag("c:printSettings");
    w.end_tag("c:chartSpace");
    w.into_bytes()
}

fn write_single_chart_type(w: &mut XmlWriter, chart: &Chart, has_secondary: bool) {
    let ct = chart.chart_type;
    // Primary series
    let primary: Vec<(usize, &crate::features::chart::ChartSeries)> =
        chart.series.iter().enumerate().filter(|(_, s)| !s.secondary_axis).collect();
    if !primary.is_empty() {
        write_chart_type_block(w, ct, &primary, "1", "2");
    }
    // Secondary series (same chart type, different axes)
    if has_secondary {
        let secondary: Vec<(usize, &crate::features::chart::ChartSeries)> =
            chart.series.iter().enumerate().filter(|(_, s)| s.secondary_axis).collect();
        if !secondary.is_empty() {
            write_chart_type_block(w, ct, &secondary, "3", "4");
        }
    }
}

fn write_combo_chart(w: &mut XmlWriter, chart: &Chart, has_secondary: bool) {
    // Group series by effective chart type
    let mut groups: Vec<(ChartType, Vec<(usize, &crate::features::chart::ChartSeries)>)> = Vec::new();
    for (i, s) in chart.series.iter().enumerate() {
        let ct = s.chart_type_override.unwrap_or(chart.chart_type);
        if let Some(g) = groups.iter_mut().find(|(t, _)| *t == ct) {
            g.1.push((i, s));
        } else {
            groups.push((ct, vec![(i, s)]));
        }
    }
    for (ct, series_list) in &groups {
        let any_secondary = series_list.iter().any(|(_, s)| s.secondary_axis);
        let (cat_ax, val_ax) = if any_secondary { ("3", "4") } else { ("1", "2") };
        write_chart_type_block(w, *ct, series_list, cat_ax, val_ax);
    }
    // If combo has secondary axis series but no explicit secondary group, axes still needed
    let _ = has_secondary;
}

fn write_chart_type_block(w: &mut XmlWriter, ct: ChartType, series: &[(usize, &crate::features::chart::ChartSeries)], cat_ax_id: &str, val_ax_id: &str) {
    let tag = chart_type_tag(ct);
    w.start_tag(tag, &[]);

    match ct {
        ChartType::Bar => { w.empty_tag("c:barDir", &[("val", "bar")]); w.empty_tag("c:grouping", &[("val", "clustered")]); }
        ChartType::Column => { w.empty_tag("c:barDir", &[("val", "col")]); w.empty_tag("c:grouping", &[("val", "clustered")]); }
        ChartType::Line | ChartType::Area => { w.empty_tag("c:grouping", &[("val", "standard")]); }
        ChartType::Scatter => { w.empty_tag("c:scatterStyle", &[("val", "lineMarker")]); }
        ChartType::Radar => { w.empty_tag("c:radarStyle", &[("val", "marker")]); }
        _ => {}
    }

    for &(idx, s) in series {
        write_series(w, idx, s, ct);
    }

    if !matches!(ct, ChartType::Pie | ChartType::Doughnut) {
        w.empty_tag("c:axId", &[("val", cat_ax_id)]);
        w.empty_tag("c:axId", &[("val", val_ax_id)]);
    }

    w.end_tag(tag);
}

fn write_series(w: &mut XmlWriter, idx: usize, s: &crate::features::chart::ChartSeries, ct: ChartType) {
    let idx_s = idx.to_string();
    w.start_tag("c:ser", &[]);
    w.empty_tag("c:idx", &[("val", &idx_s)]);
    w.empty_tag("c:order", &[("val", &idx_s)]);
    if let Some(ref name) = s.name {
        w.start_tag("c:tx", &[]);
        w.text_element("c:v", &[], name);
        w.end_tag("c:tx");
    }

    // Data labels (per-series)
    if s.data_labels {
        w.start_tag("c:dLbls", &[]);
        if let Some(pos) = s.data_label_pos {
            w.empty_tag("c:dLblPos", &[("val", pos.xml_str())]);
        }
        w.empty_tag("c:showVal", &[("val", "1")]);
        w.empty_tag("c:showCatName", &[("val", "0")]);
        w.empty_tag("c:showSerName", &[("val", "0")]);
        w.empty_tag("c:showPercent", &[("val", "0")]);
        w.end_tag("c:dLbls");
    }

    // Trendline
    if let Some(ref tl) = s.trendline {
        write_trendline(w, tl);
    }

    // Categories
    if let Some(ref cats) = s.categories {
        let cat_tag = if matches!(ct, ChartType::Scatter) { "c:xVal" } else { "c:cat" };
        w.start_tag(cat_tag, &[]);
        w.start_tag("c:strRef", &[]);
        w.text_element("c:f", &[], cats);
        w.end_tag("c:strRef");
        w.end_tag(cat_tag);
    }

    // Values
    let val_tag = if matches!(ct, ChartType::Scatter) { "c:yVal" } else { "c:val" };
    w.start_tag(val_tag, &[]);
    w.start_tag("c:numRef", &[]);
    w.text_element("c:f", &[], &s.values);
    w.end_tag("c:numRef");
    w.end_tag(val_tag);

    w.end_tag("c:ser");
}

fn write_trendline(w: &mut XmlWriter, tl: &crate::features::chart::TrendlineType) {
    use crate::features::chart::TrendlineType;
    w.start_tag("c:trendline", &[]);
    match tl {
        TrendlineType::Linear => w.empty_tag("c:trendlineType", &[("val", "linear")]),
        TrendlineType::Exponential => w.empty_tag("c:trendlineType", &[("val", "exp")]),
        TrendlineType::Power => w.empty_tag("c:trendlineType", &[("val", "power")]),
        TrendlineType::Logarithmic => w.empty_tag("c:trendlineType", &[("val", "log")]),
        TrendlineType::Polynomial(order) => {
            let o = order.to_string();
            w.empty_tag("c:trendlineType", &[("val", "poly")]);
            w.empty_tag("c:order", &[("val", &o)]);
        }
        TrendlineType::MovingAverage(period) => {
            let p = period.to_string();
            w.empty_tag("c:trendlineType", &[("val", "movingAvg")]);
            w.empty_tag("c:period", &[("val", &p)]);
        }
    }
    w.end_tag("c:trendline");
}

fn write_axes(w: &mut XmlWriter, chart: &Chart, has_secondary: bool) {
    // Primary category axis (axId=1, crossAx=2)
    w.start_tag("c:catAx", &[]);
    w.empty_tag("c:axId", &[("val", "1")]);
    w.start_tag("c:scaling", &[]); w.empty_tag("c:orientation", &[("val", "minMax")]); w.end_tag("c:scaling");
    w.empty_tag("c:delete", &[("val", "0")]);
    w.empty_tag("c:axPos", &[("val", "b")]);
    if let Some(ref name) = chart.x_axis_name { write_axis_title(w, name); }
    w.empty_tag("c:crossAx", &[("val", "2")]);
    w.end_tag("c:catAx");

    // Primary value axis (axId=2, crossAx=1)
    w.start_tag("c:valAx", &[]);
    w.empty_tag("c:axId", &[("val", "2")]);
    w.start_tag("c:scaling", &[]); w.empty_tag("c:orientation", &[("val", "minMax")]); w.end_tag("c:scaling");
    w.empty_tag("c:delete", &[("val", "0")]);
    w.empty_tag("c:axPos", &[("val", "l")]);
    if let Some(ref name) = chart.y_axis_name { write_axis_title(w, name); }
    w.empty_tag("c:crossAx", &[("val", "1")]);
    w.end_tag("c:valAx");

    if has_secondary {
        // Secondary category axis (axId=3, crossAx=4) — hidden, just for axis pairing
        w.start_tag("c:catAx", &[]);
        w.empty_tag("c:axId", &[("val", "3")]);
        w.start_tag("c:scaling", &[]); w.empty_tag("c:orientation", &[("val", "minMax")]); w.end_tag("c:scaling");
        w.empty_tag("c:delete", &[("val", "1")]);
        w.empty_tag("c:axPos", &[("val", "b")]);
        w.empty_tag("c:crossAx", &[("val", "4")]);
        w.end_tag("c:catAx");

        // Secondary value axis (axId=4, crossAx=3) — right side
        w.start_tag("c:valAx", &[]);
        w.empty_tag("c:axId", &[("val", "4")]);
        w.start_tag("c:scaling", &[]); w.empty_tag("c:orientation", &[("val", "minMax")]); w.end_tag("c:scaling");
        w.empty_tag("c:delete", &[("val", "0")]);
        w.empty_tag("c:axPos", &[("val", "r")]);
        if let Some(ref name) = chart.y2_axis_name { write_axis_title(w, name); }
        w.empty_tag("c:crossAx", &[("val", "3")]);
        w.empty_tag("c:crosses", &[("val", "max")]);
        w.end_tag("c:valAx");
    }
}

fn chart_type_tag(ct: ChartType) -> &'static str {
    match ct {
        ChartType::Bar => "c:barChart", ChartType::Column => "c:barChart",
        ChartType::Line => "c:lineChart", ChartType::Pie => "c:pieChart",
        ChartType::Scatter => "c:scatterChart", ChartType::Area => "c:areaChart",
        ChartType::Doughnut => "c:doughnutChart", ChartType::Radar => "c:radarChart",
    }
}

fn write_title(w: &mut XmlWriter, text: &str) {
    w.start_tag("c:title", &[]);
    w.start_tag("c:tx", &[]);
    w.start_tag("c:rich", &[]);
    w.empty_tag("a:bodyPr", &[]);
    w.empty_tag("a:lstStyle", &[]);
    w.start_tag("a:p", &[]);
    w.start_tag("a:r", &[]);
    w.text_element("a:t", &[], text);
    w.end_tag("a:r");
    w.end_tag("a:p");
    w.end_tag("c:rich");
    w.end_tag("c:tx");
    w.empty_tag("c:overlay", &[("val", "0")]);
    w.end_tag("c:title");
}

fn write_axis_title(w: &mut XmlWriter, title: &str) {
    w.start_tag("c:title", &[]);
    w.start_tag("c:tx", &[]);
    w.start_tag("c:rich", &[]);
    w.empty_tag("a:bodyPr", &[]);
    w.empty_tag("a:lstStyle", &[]);
    w.start_tag("a:p", &[]);
    w.start_tag("a:r", &[]);
    w.text_element("a:t", &[], title);
    w.end_tag("a:r");
    w.end_tag("a:p");
    w.end_tag("c:rich");
    w.end_tag("c:tx");
    w.end_tag("c:title");
}
