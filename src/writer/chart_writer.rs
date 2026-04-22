use crate::features::chart::{Chart, ChartType, LegendPosition, TickMark};
use crate::xml::xml_writer::XmlWriter;

pub fn write_chart_xml(chart: &Chart, _chart_id: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "c:chartSpace",
        &[
            (
                "xmlns:c",
                "http://schemas.openxmlformats.org/drawingml/2006/chart",
            ),
            (
                "xmlns:a",
                "http://schemas.openxmlformats.org/drawingml/2006/main",
            ),
            (
                "xmlns:r",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
            ),
        ],
    );
    w.empty_tag("c:lang", &[("val", "en-US")]);

    // Pivot chart source — must come before <c:chart>
    if let Some(ref ps) = chart.pivot_source {
        let name = format!("{}!{}", ps.sheet_name, ps.pivot_table_name);
        w.start_tag("c:pivotSource", &[]);
        w.start_tag("c:name", &[]);
        w.text(&name);
        w.end_tag("c:name");
        w.empty_tag("c:fmtId", &[("val", "0")]);
        w.end_tag("c:pivotSource");
    }

    w.start_tag("c:chart", &[]);

    // Title
    if let Some(ref title) = chart.title {
        write_title(&mut w, title);
    }
    w.empty_tag(
        "c:autoTitleDeleted",
        &[("val", if chart.title.is_some() { "0" } else { "1" })],
    );

    // 3D view settings
    if let Some(ref v) = chart.view3d {
        w.start_tag("c:view3D", &[]);
        w.empty_tag("c:rotX", &[("val", &v.rot_x.to_string())]);
        w.empty_tag("c:rotY", &[("val", &v.rot_y.to_string())]);
        w.empty_tag("c:perspective", &[("val", &v.perspective.to_string())]);
        w.empty_tag(
            "c:rAngAx",
            &[("val", if v.right_angle_axes { "1" } else { "0" })],
        );
        w.end_tag("c:view3D");
    } else if matches!(
        chart.chart_type,
        ChartType::Column3D
            | ChartType::Bar3D
            | ChartType::Line3D
            | ChartType::Pie3D
            | ChartType::Area3D
            | ChartType::Surface
            | ChartType::WireframeSurface
    ) {
        // Default 3D view for 3D chart types
        w.start_tag("c:view3D", &[]);
        w.empty_tag("c:rotX", &[("val", "15")]);
        w.empty_tag("c:rotY", &[("val", "20")]);
        if matches!(
            chart.chart_type,
            ChartType::Surface | ChartType::WireframeSurface
        ) {
            w.empty_tag("c:rAngAx", &[("val", "0")]);
        } else {
            w.empty_tag("c:perspective", &[("val", "30")]);
            w.empty_tag("c:rAngAx", &[("val", "1")]);
        }
        w.end_tag("c:view3D");
    }

    // Floor, side wall, back wall for 3D and surface charts
    if matches!(
        chart.chart_type,
        ChartType::Column3D
            | ChartType::Bar3D
            | ChartType::Line3D
            | ChartType::Area3D
            | ChartType::Surface
            | ChartType::WireframeSurface
    ) {
        for tag in &["c:floor", "c:sideWall", "c:backWall"] {
            w.start_tag(tag, &[]);
            w.empty_tag("c:thickness", &[("val", "0")]);
            w.end_tag(tag);
        }
    }

    // Pivot format entries — one per series with Office theme accent colors
    if chart.pivot_source.is_some() {
        write_pivot_fmts(&mut w, chart.series.len().max(1));
    }

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
    if !matches!(
        base_type,
        ChartType::Pie | ChartType::Doughnut | ChartType::Pie3D
    ) {
        write_axes(&mut w, chart, has_secondary);
    }

    // Plot area formatting (Task 37)
    if let Some(ref pf) = chart.plot_area_format {
        write_plot_area_sppr(&mut w, pf);
    }

    w.end_tag("c:plotArea");

    // Data table
    if chart.show_data_table {
        w.start_tag("c:dTable", &[]);
        w.empty_tag("c:showHorzBorder", &[("val", "1")]);
        w.empty_tag("c:showVertBorder", &[("val", "1")]);
        w.empty_tag("c:showOutline", &[("val", "1")]);
        w.empty_tag("c:showKeys", &[("val", "1")]);
        w.end_tag("c:dTable");
    }

    // Legend
    if !matches!(chart.legend_pos, LegendPosition::None) {
        let pos = match chart.legend_pos {
            LegendPosition::Top => "t",
            LegendPosition::Bottom => "b",
            LegendPosition::Left => "l",
            LegendPosition::Right => "r",
            LegendPosition::None => unreachable!(),
        };
        w.start_tag("c:legend", &[]);
        w.empty_tag("c:legendPos", &[("val", pos)]);
        w.empty_tag("c:overlay", &[("val", "0")]);
        w.end_tag("c:legend");
    }

    w.empty_tag("c:plotVisOnly", &[("val", "1")]);

    w.end_tag("c:chart");

    // Chart style (child of c:chartSpace, after c:chart)
    if let Some(style) = chart.style {
        w.empty_tag("c:style", &[("val", &style.to_string())]);
    }
    w.start_tag("c:printSettings", &[]);
    w.empty_tag("c:headerFooter", &[]);
    w.empty_tag(
        "c:pageMargins",
        &[
            ("b", "0.75"),
            ("l", "0.7"),
            ("r", "0.7"),
            ("t", "0.75"),
            ("header", "0.3"),
            ("footer", "0.3"),
        ],
    );
    w.empty_tag("c:pageSetup", &[]);
    w.end_tag("c:printSettings");

    // Pivot chart extensions — drop zones and expand/collapse buttons
    if let Some(ref ps) = chart.pivot_source {
        write_pivot_extensions(&mut w, ps);
    }

    w.end_tag("c:chartSpace");
    w.into_bytes()
}

fn write_single_chart_type(w: &mut XmlWriter, chart: &Chart, has_secondary: bool) {
    let ct = chart.chart_type;

    // Pivot charts: emit series from pre-computed pivot data with cached values
    if chart.pivot_source.is_some() && !chart.pivot_series.is_empty() {
        let tag = chart_type_tag(ct);
        w.start_tag(tag, &[]);
        if matches!(ct, ChartType::Bar) {
            w.empty_tag("c:barDir", &[("val", "bar")]);
        } else if matches!(ct, ChartType::Column) {
            w.empty_tag("c:barDir", &[("val", "col")]);
        }
        if matches!(
            ct,
            ChartType::Bar | ChartType::Column | ChartType::Line | ChartType::Area
        ) {
            w.empty_tag("c:grouping", &[("val", "clustered")]);
        }
        w.empty_tag("c:varyColors", &[("val", "0")]);
        for (si, ps) in chart.pivot_series.iter().enumerate() {
            write_pivot_series(w, si, ps);
        }
        if matches!(ct, ChartType::Bar | ChartType::Column) {
            w.empty_tag("c:gapWidth", &[("val", "219")]);
            w.empty_tag("c:overlap", &[("val", "-27")]);
        }
        w.empty_tag("c:axId", &[("val", "111111111")]);
        w.empty_tag("c:axId", &[("val", "222222222")]);
        w.end_tag(tag);
        return;
    }

    // Pivot charts without pre-computed data: empty chart type block
    if chart.pivot_source.is_some() {
        let tag = chart_type_tag(ct);
        w.start_tag(tag, &[]);
        if matches!(ct, ChartType::Bar) {
            w.empty_tag("c:barDir", &[("val", "bar")]);
        } else if matches!(ct, ChartType::Column) {
            w.empty_tag("c:barDir", &[("val", "col")]);
        }
        if matches!(
            ct,
            ChartType::Bar | ChartType::Column | ChartType::Line | ChartType::Area
        ) {
            w.empty_tag("c:grouping", &[("val", "clustered")]);
        }
        w.empty_tag("c:varyColors", &[("val", "0")]);
        if matches!(ct, ChartType::Bar | ChartType::Column) {
            w.empty_tag("c:gapWidth", &[("val", "219")]);
            w.empty_tag("c:overlap", &[("val", "-27")]);
        }
        w.empty_tag("c:axId", &[("val", "111111111")]);
        w.empty_tag("c:axId", &[("val", "222222222")]);
        w.end_tag(tag);
        return;
    }

    // Primary series
    let primary: Vec<(usize, &crate::features::chart::ChartSeries)> = chart
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.secondary_axis)
        .collect();
    if !primary.is_empty() {
        write_chart_type_block_with_accessories(
            w,
            ct,
            &primary,
            "111111111",
            "222222222",
            chart.drop_lines,
            chart.high_low_lines,
        );
    }
    // Secondary series (same chart type, different axes)
    if has_secondary {
        let secondary: Vec<(usize, &crate::features::chart::ChartSeries)> = chart
            .series
            .iter()
            .enumerate()
            .filter(|(_, s)| s.secondary_axis)
            .collect();
        if !secondary.is_empty() {
            write_chart_type_block_with_accessories(
                w,
                ct,
                &secondary,
                "333333333",
                "444444444",
                chart.drop_lines,
                chart.high_low_lines,
            );
        }
    }
}

fn write_combo_chart(w: &mut XmlWriter, chart: &Chart, has_secondary: bool) {
    // Group series by effective chart type
    let mut groups: Vec<(
        ChartType,
        Vec<(usize, &crate::features::chart::ChartSeries)>,
    )> = Vec::new();
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
        let (cat_ax, val_ax) = if any_secondary {
            ("333333333", "444444444")
        } else {
            ("111111111", "222222222")
        };
        write_chart_type_block_with_accessories(
            w,
            *ct,
            series_list,
            cat_ax,
            val_ax,
            chart.drop_lines,
            chart.high_low_lines,
        );
    }
    // If combo has secondary axis series but no explicit secondary group, axes still needed
    let _ = has_secondary;
}

fn write_chart_type_block_with_accessories(
    w: &mut XmlWriter,
    ct: ChartType,
    series: &[(usize, &crate::features::chart::ChartSeries)],
    cat_ax_id: &str,
    val_ax_id: &str,
    drop_lines: bool,
    hi_low_lines: bool,
) {
    let tag = chart_type_tag(ct);
    w.start_tag(tag, &[]);

    match ct {
        ChartType::Bar => {
            w.empty_tag("c:barDir", &[("val", "bar")]);
            w.empty_tag("c:grouping", &[("val", "clustered")]);
        }
        ChartType::Column => {
            w.empty_tag("c:barDir", &[("val", "col")]);
            w.empty_tag("c:grouping", &[("val", "clustered")]);
        }
        ChartType::Bar3D => {
            w.empty_tag("c:barDir", &[("val", "bar")]);
            w.empty_tag("c:grouping", &[("val", "clustered")]);
        }
        ChartType::Column3D => {
            w.empty_tag("c:barDir", &[("val", "col")]);
            w.empty_tag("c:grouping", &[("val", "clustered")]);
        }
        ChartType::Line | ChartType::Area | ChartType::Line3D | ChartType::Area3D => {
            w.empty_tag("c:grouping", &[("val", "standard")]);
        }
        ChartType::Scatter => {
            w.empty_tag("c:scatterStyle", &[("val", "lineMarker")]);
        }
        ChartType::Radar => {
            w.empty_tag("c:radarStyle", &[("val", "marker")]);
        }
        ChartType::Bubble => {
            w.empty_tag("c:varyColors", &[("val", "0")]);
        }
        ChartType::Surface | ChartType::WireframeSurface => {
            let wf = if matches!(ct, ChartType::WireframeSurface) {
                "1"
            } else {
                "0"
            };
            w.empty_tag("c:wireframe", &[("val", wf)]);
        }
        _ => {}
    }
    // varyColors — NOT for surface or bubble (bubble already emitted above)
    if !matches!(
        ct,
        ChartType::Bubble | ChartType::Surface | ChartType::WireframeSurface
    ) {
        w.empty_tag("c:varyColors", &[("val", "0")]);
    }

    for &(idx, s) in series {
        write_series(w, idx, s, ct);
    }

    // Chart-type-level elements after series
    if matches!(
        ct,
        ChartType::Bar | ChartType::Column | ChartType::Bar3D | ChartType::Column3D
    ) {
        w.empty_tag("c:gapWidth", &[("val", "219")]);
        w.empty_tag("c:overlap", &[("val", "-27")]);
    }
    if matches!(ct, ChartType::Line | ChartType::Line3D) {
        w.empty_tag("c:marker", &[("val", "1")]);
        w.empty_tag("c:smooth", &[("val", "0")]);
    }

    // Drop lines and hi-low lines (valid for line and stock chart types)
    if matches!(ct, ChartType::Line | ChartType::Line3D | ChartType::Stock) {
        if drop_lines {
            w.empty_tag("c:dropLines", &[]);
        }
        if hi_low_lines {
            w.empty_tag("c:hiLowLines", &[]);
        }
    }

    if !matches!(ct, ChartType::Pie | ChartType::Doughnut | ChartType::Pie3D) {
        // Band formats for surface charts (color bands)
        if matches!(ct, ChartType::Surface | ChartType::WireframeSurface) {
            let accents = [
                "accent1", "accent2", "accent3", "accent4", "accent5", "accent6",
            ];
            w.start_tag("c:bandFmts", &[]);
            for (i, accent) in accents.iter().enumerate() {
                let idx = i.to_string();
                w.start_tag("c:bandFmt", &[]);
                w.empty_tag("c:idx", &[("val", &idx)]);
                w.start_tag("c:spPr", &[]);
                w.start_tag("a:solidFill", &[]);
                w.empty_tag("a:schemeClr", &[("val", accent)]);
                w.end_tag("a:solidFill");
                w.end_tag("c:spPr");
                w.end_tag("c:bandFmt");
            }
            w.end_tag("c:bandFmts");
        }
        w.empty_tag("c:axId", &[("val", cat_ax_id)]);
        w.empty_tag("c:axId", &[("val", val_ax_id)]);
        // Surface charts require a third axis (series axis)
        if matches!(ct, ChartType::Surface | ChartType::WireframeSurface) {
            w.empty_tag("c:axId", &[("val", "555555555")]);
        }
    }

    w.end_tag(tag);
}

fn write_series(
    w: &mut XmlWriter,
    idx: usize,
    s: &crate::features::chart::ChartSeries,
    ct: ChartType,
) {
    use crate::features::chart::MarkerType;
    let idx_s = idx.to_string();
    w.start_tag("c:ser", &[]);
    w.empty_tag("c:idx", &[("val", &idx_s)]);
    w.empty_tag("c:order", &[("val", &idx_s)]);
    if let Some(ref name) = s.name {
        // Surface charts don't use c:tx (series name) — it causes issues
        if !matches!(ct, ChartType::Surface | ChartType::WireframeSurface) {
            w.start_tag("c:tx", &[]);
            w.text_element("c:v", &[], name);
            w.end_tag("c:tx");
        }
    }

    // Series shape properties (fill, line width, dash style, gradient)
    let has_sppr = s.color.is_some()
        || s.line_width.is_some()
        || s.dash_style.is_some()
        || s.gradient.is_some();
    if has_sppr {
        w.start_tag("c:spPr", &[]);
        // Fill
        if let Some(ref stops) = s.gradient {
            write_gradient_fill(w, stops);
        } else if let Some(rgb) = s.color {
            w.start_tag("a:solidFill", &[]);
            let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
            w.empty_tag("a:srgbClr", &[("val", &hex)]);
            w.end_tag("a:solidFill");
        }
        // Line (width and dash style)
        if s.line_width.is_some() || s.dash_style.is_some() {
            let emu_s;
            let mut ln_attrs: Vec<(&str, &str)> = Vec::new();
            if let Some(width) = s.line_width {
                emu_s = ((width * 12700.0) as u64).to_string();
                ln_attrs.push(("w", &emu_s));
            }
            w.start_tag("a:ln", &ln_attrs);
            if let Some(dash) = s.dash_style {
                let val = dash_style_val(dash);
                w.empty_tag("a:prstDash", &[("val", val)]);
            }
            w.end_tag("a:ln");
        }
        w.end_tag("c:spPr");
    }

    // Bar/column: invertIfNegative
    if matches!(ct, ChartType::Bar | ChartType::Column) {
        w.empty_tag("c:invertIfNegative", &[("val", "0")]);
    }

    // Marker (line/scatter)
    if let Some(marker) = s.marker {
        w.start_tag("c:marker", &[]);
        let sym = match marker {
            MarkerType::None => "none",
            MarkerType::Circle => "circle",
            MarkerType::Diamond => "diamond",
            MarkerType::Square => "square",
            MarkerType::Triangle => "triangle",
            MarkerType::Star => "star",
            MarkerType::Plus => "plus",
            MarkerType::X => "x",
        };
        w.empty_tag("c:symbol", &[("val", sym)]);
        if let Some(sz) = s.marker_size {
            let sz_s = sz.to_string();
            w.empty_tag("c:size", &[("val", &sz_s)]);
        }
        w.end_tag("c:marker");
    } else if matches!(ct, ChartType::Line) {
        w.start_tag("c:marker", &[]);
        w.empty_tag("c:symbol", &[("val", "none")]);
        w.end_tag("c:marker");
    }

    // Point colors
    for &(pt_idx, rgb) in &s.point_colors {
        let pt_s = pt_idx.to_string();
        w.start_tag("c:dPt", &[]);
        w.empty_tag("c:idx", &[("val", &pt_s)]);
        w.start_tag("c:spPr", &[]);
        w.start_tag("a:solidFill", &[]);
        let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        w.empty_tag("a:srgbClr", &[("val", &hex)]);
        w.end_tag("a:solidFill");
        w.end_tag("c:spPr");
        w.end_tag("c:dPt");
    }

    // Data labels (per-series)
    if s.data_labels {
        w.start_tag("c:dLbls", &[]);
        w.empty_tag("c:showLegendKey", &[("val", "0")]);
        w.empty_tag("c:showVal", &[("val", "1")]);
        w.empty_tag("c:showCatName", &[("val", "0")]);
        w.empty_tag("c:showSerName", &[("val", "0")]);
        w.empty_tag("c:showPercent", &[("val", "0")]);
        w.empty_tag("c:showBubbleSize", &[("val", "0")]);
        w.end_tag("c:dLbls");
    }

    // Trendline
    if s.trendline.is_some() {
        write_trendline(w, s);
    }

    // Error bars
    if let Some(ref eb) = s.error_bars {
        write_error_bars(w, eb);
    }

    // Categories (not for surface charts — surface uses only val)
    if let Some(ref cats) = s.categories
        && !matches!(ct, ChartType::Surface | ChartType::WireframeSurface)
    {
        let cat_tag = if matches!(ct, ChartType::Scatter | ChartType::Bubble) {
            "c:xVal"
        } else {
            "c:cat"
        };
        w.start_tag(cat_tag, &[]);
        w.start_tag("c:strRef", &[]);
        w.text_element("c:f", &[], cats);
        w.end_tag("c:strRef");
        w.end_tag(cat_tag);
    }

    // Values
    let val_tag = if matches!(ct, ChartType::Scatter | ChartType::Bubble) {
        "c:yVal"
    } else {
        "c:val"
    };
    w.start_tag(val_tag, &[]);
    w.start_tag("c:numRef", &[]);
    w.text_element("c:f", &[], &s.values);
    // Surface charts require a numCache to avoid crashing Excel's Chart module
    if matches!(ct, ChartType::Surface | ChartType::WireframeSurface) {
        w.start_tag("c:numCache", &[]);
        w.text_element("c:formatCode", &[], "General");
        w.empty_tag("c:ptCount", &[("val", "0")]);
        w.end_tag("c:numCache");
    }
    w.end_tag("c:numRef");
    w.end_tag(val_tag);

    // Bubble sizes (bubble chart only)
    if matches!(ct, ChartType::Bubble)
        && let Some(ref sizes) = s.bubble_sizes
    {
        w.start_tag("c:bubbleSize", &[]);
        w.start_tag("c:numRef", &[]);
        w.text_element("c:f", &[], sizes);
        w.end_tag("c:numRef");
        w.end_tag("c:bubbleSize");
    }

    // Line: smooth
    if matches!(ct, ChartType::Line) {
        w.empty_tag("c:smooth", &[("val", "0")]);
    }

    w.end_tag("c:ser");
}

fn write_trendline(w: &mut XmlWriter, s: &crate::features::chart::ChartSeries) {
    use crate::features::chart::TrendlineType;
    let tl = s.trendline.as_ref().unwrap();
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
    if s.trendline_display_rsquared {
        w.empty_tag("c:dispRSqr", &[("val", "1")]);
    }
    if s.trendline_display_equation {
        w.empty_tag("c:dispEq", &[("val", "1")]);
    }
    w.end_tag("c:trendline");
}

fn write_axes(w: &mut XmlWriter, chart: &Chart, has_secondary: bool) {
    // Primary category axis
    w.start_tag("c:catAx", &[]);
    w.empty_tag("c:axId", &[("val", "111111111")]);
    w.start_tag("c:scaling", &[]);
    let x_orient = if chart.x_axis_reverse {
        "maxMin"
    } else {
        "minMax"
    };
    w.empty_tag("c:orientation", &[("val", x_orient)]);
    w.end_tag("c:scaling");
    w.empty_tag("c:axPos", &[("val", "b")]);
    // Axis formatting: major gridlines
    if let Some(ref fmt) = chart.x_axis_format {
        write_axis_gridlines(w, fmt);
    }
    if let Some(ref name) = chart.x_axis_name {
        write_axis_title(w, name);
    }
    // Axis formatting: numFmt
    if let Some(ref fmt) = chart.x_axis_format
        && let Some(ref nf) = fmt.num_format
    {
        w.empty_tag("c:numFmt", &[("formatCode", nf), ("sourceLinked", "0")]);
    }
    // Axis formatting: tick marks
    if let Some(ref fmt) = chart.x_axis_format {
        write_axis_tick_marks(w, fmt);
    }
    w.empty_tag("c:tickLblPos", &[("val", "nextTo")]);
    w.empty_tag("c:crossAx", &[("val", "222222222")]);
    w.empty_tag("c:crosses", &[("val", "autoZero")]);
    w.empty_tag("c:auto", &[("val", "1")]);
    w.empty_tag("c:lblAlgn", &[("val", "ctr")]);
    w.empty_tag("c:lblOffset", &[("val", "100")]);
    // Axis formatting: txPr (font)
    if let Some(ref fmt) = chart.x_axis_format {
        write_axis_txpr(w, fmt);
    }
    w.end_tag("c:catAx");

    // Primary value axis
    w.start_tag("c:valAx", &[]);
    w.empty_tag("c:axId", &[("val", "222222222")]);
    w.start_tag("c:scaling", &[]);
    let y_orient = if chart.y_axis_reverse {
        "maxMin"
    } else {
        "minMax"
    };
    w.empty_tag("c:orientation", &[("val", y_orient)]);
    if let Some(max) = chart.y_axis_max {
        let s = format!("{max}");
        w.empty_tag("c:max", &[("val", &s)]);
    }
    if let Some(min) = chart.y_axis_min {
        let s = format!("{min}");
        w.empty_tag("c:min", &[("val", &s)]);
    }
    if let Some(base) = chart.y_axis_log_base {
        let s = format!("{base}");
        w.empty_tag("c:logBase", &[("val", &s)]);
    }
    w.end_tag("c:scaling");
    w.empty_tag("c:axPos", &[("val", "l")]);
    // Axis formatting: major/minor gridlines
    if let Some(ref fmt) = chart.y_axis_format {
        write_axis_gridlines(w, fmt);
    } else {
        // Default: major gridlines on value axis
        w.empty_tag("c:majorGridlines", &[]);
    }
    if let Some(ref name) = chart.y_axis_name {
        write_axis_title(w, name);
    }
    // Axis formatting: numFmt
    if let Some(ref fmt) = chart.y_axis_format {
        if let Some(ref nf) = fmt.num_format {
            w.empty_tag("c:numFmt", &[("formatCode", nf), ("sourceLinked", "0")]);
        } else {
            w.empty_tag(
                "c:numFmt",
                &[("formatCode", "General"), ("sourceLinked", "1")],
            );
        }
    } else {
        w.empty_tag(
            "c:numFmt",
            &[("formatCode", "General"), ("sourceLinked", "1")],
        );
    }
    // Axis formatting: tick marks
    if let Some(ref fmt) = chart.y_axis_format {
        write_axis_tick_marks(w, fmt);
    }
    w.empty_tag("c:tickLblPos", &[("val", "nextTo")]);
    w.empty_tag("c:crossAx", &[("val", "111111111")]);
    w.empty_tag("c:crosses", &[("val", "autoZero")]);
    w.empty_tag("c:crossBetween", &[("val", "between")]);
    // Axis formatting: txPr (font)
    if let Some(ref fmt) = chart.y_axis_format {
        write_axis_txpr(w, fmt);
    }
    w.end_tag("c:valAx");

    if has_secondary {
        // Secondary category axis — hidden
        w.start_tag("c:catAx", &[]);
        w.empty_tag("c:axId", &[("val", "333333333")]);
        w.start_tag("c:scaling", &[]);
        w.empty_tag("c:orientation", &[("val", "minMax")]);
        w.end_tag("c:scaling");
        w.empty_tag("c:delete", &[("val", "1")]);
        w.empty_tag("c:axPos", &[("val", "b")]);
        w.empty_tag("c:tickLblPos", &[("val", "nextTo")]);
        w.empty_tag("c:crossAx", &[("val", "444444444")]);
        w.empty_tag("c:crosses", &[("val", "autoZero")]);
        w.end_tag("c:catAx");

        // Secondary value axis — right side
        w.start_tag("c:valAx", &[]);
        w.empty_tag("c:axId", &[("val", "444444444")]);
        w.start_tag("c:scaling", &[]);
        w.empty_tag("c:orientation", &[("val", "minMax")]);
        w.end_tag("c:scaling");
        w.empty_tag("c:axPos", &[("val", "r")]);
        if let Some(ref name) = chart.y2_axis_name {
            write_axis_title(w, name);
        }
        w.empty_tag(
            "c:numFmt",
            &[("formatCode", "General"), ("sourceLinked", "1")],
        );
        w.empty_tag("c:tickLblPos", &[("val", "nextTo")]);
        w.empty_tag("c:crossAx", &[("val", "333333333")]);
        w.empty_tag("c:crosses", &[("val", "max")]);
        w.empty_tag("c:crossBetween", &[("val", "between")]);
        w.end_tag("c:valAx");
    }

    // Series axis for surface charts
    if matches!(
        chart.chart_type,
        ChartType::Surface | ChartType::WireframeSurface
    ) {
        w.start_tag("c:serAx", &[]);
        w.empty_tag("c:axId", &[("val", "555555555")]);
        w.start_tag("c:scaling", &[]);
        w.empty_tag("c:orientation", &[("val", "minMax")]);
        w.end_tag("c:scaling");
        w.empty_tag("c:delete", &[("val", "0")]);
        w.empty_tag("c:axPos", &[("val", "b")]);
        w.empty_tag("c:tickLblPos", &[("val", "nextTo")]);
        w.empty_tag("c:crossAx", &[("val", "222222222")]);
        w.empty_tag("c:crosses", &[("val", "autoZero")]);
        w.end_tag("c:serAx");
    }
}

fn chart_type_tag(ct: ChartType) -> &'static str {
    match ct {
        ChartType::Bar => "c:barChart",
        ChartType::Column => "c:barChart",
        ChartType::Line => "c:lineChart",
        ChartType::Pie => "c:pieChart",
        ChartType::Scatter => "c:scatterChart",
        ChartType::Area => "c:areaChart",
        ChartType::Doughnut => "c:doughnutChart",
        ChartType::Radar => "c:radarChart",
        ChartType::Stock => "c:stockChart",
        ChartType::Bubble => "c:bubbleChart",
        ChartType::Column3D => "c:bar3DChart",
        ChartType::Bar3D => "c:bar3DChart",
        ChartType::Line3D => "c:line3DChart",
        ChartType::Pie3D => "c:pie3DChart",
        ChartType::Area3D => "c:area3DChart",
        ChartType::Surface => "c:surface3DChart",
        ChartType::WireframeSurface => "c:surface3DChart",
        ChartType::Map => "c:barChart", // Map charts use a special namespace; fallback to bar
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

/// Office theme accent colors used for pivot chart series formatting.
const ACCENT_COLORS: &[&str] = &[
    "accent1", "accent2", "accent3", "accent4", "accent5", "accent6",
];

/// Write a pivot chart series with inline cached values (no cell references needed).
fn write_pivot_series(
    w: &mut XmlWriter,
    idx: usize,
    ps: &crate::features::chart::PivotChartSeriesData,
) {
    let idx_s = idx.to_string();
    let accent = ACCENT_COLORS[idx % ACCENT_COLORS.len()];
    w.start_tag("c:ser", &[]);
    w.empty_tag("c:idx", &[("val", &idx_s)]);
    w.empty_tag("c:order", &[("val", &idx_s)]);
    // Series name
    w.start_tag("c:tx", &[]);
    w.text_element("c:v", &[], &ps.name);
    w.end_tag("c:tx");
    // Series fill color
    w.start_tag("c:spPr", &[]);
    w.start_tag("a:solidFill", &[]);
    w.empty_tag("a:schemeClr", &[("val", accent)]);
    w.end_tag("a:solidFill");
    w.start_tag("a:ln", &[]);
    w.empty_tag("a:noFill", &[]);
    w.end_tag("a:ln");
    w.empty_tag("a:effectLst", &[]);
    w.end_tag("c:spPr");
    w.empty_tag("c:invertIfNegative", &[("val", "0")]);
    // Categories — inline string cache
    let pt_count = ps.categories.len().to_string();
    w.start_tag("c:cat", &[]);
    w.start_tag("c:strLit", &[]);
    w.empty_tag("c:ptCount", &[("val", &pt_count)]);
    for (ci, cat) in ps.categories.iter().enumerate() {
        let ci_s = ci.to_string();
        w.start_tag("c:pt", &[("idx", &ci_s)]);
        w.text_element("c:v", &[], cat);
        w.end_tag("c:pt");
    }
    w.end_tag("c:strLit");
    w.end_tag("c:cat");
    // Values — inline number cache
    w.start_tag("c:val", &[]);
    w.start_tag("c:numLit", &[]);
    w.empty_tag("c:formatCode", &[]);
    w.empty_tag("c:ptCount", &[("val", &pt_count)]);
    for (vi, val) in ps.values.iter().enumerate() {
        let vi_s = vi.to_string();
        let val_s = format!("{val}");
        w.start_tag("c:pt", &[("idx", &vi_s)]);
        w.text_element("c:v", &[], &val_s);
        w.end_tag("c:pt");
    }
    w.end_tag("c:numLit");
    w.end_tag("c:val");
    w.end_tag("c:ser");
}

/// Write <c:pivotFmts> — one format entry per series with proper accent colors,
/// marker suppression, and data label configuration.
fn write_pivot_fmts(w: &mut XmlWriter, count: usize) {
    w.start_tag("c:pivotFmts", &[]);
    for i in 0..count {
        let idx = i.to_string();
        let accent = ACCENT_COLORS[i % ACCENT_COLORS.len()];
        w.start_tag("c:pivotFmt", &[]);
        w.empty_tag("c:idx", &[("val", &idx)]);
        // Series fill — theme accent color
        w.start_tag("c:spPr", &[]);
        w.start_tag("a:solidFill", &[]);
        w.empty_tag("a:schemeClr", &[("val", accent)]);
        w.end_tag("a:solidFill");
        w.start_tag("a:ln", &[]);
        w.empty_tag("a:noFill", &[]);
        w.end_tag("a:ln");
        w.empty_tag("a:effectLst", &[]);
        w.end_tag("c:spPr");
        // Marker — suppress for bar/column charts
        w.start_tag("c:marker", &[]);
        w.empty_tag("c:symbol", &[("val", "none")]);
        w.end_tag("c:marker");
        // Data label defaults — all hidden
        w.start_tag("c:dLbl", &[]);
        w.empty_tag("c:idx", &[("val", "0")]);
        w.empty_tag("c:showLegendKey", &[("val", "0")]);
        w.empty_tag("c:showVal", &[("val", "0")]);
        w.empty_tag("c:showCatName", &[("val", "0")]);
        w.empty_tag("c:showSerName", &[("val", "0")]);
        w.empty_tag("c:showPercent", &[("val", "0")]);
        w.empty_tag("c:showBubbleSize", &[("val", "0")]);
        w.end_tag("c:dLbl");
        w.end_tag("c:pivotFmt");
    }
    w.end_tag("c:pivotFmts");
}

/// Write pivot chart extensions — drop zone controls and expand/collapse buttons.
fn write_pivot_extensions(w: &mut XmlWriter, ps: &crate::features::chart::PivotChartSource) {
    w.start_tag("c:extLst", &[]);
    // c14:pivotOptions — drop zone visibility
    w.start_tag(
        "c:ext",
        &[
            (
                "xmlns:c14",
                "http://schemas.microsoft.com/office/drawing/2007/8/2/chart",
            ),
            ("uri", "{781A3756-C4B2-4CAC-9D66-4F8BD8637D16}"),
        ],
    );
    w.start_tag("c14:pivotOptions", &[]);
    let b = |v: bool| if v { "1" } else { "0" };
    w.empty_tag(
        "c14:dropZoneFilter",
        &[("val", b(ps.show_drop_zone_filter))],
    );
    w.empty_tag(
        "c14:dropZoneCategories",
        &[("val", b(ps.show_drop_zone_categories))],
    );
    w.empty_tag("c14:dropZoneData", &[("val", b(ps.show_drop_zone_data))]);
    w.empty_tag(
        "c14:dropZoneSeries",
        &[("val", b(ps.show_drop_zone_series))],
    );
    w.empty_tag("c14:dropZonesVisible", &[("val", "1")]);
    w.end_tag("c14:pivotOptions");
    w.end_tag("c:ext");
    // c16:pivotOptions16 — expand/collapse field buttons
    w.start_tag(
        "c:ext",
        &[
            (
                "xmlns:c16",
                "http://schemas.microsoft.com/office/drawing/2014/chart",
            ),
            ("uri", "{E28EC0CA-F0BB-4C9C-879D-F8772B89E7AC}"),
        ],
    );
    w.start_tag("c16:pivotOptions16", &[]);
    w.empty_tag(
        "c16:showExpandCollapseFieldButtons",
        &[("val", b(ps.show_expand_collapse))],
    );
    w.end_tag("c16:pivotOptions16");
    w.end_tag("c:ext");
    w.end_tag("c:extLst");
}

// ── Axis formatting helpers (Task 36) ───────────────────────────────────────

/// Write major/minor gridlines for an axis based on AxisFormat settings.
fn write_axis_gridlines(w: &mut XmlWriter, fmt: &crate::features::chart::AxisFormat) {
    if fmt.major_gridlines {
        if fmt.gridline_color.is_some() || fmt.gridline_width.is_some() {
            w.start_tag("c:majorGridlines", &[]);
            w.start_tag("c:spPr", &[]);
            if let Some(width) = fmt.gridline_width {
                // EMU: 1pt = 12700 EMU
                let emu = (width * 12700.0) as u64;
                let emu_s = emu.to_string();
                w.start_tag("a:ln", &[("w", &emu_s)]);
            } else {
                w.start_tag("a:ln", &[]);
            }
            if let Some(rgb) = fmt.gridline_color {
                w.start_tag("a:solidFill", &[]);
                let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
                w.empty_tag("a:srgbClr", &[("val", &hex)]);
                w.end_tag("a:solidFill");
            }
            w.end_tag("a:ln");
            w.end_tag("c:spPr");
            w.end_tag("c:majorGridlines");
        } else {
            w.empty_tag("c:majorGridlines", &[]);
        }
    }
    if fmt.minor_gridlines {
        w.empty_tag("c:minorGridlines", &[]);
    }
}

/// Write tick mark elements for an axis.
fn write_axis_tick_marks(w: &mut XmlWriter, fmt: &crate::features::chart::AxisFormat) {
    if let Some(ref tm) = fmt.major_tick_mark {
        let val = tick_mark_val(tm);
        w.empty_tag("c:majorTickMark", &[("val", val)]);
    }
    if let Some(ref tm) = fmt.minor_tick_mark {
        let val = tick_mark_val(tm);
        w.empty_tag("c:minorTickMark", &[("val", val)]);
    }
}

/// Write txPr (text properties) element for axis font formatting.
fn write_axis_txpr(w: &mut XmlWriter, fmt: &crate::features::chart::AxisFormat) {
    let has_font = fmt.font_name.is_some()
        || fmt.font_size.is_some()
        || fmt.font_color.is_some()
        || fmt.font_bold;
    if !has_font {
        return;
    }
    w.start_tag("c:txPr", &[]);
    w.empty_tag("a:bodyPr", &[]);
    w.empty_tag("a:lstStyle", &[]);
    w.start_tag("a:p", &[]);
    w.start_tag("a:pPr", &[]);
    // Build defRPr attributes
    let sz_s;
    let mut attrs: Vec<(&str, &str)> = Vec::new();
    if let Some(size) = fmt.font_size {
        sz_s = ((size * 100.0) as u32).to_string();
        attrs.push(("sz", &sz_s));
    }
    if fmt.font_bold {
        attrs.push(("b", "1"));
    }
    w.start_tag("a:defRPr", &attrs);
    if let Some(rgb) = fmt.font_color {
        w.start_tag("a:solidFill", &[]);
        let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        w.empty_tag("a:srgbClr", &[("val", &hex)]);
        w.end_tag("a:solidFill");
    }
    if let Some(ref font) = fmt.font_name {
        w.empty_tag("a:latin", &[("typeface", font)]);
        w.empty_tag("a:cs", &[("typeface", font)]);
    }
    w.end_tag("a:defRPr");
    w.end_tag("a:pPr");
    w.end_tag("a:p");
    w.end_tag("c:txPr");
}

/// Convert TickMark enum to XML attribute value.
fn tick_mark_val(tm: &TickMark) -> &'static str {
    match tm {
        TickMark::None => "none",
        TickMark::Inside => "in",
        TickMark::Outside => "out",
        TickMark::Cross => "cross",
    }
}

// ── Plot area and series formatting helpers (Task 37) ────────────────────────

/// Write <c:spPr> for the plot area with fill, border, and gradient.
fn write_plot_area_sppr(w: &mut XmlWriter, pf: &crate::features::chart::PlotAreaFormat) {
    w.start_tag("c:spPr", &[]);
    // Fill
    if let Some(ref stops) = pf.gradient {
        write_gradient_fill(w, stops);
    } else if let Some(rgb) = pf.fill {
        w.start_tag("a:solidFill", &[]);
        let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        w.empty_tag("a:srgbClr", &[("val", &hex)]);
        w.end_tag("a:solidFill");
    }
    // Border
    if let Some(rgb) = pf.border {
        w.start_tag("a:ln", &[]);
        w.start_tag("a:solidFill", &[]);
        let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        w.empty_tag("a:srgbClr", &[("val", &hex)]);
        w.end_tag("a:solidFill");
        w.end_tag("a:ln");
    }
    w.end_tag("c:spPr");
}

/// Write a gradient fill element.
fn write_gradient_fill(w: &mut XmlWriter, stops: &[([u8; 3], f64)]) {
    w.start_tag("a:gradFill", &[]);
    w.start_tag("a:gsLst", &[]);
    for &(rgb, pos) in stops {
        let pos_val = ((pos * 100000.0) as u64).to_string();
        w.start_tag("a:gs", &[("pos", &pos_val)]);
        let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        w.empty_tag("a:srgbClr", &[("val", &hex)]);
        w.end_tag("a:gs");
    }
    w.end_tag("a:gsLst");
    w.end_tag("a:gradFill");
}

/// Convert DashStyle enum to XML preset dash value.
fn dash_style_val(ds: crate::features::chart::DashStyle) -> &'static str {
    use crate::features::chart::DashStyle;
    match ds {
        DashStyle::Solid => "solid",
        DashStyle::Dash => "dash",
        DashStyle::Dot => "dot",
        DashStyle::DashDot => "dashDot",
        DashStyle::LongDash => "lgDash",
        DashStyle::LongDashDot => "lgDashDot",
    }
}

/// Write <c:errBars> element for error bars on a series.
fn write_error_bars(w: &mut XmlWriter, eb: &crate::features::chart::ErrorBar) {
    use crate::features::chart::{ErrorBarType, ErrorBarValueType};
    w.start_tag("c:errBars", &[]);
    // Direction
    let dir = match eb.bar_type {
        ErrorBarType::Both => "both",
        ErrorBarType::Plus => "plus",
        ErrorBarType::Minus => "minus",
    };
    w.empty_tag("c:errBarType", &[("val", dir)]);
    // Value type
    let (val_type, needs_val) = match eb.value_type {
        ErrorBarValueType::FixedValue => ("fixedVal", true),
        ErrorBarValueType::Percentage => ("percentage", true),
        ErrorBarValueType::StandardDeviation => ("stdDev", true),
        ErrorBarValueType::StandardError => ("stdErr", false),
    };
    w.empty_tag("c:errValType", &[("val", val_type)]);
    w.empty_tag("c:noEndCap", &[("val", "0")]);
    if needs_val {
        let val_s = eb.value.to_string();
        w.empty_tag("c:val", &[("val", &val_s)]);
    }
    w.end_tag("c:errBars");
}
