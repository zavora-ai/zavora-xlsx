//! Tests for Task 37: Plot Area and Series Formatting
//!
//! Verifies that plot area formatting, series line styling, and error bars
//! are correctly serialized in chart XML.

use zavora_xlsx::{
    Chart, ChartType, DashStyle, ErrorBar, ErrorBarType, ErrorBarValueType, PlotAreaFormat,
};

/// Helper: create a chart and return the generated chart XML as a string.
fn chart_xml(chart: &Chart) -> String {
    let bytes = zavora_xlsx::writer::chart_writer::write_chart_xml(chart, 1);
    String::from_utf8(bytes).unwrap()
}

#[test]
fn test_plot_area_fill_and_border() {
    let mut chart = Chart::new(ChartType::Column);
    let s = chart.add_series();
    s.set_values("Sheet1!$A$1:$A$5");

    let mut pf = PlotAreaFormat::new();
    pf.set_fill([245, 245, 245]);
    pf.set_border([180, 180, 180]);
    chart.set_plot_area_format(pf);

    let xml = chart_xml(&chart);

    // Plot area should have spPr with solid fill
    assert!(xml.contains("<c:spPr>"), "plot area should have spPr");
    assert!(
        xml.contains("<a:solidFill><a:srgbClr val=\"F5F5F5\"/></a:solidFill>"),
        "plot area fill should be F5F5F5"
    );
    // Border
    assert!(
        xml.contains("<a:ln><a:solidFill><a:srgbClr val=\"B4B4B4\"/></a:solidFill></a:ln>"),
        "plot area border should be B4B4B4"
    );
}

#[test]
fn test_plot_area_gradient() {
    let mut chart = Chart::new(ChartType::Line);
    let s = chart.add_series();
    s.set_values("Sheet1!$A$1:$A$5");

    let mut pf = PlotAreaFormat::new();
    pf.set_gradient(vec![([255, 255, 255], 0.0), ([200, 200, 200], 1.0)]);
    chart.set_plot_area_format(pf);

    let xml = chart_xml(&chart);

    assert!(xml.contains("<a:gradFill>"), "should have gradient fill");
    assert!(xml.contains("<a:gsLst>"), "should have gradient stop list");
    assert!(
        xml.contains("<a:gs pos=\"0\"><a:srgbClr val=\"FFFFFF\"/></a:gs>"),
        "first gradient stop"
    );
    assert!(
        xml.contains("<a:gs pos=\"100000\"><a:srgbClr val=\"C8C8C8\"/></a:gs>"),
        "second gradient stop"
    );
}

#[test]
fn test_series_line_width_and_dash_style() {
    let mut chart = Chart::new(ChartType::Line);
    let s = chart.add_series();
    s.set_values("Sheet1!$A$1:$A$5")
        .set_line_width(2.5)
        .set_dash_style(DashStyle::DashDot);

    let xml = chart_xml(&chart);

    // Line width: 2.5pt = 2.5 * 12700 = 31750 EMU
    assert!(
        xml.contains("w=\"31750\""),
        "line width should be 31750 EMU"
    );
    assert!(
        xml.contains("<a:prstDash val=\"dashDot\"/>"),
        "dash style should be dashDot"
    );
}

#[test]
fn test_series_gradient() {
    let mut chart = Chart::new(ChartType::Column);
    let s = chart.add_series();
    s.set_values("Sheet1!$A$1:$A$5")
        .set_gradient(vec![([0, 100, 200], 0.0), ([0, 200, 255], 1.0)]);

    let xml = chart_xml(&chart);

    // Series should have gradient fill in spPr
    assert!(
        xml.contains("<a:gradFill>"),
        "series should have gradient fill"
    );
    assert!(
        xml.contains("<a:gs pos=\"0\"><a:srgbClr val=\"0064C8\"/></a:gs>"),
        "first series gradient stop"
    );
    assert!(
        xml.contains("<a:gs pos=\"100000\"><a:srgbClr val=\"00C8FF\"/></a:gs>"),
        "second series gradient stop"
    );
}

#[test]
fn test_error_bars_percentage() {
    let mut chart = Chart::new(ChartType::Column);
    let s = chart.add_series();
    s.set_values("Sheet1!$A$1:$A$5")
        .set_error_bars(ErrorBar::new(
            ErrorBarType::Both,
            ErrorBarValueType::Percentage,
            5.0,
        ));

    let xml = chart_xml(&chart);

    assert!(
        xml.contains("<c:errBars>"),
        "should have error bars element"
    );
    assert!(
        xml.contains("<c:errBarType val=\"both\"/>"),
        "error bar type should be both"
    );
    assert!(
        xml.contains("<c:errValType val=\"percentage\"/>"),
        "error value type should be percentage"
    );
    assert!(
        xml.contains("<c:val val=\"5\"/>"),
        "error bar value should be 5"
    );
    assert!(
        xml.contains("<c:noEndCap val=\"0\"/>"),
        "should have noEndCap"
    );
}

#[test]
fn test_error_bars_fixed_value_plus() {
    let mut chart = Chart::new(ChartType::Line);
    let s = chart.add_series();
    s.set_values("Sheet1!$A$1:$A$5")
        .set_error_bars(ErrorBar::new(
            ErrorBarType::Plus,
            ErrorBarValueType::FixedValue,
            3.5,
        ));

    let xml = chart_xml(&chart);

    assert!(
        xml.contains("<c:errBarType val=\"plus\"/>"),
        "error bar type should be plus"
    );
    assert!(
        xml.contains("<c:errValType val=\"fixedVal\"/>"),
        "error value type should be fixedVal"
    );
    assert!(
        xml.contains("<c:val val=\"3.5\"/>"),
        "error bar value should be 3.5"
    );
}

#[test]
fn test_error_bars_standard_error() {
    let mut chart = Chart::new(ChartType::Column);
    let s = chart.add_series();
    s.set_values("Sheet1!$A$1:$A$5")
        .set_error_bars(ErrorBar::new(
            ErrorBarType::Minus,
            ErrorBarValueType::StandardError,
            0.0,
        ));

    let xml = chart_xml(&chart);

    assert!(
        xml.contains("<c:errBarType val=\"minus\"/>"),
        "error bar type should be minus"
    );
    assert!(
        xml.contains("<c:errValType val=\"stdErr\"/>"),
        "error value type should be stdErr"
    );
    // StandardError should NOT have a <c:val> element
    assert!(
        !xml.contains("<c:val val="),
        "standard error should not have val element"
    );
}

#[test]
fn test_dash_style_variants() {
    let styles = [
        (DashStyle::Solid, "solid"),
        (DashStyle::Dash, "dash"),
        (DashStyle::Dot, "dot"),
        (DashStyle::DashDot, "dashDot"),
        (DashStyle::LongDash, "lgDash"),
        (DashStyle::LongDashDot, "lgDashDot"),
    ];

    for (style, expected) in styles {
        let mut chart = Chart::new(ChartType::Line);
        let s = chart.add_series();
        s.set_values("Sheet1!$A$1:$A$5").set_dash_style(style);

        let xml = chart_xml(&chart);
        let expected_xml = format!("<a:prstDash val=\"{}\"/>", expected);
        assert!(
            xml.contains(&expected_xml),
            "dash style {:?} should produce {}",
            style,
            expected_xml
        );
    }
}

#[test]
fn test_combined_plot_area_and_series_formatting() {
    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Formatted Chart");

    // Plot area
    let mut pf = PlotAreaFormat::new();
    pf.set_fill([240, 248, 255]);
    chart.set_plot_area_format(pf);

    // Series with line width and error bars
    let s = chart.add_series();
    s.set_values("Sheet1!$B$1:$B$5")
        .set_categories("Sheet1!$A$1:$A$5")
        .set_name("Revenue")
        .set_color((70, 130, 180))
        .set_line_width(1.5)
        .set_error_bars(ErrorBar::new(
            ErrorBarType::Both,
            ErrorBarValueType::StandardDeviation,
            1.0,
        ));

    let xml = chart_xml(&chart);

    // Plot area fill
    assert!(xml.contains("F0F8FF"), "plot area fill color");
    // Series line width: 1.5pt = 19050 EMU
    assert!(xml.contains("w=\"19050\""), "series line width");
    // Error bars
    assert!(xml.contains("<c:errBars>"), "error bars present");
    assert!(
        xml.contains("<c:errValType val=\"stdDev\"/>"),
        "stdDev type"
    );
}
