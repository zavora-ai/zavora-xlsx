//! Parser for chart XML parts (`xl/charts/chart{N}.xml`) and ChartEx parts
//! (`xl/charts/chartEx{N}.xml`).
//!
//! Converts raw chart XML bytes into [`Chart`] and [`TreemapChart`] structs by
//! parsing the `c:chartSpace` / `cx:chartSpace` root elements.

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::features::chart::{Chart, ChartSeries, ChartType, LegendPosition};
use crate::features::treemap::TreemapChart;
use crate::xml::xml_reader::get_attr_str;

// ── Drawing relationship parsing (Task 4.2) ────────────────────────────────

/// Discovered chart reference from a drawing XML part.
#[derive(Debug)]
pub struct DrawingChartRef {
    /// The cell the chart's top-left corner sits over, from the drawing's `<xdr:from>`.
    ///
    /// Absent when the drawing does not say. This used to be dropped entirely, so every chart
    /// read back as anchored at A1 and an application could not put one where the file puts it.
    pub from_row: Option<u32>,
    pub from_col: Option<u16>,
    /// Relationship ID (e.g. "rId1") pointing to the chart part.
    pub r_id: String,
    /// Whether this is a ChartEx reference (`cx:chart`) vs standard (`c:chart`).
    pub is_chartex: bool,
}

/// Parse a drawing XML (`xl/drawings/drawing{N}.xml`) to discover chart
/// relationship IDs embedded in `<xdr:graphicFrame>` anchors.
/// A chart part, and where on the sheet it sits.
#[derive(Debug, Clone)]
pub struct ResolvedChart {
    pub path: String,
    pub is_chartex: bool,
    pub from_row: Option<u32>,
    pub from_col: Option<u16>,
}

pub fn parse_drawing_chart_refs(data: &[u8]) -> Vec<DrawingChartRef> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(1024);
    let mut refs = Vec::new();
    let mut in_from = false;
    let mut reading: Option<Vec<u8>> = None;
    let mut from_row: Option<u32> = None;
    let mut from_col: Option<u16> = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                let name = local.as_ref();
                // The anchor comes before the chart it belongs to, so the position seen most
                // recently is this chart's.
                match name {
                    b"from" => in_from = true,
                    b"to" => in_from = false,
                    b"col" | b"row" if in_from => reading = Some(name.to_vec()),
                    _ => {}
                }
                // Standard chart: <c:chart r:id="rIdN" .../>
                if name == b"chart" {
                    // Distinguish c:chart vs cx:chart by checking namespace prefix
                    let full_name = e.name();
                    let full = full_name.as_ref();
                    let is_cx = full.starts_with(b"cx:");
                    if let Some(rid) = get_attr_str(e.attributes(), b"r:id") {
                        refs.push(DrawingChartRef {
                            r_id: rid.to_string(),
                            is_chartex: is_cx,
                            from_row,
                            from_col,
                        });
                        // Cleared so a second chart in the same drawing cannot inherit the
                        // first one's position when its own anchor is missing.
                        from_row = None;
                        from_col = None;
                    }
                }
            }
            Ok(Event::Text(ref text)) => {
                if let Some(which) = reading.take()
                    && let Ok(read) = text.unescape()
                {
                    let number = read.trim().parse::<u32>().ok();
                    if which == b"row" {
                        from_row = number;
                    } else {
                        from_col = number.and_then(|value| u16::try_from(value).ok());
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.local_name().as_ref() == b"from" {
                    in_from = false;
                }
                reading = None;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    refs
}

/// Resolve drawing relationship IDs to chart part paths using the drawing
/// rels file (`xl/drawings/_rels/drawing{N}.xml.rels`).
///
/// Returns each chart's path, whether it is a chartex, and where it is anchored.
pub fn resolve_chart_paths(
    drawing_refs: &[DrawingChartRef],
    rels_data: &[u8],
) -> Vec<ResolvedChart> {
    let rels = match crate::reader::rel_parser::parse_rels(rels_data) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut paths = Vec::new();
    for dref in drawing_refs {
        if let Some(rel) = rels.iter().find(|r| r.id == dref.r_id) {
            // Target is relative like "../charts/chart1.xml"
            let target = &rel.target;
            let full_path = if let Some(stripped) = target.strip_prefix("../") {
                format!("xl/{}", stripped)
            } else if let Some(stripped) = target.strip_prefix("/xl/") {
                stripped.to_string()
            } else if target.starts_with("xl/") {
                target.to_string()
            } else {
                format!("xl/charts/{target}")
            };
            paths.push(ResolvedChart {
                path: full_path,
                is_chartex: dref.is_chartex,
                from_row: dref.from_row,
                from_col: dref.from_col,
            });
        }
    }
    paths
}

// ── Standard chart parsing (c:chart namespace) (Tasks 4.3–4.5) ─────────────

/// Parse a standard chart XML part (`c:chartSpace`) into a [`Chart`] struct.
pub fn read_chart(data: &[u8]) -> crate::Result<Chart> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(2048);

    let mut chart_type = ChartType::Bar;
    let mut series: Vec<ChartSeries> = Vec::new();
    let mut title: Option<String> = None;
    let mut x_axis_name: Option<String> = None;
    let mut y_axis_name: Option<String> = None;
    let mut legend_pos = LegendPosition::None;
    let mut bar_dir: Option<String> = None;
    let mut style: Option<u8> = None;

    // Track parsing context
    let mut in_chart = false;
    let mut in_plot_area = false;
    let mut in_chart_type_block = false;
    let mut current_chart_tag: Option<String> = None;
    let mut in_ser = false;
    let mut in_title = false;
    let mut in_legend = false;
    let mut in_cat_ax = false;
    let mut in_val_ax = false;
    let mut in_ax_title = false;
    let mut in_tx = false;
    let mut in_cat = false;
    let mut in_val = false;
    let mut in_str_ref = false;
    let mut in_num_ref = false;
    let mut in_f = false;
    let mut f_text = String::new();
    let mut in_a_t = false;
    let mut a_t_text = String::new();
    let mut in_c_v = false;
    let mut c_v_text = String::new();

    // Current series being built
    let mut cur_series_name: Option<String> = None;
    let mut cur_series_cats: Option<String> = None;
    let mut cur_series_vals = String::new();

    // For collecting title text
    let mut title_text = String::new();
    let mut ax_title_text = String::new();

    // Depth tracking for nested title elements
    let mut title_depth = 0u32;
    let mut ax_title_depth = 0u32;

    // Track which axis we found the title in
    let mut found_cat_ax_title = false;
    let mut found_val_ax_title = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                let name = local.as_ref();
                match name {
                    b"chart" => {
                        in_chart = true;
                    }
                    b"plotArea" if in_chart => {
                        in_plot_area = true;
                    }
                    b"barChart" | b"lineChart" | b"pieChart" | b"scatterChart" | b"areaChart"
                    | b"doughnutChart" | b"radarChart" | b"stockChart" | b"bar3DChart"
                    | b"line3DChart" | b"pie3DChart" | b"area3DChart" | b"bubbleChart"
                    | b"surface3DChart" | b"surfaceChart"
                        if in_plot_area =>
                    {
                        in_chart_type_block = true;
                        current_chart_tag =
                            Some(std::str::from_utf8(name).unwrap_or("").to_string());
                        bar_dir = None;
                    }
                    b"barDir" if in_chart_type_block => {
                        bar_dir = get_attr_str(e.attributes(), b"val").map(|s| s.to_string());
                    }
                    b"ser" if in_chart_type_block => {
                        in_ser = true;
                        cur_series_name = None;
                        cur_series_cats = None;
                        cur_series_vals = String::new();
                    }
                    b"tx" if in_ser => {
                        in_tx = true;
                    }
                    b"cat" | b"xVal" if in_ser => {
                        in_cat = true;
                    }
                    b"val" | b"yVal" if in_ser => {
                        in_val = true;
                    }
                    b"strRef" if in_tx || in_cat => {
                        in_str_ref = true;
                    }
                    b"numRef" if in_val => {
                        in_num_ref = true;
                    }
                    b"f" if in_str_ref || in_num_ref => {
                        in_f = true;
                        f_text.clear();
                    }
                    b"v" if in_tx => {
                        in_c_v = true;
                        c_v_text.clear();
                    }
                    b"title"
                        if in_chart && !in_plot_area && !in_ser && !in_cat_ax && !in_val_ax =>
                    {
                        in_title = true;
                        title_depth = 1;
                        title_text.clear();
                    }
                    b"title" if in_title => {
                        title_depth += 1;
                    }
                    b"title" if (in_cat_ax || in_val_ax) && !in_ax_title => {
                        in_ax_title = true;
                        ax_title_depth = 1;
                        ax_title_text.clear();
                        if in_cat_ax {
                            found_cat_ax_title = true;
                        }
                        if in_val_ax {
                            found_val_ax_title = true;
                        }
                    }
                    b"title" if in_ax_title => {
                        ax_title_depth += 1;
                    }
                    b"legend" if in_chart => {
                        in_legend = true;
                    }
                    b"legendPos" if in_legend => {
                        legend_pos = match get_attr_str(e.attributes(), b"val") {
                            Some("t") => LegendPosition::Top,
                            Some("b") => LegendPosition::Bottom,
                            Some("l") => LegendPosition::Left,
                            Some("r") => LegendPosition::Right,
                            _ => LegendPosition::Bottom,
                        };
                    }
                    b"catAx" if in_plot_area => {
                        in_cat_ax = true;
                    }
                    b"valAx" if in_plot_area => {
                        in_val_ax = true;
                    }
                    b"style" => {
                        if let Some(val) = get_attr_str(e.attributes(), b"val")
                            && let Ok(n) = val.parse::<u8>()
                        {
                            style = Some(n);
                        }
                    }
                    b"t" => {
                        in_a_t = true;
                        a_t_text.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                let name = local.as_ref();
                match name {
                    b"barDir" if in_chart_type_block => {
                        bar_dir = get_attr_str(e.attributes(), b"val").map(|s| s.to_string());
                    }
                    b"legendPos" if in_legend => {
                        legend_pos = match get_attr_str(e.attributes(), b"val") {
                            Some("t") => LegendPosition::Top,
                            Some("b") => LegendPosition::Bottom,
                            Some("l") => LegendPosition::Left,
                            Some("r") => LegendPosition::Right,
                            _ => LegendPosition::Bottom,
                        };
                    }
                    b"style" => {
                        if let Some(val) = get_attr_str(e.attributes(), b"val")
                            && let Ok(n) = val.parse::<u8>()
                        {
                            style = Some(n);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref t)) => {
                if in_f && let Ok(s) = t.unescape() {
                    f_text.push_str(&s);
                }
                if in_a_t && let Ok(s) = t.unescape() {
                    a_t_text.push_str(&s);
                }
                if in_c_v && let Ok(s) = t.unescape() {
                    c_v_text.push_str(&s);
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                let name = local.as_ref();
                match name {
                    b"chart" => {
                        in_chart = false;
                    }
                    b"plotArea" => {
                        in_plot_area = false;
                    }
                    b"barChart" | b"lineChart" | b"pieChart" | b"scatterChart" | b"areaChart"
                    | b"doughnutChart" | b"radarChart" | b"stockChart" | b"bar3DChart"
                    | b"line3DChart" | b"pie3DChart" | b"area3DChart" | b"bubbleChart"
                    | b"surface3DChart" | b"surfaceChart"
                        if in_chart_type_block =>
                    {
                        // Determine chart type from the tag name
                        if let Some(ref tag) = current_chart_tag {
                            chart_type = resolve_chart_type(tag, bar_dir.as_deref());
                        }
                        in_chart_type_block = false;
                        current_chart_tag = None;
                    }
                    b"ser" if in_ser => {
                        let mut s = ChartSeries::new();
                        s.values = cur_series_vals.clone();
                        s.categories = cur_series_cats.clone();
                        s.name = cur_series_name.clone();
                        series.push(s);
                        in_ser = false;
                    }
                    b"tx" if in_ser => {
                        in_tx = false;
                    }
                    b"cat" | b"xVal" if in_ser => {
                        in_cat = false;
                    }
                    b"val" | b"yVal" if in_ser => {
                        in_val = false;
                    }
                    b"strRef" => {
                        in_str_ref = false;
                    }
                    b"numRef" => {
                        in_num_ref = false;
                    }
                    b"f" => {
                        if in_f {
                            if in_str_ref && in_tx && in_ser {
                                cur_series_name = Some(f_text.clone());
                            } else if in_str_ref && in_cat && in_ser {
                                cur_series_cats = Some(f_text.clone());
                            } else if in_num_ref && in_val && in_ser {
                                cur_series_vals = f_text.clone();
                            }
                            in_f = false;
                        }
                    }
                    b"v" if in_c_v => {
                        if in_tx && in_ser && cur_series_name.is_none() {
                            cur_series_name = Some(c_v_text.clone());
                        }
                        in_c_v = false;
                    }
                    b"t" => {
                        if in_a_t {
                            if in_title && !in_ax_title && !a_t_text.is_empty() {
                                title_text.push_str(&a_t_text);
                            }
                            if in_ax_title && !a_t_text.is_empty() {
                                ax_title_text.push_str(&a_t_text);
                            }
                            in_a_t = false;
                        }
                    }
                    b"title" if in_ax_title => {
                        ax_title_depth -= 1;
                        if ax_title_depth == 0 {
                            if found_cat_ax_title
                                && x_axis_name.is_none()
                                && !ax_title_text.is_empty()
                            {
                                x_axis_name = Some(ax_title_text.clone());
                            }
                            if found_val_ax_title
                                && y_axis_name.is_none()
                                && !ax_title_text.is_empty()
                            {
                                y_axis_name = Some(ax_title_text.clone());
                            }
                            in_ax_title = false;
                            found_cat_ax_title = false;
                            found_val_ax_title = false;
                        }
                    }
                    b"title" if in_title => {
                        title_depth -= 1;
                        if title_depth == 0 {
                            if !title_text.is_empty() {
                                title = Some(title_text.clone());
                            }
                            in_title = false;
                        }
                    }
                    b"legend" => {
                        in_legend = false;
                    }
                    b"catAx" => {
                        in_cat_ax = false;
                    }
                    b"valAx" => {
                        in_val_ax = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    let mut chart = Chart::new(chart_type);
    chart.title = title;
    chart.x_axis_name = x_axis_name;
    chart.y_axis_name = y_axis_name;
    chart.legend_pos = legend_pos;
    chart.series = series;
    chart.style = style;

    Ok(chart)
}

/// Map chart type XML element name + barDir to ChartType.
fn resolve_chart_type(tag: &str, bar_dir: Option<&str>) -> ChartType {
    match tag {
        "barChart" => match bar_dir {
            Some("bar") => ChartType::Bar,
            Some("col") => ChartType::Column,
            _ => ChartType::Column,
        },
        "bar3DChart" => match bar_dir {
            Some("bar") => ChartType::Bar3D,
            Some("col") => ChartType::Column3D,
            _ => ChartType::Column3D,
        },
        "lineChart" => ChartType::Line,
        "line3DChart" => ChartType::Line3D,
        "pieChart" => ChartType::Pie,
        "pie3DChart" => ChartType::Pie3D,
        "scatterChart" => ChartType::Scatter,
        "areaChart" => ChartType::Area,
        "area3DChart" => ChartType::Area3D,
        "doughnutChart" => ChartType::Doughnut,
        "radarChart" => ChartType::Radar,
        "stockChart" => ChartType::Stock,
        "bubbleChart" => ChartType::Bubble,
        "surface3DChart" => ChartType::Surface,
        "surfaceChart" => ChartType::Surface,
        _ => ChartType::Bar,
    }
}

// ── ChartEx parsing (cx:chart namespace) (Task 4.6) ─────────────────────────

/// Parse a ChartEx XML part (`cx:chartSpace`) into a [`TreemapChart`] struct.
pub fn read_chartex(data: &[u8]) -> crate::Result<TreemapChart> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(2048);

    let mut chart = TreemapChart::new();
    let mut in_chart_data = false;
    let mut in_str_dim = false;
    let mut in_num_dim = false;
    let mut in_lvl = false;
    let mut in_pt = false;
    let mut in_title = false;
    let mut in_tx = false;
    let mut in_series = false;
    let mut in_f = false;
    let mut in_a_t = false;
    let mut in_v = false;
    let mut pt_text = String::new();
    let mut f_text = String::new();
    let mut a_t_text = String::new();
    let mut v_text = String::new();
    let mut title_text = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                let name = local.as_ref();
                match name {
                    b"chartData" => {
                        in_chart_data = true;
                    }
                    b"strDim" if in_chart_data => {
                        in_str_dim = true;
                    }
                    b"numDim" if in_chart_data => {
                        in_num_dim = true;
                    }
                    b"lvl" if in_str_dim || in_num_dim => {
                        in_lvl = true;
                    }
                    b"pt" if in_lvl => {
                        in_pt = true;
                        pt_text.clear();
                    }
                    b"f" if in_str_dim || in_num_dim => {
                        in_f = true;
                        f_text.clear();
                    }
                    b"series" => {
                        in_series = true;
                    }
                    b"title" => {
                        in_title = true;
                        title_text.clear();
                    }
                    b"tx" if in_series => {
                        in_tx = true;
                    }
                    b"v" if in_tx => {
                        in_v = true;
                        v_text.clear();
                    }
                    b"t" => {
                        in_a_t = true;
                        a_t_text.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref t)) => {
                if in_pt && let Ok(s) = t.unescape() {
                    pt_text.push_str(&s);
                }
                if in_f && let Ok(s) = t.unescape() {
                    f_text.push_str(&s);
                }
                if in_a_t && let Ok(s) = t.unescape() {
                    a_t_text.push_str(&s);
                }
                if in_v && let Ok(s) = t.unescape() {
                    v_text.push_str(&s);
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                let name = local.as_ref();
                match name {
                    b"chartData" => {
                        in_chart_data = false;
                    }
                    b"strDim" => {
                        in_str_dim = false;
                    }
                    b"numDim" => {
                        in_num_dim = false;
                    }
                    b"lvl" => {
                        in_lvl = false;
                    }
                    b"pt" if in_pt => {
                        if in_str_dim {
                            chart.categories.push(pt_text.clone());
                            chart.colors.push(None);
                        } else if in_num_dim && let Ok(v) = pt_text.parse::<f64>() {
                            chart.values.push(v);
                        }
                        in_pt = false;
                    }
                    b"f" if in_f => {
                        if in_str_dim {
                            chart.cat_range = Some(f_text.clone());
                        } else if in_num_dim {
                            chart.val_range = Some(f_text.clone());
                        }
                        in_f = false;
                    }
                    b"t" if in_a_t => {
                        if in_title {
                            title_text.push_str(&a_t_text);
                        }
                        in_a_t = false;
                    }
                    b"v" if in_v => {
                        if in_tx && in_series {
                            chart.series_name = Some(v_text.clone());
                        }
                        in_v = false;
                    }
                    b"title" if in_title => {
                        if !title_text.is_empty() {
                            chart.title = Some(title_text.clone());
                        }
                        in_title = false;
                    }
                    b"tx" => {
                        in_tx = false;
                    }
                    b"series" => {
                        in_series = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    Ok(chart)
}
