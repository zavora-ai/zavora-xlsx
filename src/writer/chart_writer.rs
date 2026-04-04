use crate::features::chart::{Chart, ChartType, LegendPosition};
use crate::xml::xml_writer::XmlWriter;

pub fn write_chart_xml(chart: &Chart, chart_id: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("c:chartSpace", &[
        ("xmlns:c", "http://schemas.openxmlformats.org/drawingml/2006/chart"),
        ("xmlns:a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
    ]);
    w.start_tag("c:chart", &[]);

    // Title
    if let Some(ref title) = chart.title {
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
        w.empty_tag("c:overlay", &[("val", "0")]);
        w.end_tag("c:title");
    }

    w.empty_tag("c:autoTitleDeleted", &[("val", if chart.title.is_some() { "0" } else { "1" })]);

    // Plot area
    w.start_tag("c:plotArea", &[]);
    w.empty_tag("c:layout", &[]);

    write_chart_type_xml(&mut w, chart);

    // Axes (not for pie/doughnut)
    if !matches!(chart.chart_type, ChartType::Pie | ChartType::Doughnut) {
        // Category axis
        w.start_tag("c:catAx", &[]);
        w.empty_tag("c:axId", &[("val", "1")]);
        w.start_tag("c:scaling", &[]); w.empty_tag("c:orientation", &[("val", "minMax")]); w.end_tag("c:scaling");
        w.empty_tag("c:delete", &[("val", "0")]);
        w.empty_tag("c:axPos", &[("val", "b")]);
        if let Some(ref name) = chart.x_axis_name { write_axis_title(&mut w, name); }
        w.empty_tag("c:crossAx", &[("val", "2")]);
        w.end_tag("c:catAx");

        // Value axis
        w.start_tag("c:valAx", &[]);
        w.empty_tag("c:axId", &[("val", "2")]);
        w.start_tag("c:scaling", &[]); w.empty_tag("c:orientation", &[("val", "minMax")]); w.end_tag("c:scaling");
        w.empty_tag("c:delete", &[("val", "0")]);
        w.empty_tag("c:axPos", &[("val", "l")]);
        if let Some(ref name) = chart.y_axis_name { write_axis_title(&mut w, name); }
        w.empty_tag("c:crossAx", &[("val", "1")]);
        w.end_tag("c:valAx");
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
    w.end_tag("c:chartSpace");
    w.into_bytes()
}

fn write_chart_type_xml(w: &mut XmlWriter, chart: &Chart) {
    let tag = match chart.chart_type {
        ChartType::Bar => "c:barChart",
        ChartType::Column => "c:barChart",
        ChartType::Line => "c:lineChart",
        ChartType::Pie => "c:pieChart",
        ChartType::Scatter => "c:scatterChart",
        ChartType::Area => "c:areaChart",
        ChartType::Doughnut => "c:doughnutChart",
        ChartType::Radar => "c:radarChart",
    };

    w.start_tag(tag, &[]);

    // Bar/Column direction
    if matches!(chart.chart_type, ChartType::Bar) {
        w.empty_tag("c:barDir", &[("val", "bar")]);
        w.empty_tag("c:grouping", &[("val", "clustered")]);
    } else if matches!(chart.chart_type, ChartType::Column) {
        w.empty_tag("c:barDir", &[("val", "col")]);
        w.empty_tag("c:grouping", &[("val", "clustered")]);
    } else if matches!(chart.chart_type, ChartType::Line | ChartType::Area) {
        w.empty_tag("c:grouping", &[("val", "standard")]);
    } else if matches!(chart.chart_type, ChartType::Scatter) {
        w.empty_tag("c:scatterStyle", &[("val", "lineMarker")]);
    } else if matches!(chart.chart_type, ChartType::Radar) {
        w.empty_tag("c:radarStyle", &[("val", "marker")]);
    }

    // Series
    for (i, series) in chart.series.iter().enumerate() {
        let idx = i.to_string();
        w.start_tag("c:ser", &[]);
        w.empty_tag("c:idx", &[("val", &idx)]);
        w.empty_tag("c:order", &[("val", &idx)]);
        if let Some(ref name) = series.name {
            w.start_tag("c:tx", &[]);
            w.text_element("c:v", &[], name);
            w.end_tag("c:tx");
        }
        if let Some(ref cats) = series.categories {
            let cat_tag = if matches!(chart.chart_type, ChartType::Scatter) { "c:xVal" } else { "c:cat" };
            w.start_tag(cat_tag, &[]);
            w.start_tag("c:strRef", &[]);
            w.text_element("c:f", &[], cats);
            w.end_tag("c:strRef");
            w.end_tag(cat_tag);
        }
        let val_tag = if matches!(chart.chart_type, ChartType::Scatter) { "c:yVal" } else { "c:val" };
        w.start_tag(val_tag, &[]);
        w.start_tag("c:numRef", &[]);
        w.text_element("c:f", &[], &series.values);
        w.end_tag("c:numRef");
        w.end_tag(val_tag);
        w.end_tag("c:ser");
    }

    // Axis IDs (not for pie/doughnut)
    if !matches!(chart.chart_type, ChartType::Pie | ChartType::Doughnut) {
        w.empty_tag("c:axId", &[("val", "1")]);
        w.empty_tag("c:axId", &[("val", "2")]);
    }

    w.end_tag(tag);
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
