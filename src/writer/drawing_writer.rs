use crate::features::chart::Chart;
use crate::features::chartex::ChartExChart;
use crate::features::image::Image;
use crate::features::treemap::TreemapChart;
use crate::xml::xml_writer::XmlWriter;

/// EMU = English Metric Units. 1 inch = 914400 EMU. 1 pixel ≈ 9525 EMU at 96 DPI.
const PX_TO_EMU: u64 = 9525;

pub fn write_drawing_xml(charts: &[Chart], treemaps: &[TreemapChart], chartex_charts: &[ChartExChart], images: &[Image], _sheet_idx: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    let has_chartex = !treemaps.is_empty() || !chartex_charts.is_empty();
    let mut root_attrs: Vec<(&str, &str)> = vec![
        ("xmlns:xdr", "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing"),
        ("xmlns:a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
    ];
    if has_chartex {
        root_attrs.push(("xmlns:mc", "http://schemas.openxmlformats.org/markup-compatibility/2006"));
    }
    w.start_tag("xdr:wsDr", &root_attrs);

    let mut rid = 1;
    let mut obj_id = 2u32;

    for chart in charts {
        let r_id = format!("rId{rid}");
        write_two_cell_anchor_chart(&mut w, chart, &r_id);
        rid += 1; obj_id += 1;
    }

    for tc in treemaps {
        let r_id = format!("rId{rid}");
        write_two_cell_anchor_chartex(&mut w, tc, &r_id, obj_id);
        rid += 1; obj_id += 1;
    }

    for cex in chartex_charts {
        let r_id = format!("rId{rid}");
        write_two_cell_anchor_chartex_generic(&mut w, cex, &r_id, obj_id);
        rid += 1; obj_id += 1;
    }

    for image in images {
        let r_id = format!("rId{rid}");
        write_two_cell_anchor_image(&mut w, image, &r_id);
        rid += 1;
    }

    w.end_tag("xdr:wsDr");
    w.into_bytes()
}

pub fn write_drawing_rels(chart_count: usize, chartex_count: usize, image_count: usize, image_types: &[&str], global_chart_start: usize, global_chartex_start: usize, global_image_start: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("Relationships", &[("xmlns", "http://schemas.openxmlformats.org/package/2006/relationships")]);
    let mut rid = 1;
    for i in 0..chart_count {
        let id = format!("rId{rid}");
        let target = format!("../charts/chart{}.xml", global_chart_start + i + 1);
        w.empty_tag("Relationship", &[("Id", &id), ("Type", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart"), ("Target", &target)]);
        rid += 1;
    }
    for i in 0..chartex_count {
        let id = format!("rId{rid}");
        let target = format!("../charts/chartEx{}.xml", global_chartex_start + i + 1);
        w.empty_tag("Relationship", &[("Id", &id), ("Type", "http://schemas.microsoft.com/office/2014/relationships/chartEx"), ("Target", &target)]);
        rid += 1;
    }
    for i in 0..image_count {
        let id = format!("rId{rid}");
        let target = format!("../media/image{}.{}", global_image_start + i + 1, image_types[i]);
        w.empty_tag("Relationship", &[("Id", &id), ("Type", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"), ("Target", &target)]);
        rid += 1;
    }
    w.end_tag("Relationships");
    w.into_bytes()
}

fn write_two_cell_anchor_chart(w: &mut XmlWriter, chart: &Chart, r_id: &str) {
    w.start_tag("xdr:twoCellAnchor", &[]);
    // From (with pixel offset: 1px ≈ 9525 EMU)
    let x_off = chart.x_offset as u64 * 9525;
    let y_off = chart.y_offset as u64 * 9525;
    write_marker(w, "xdr:from", chart.col, chart.row, x_off, y_off);
    // To: approximate end position based on width/height
    let end_col = chart.col + (chart.width / 64).max(1) as u16; // ~64px per col
    let end_row = chart.row + (chart.height / 20).max(1);       // ~20px per row
    write_marker(w, "xdr:to", end_col, end_row, 0, 0);

    w.start_tag("xdr:graphicFrame", &[("macro", "")]);
    w.start_tag("xdr:nvGraphicFramePr", &[]);
    w.empty_tag("xdr:cNvPr", &[("id", "2"), ("name", "Chart")]);
    w.empty_tag("xdr:cNvGraphicFramePr", &[]);
    w.end_tag("xdr:nvGraphicFramePr");
    w.start_tag("xdr:xfrm", &[]);
    w.empty_tag("a:off", &[("x", "0"), ("y", "0")]);
    w.empty_tag("a:ext", &[("cx", "0"), ("cy", "0")]);
    w.end_tag("xdr:xfrm");
    w.start_tag("a:graphic", &[]);
    w.start_tag("a:graphicData", &[("uri", "http://schemas.openxmlformats.org/drawingml/2006/chart")]);
    w.empty_tag("c:chart", &[("xmlns:c", "http://schemas.openxmlformats.org/drawingml/2006/chart"), ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"), ("r:id", r_id)]);
    w.end_tag("a:graphicData");
    w.end_tag("a:graphic");
    w.end_tag("xdr:graphicFrame");
    w.empty_tag("xdr:clientData", &[]);
    w.end_tag("xdr:twoCellAnchor");
}

fn write_two_cell_anchor_image(w: &mut XmlWriter, image: &Image, r_id: &str) {
    let scaled_w = (image.width_px as f64 * image.scale_width) as u32;
    let scaled_h = (image.height_px as f64 * image.scale_height) as u32;
    let end_col = image.col + (scaled_w / 64).max(1) as u16;
    let end_row = image.row + (scaled_h / 20).max(1);

    w.start_tag("xdr:twoCellAnchor", &[("editAs", "oneCell")]);
    write_marker(w, "xdr:from", image.col, image.row, 0, 0);
    write_marker(w, "xdr:to", end_col, end_row, 0, 0);

    w.start_tag("xdr:pic", &[]);
    w.start_tag("xdr:nvPicPr", &[]);
    w.empty_tag("xdr:cNvPr", &[("id", "2"), ("name", "Image")]);
    w.start_tag("xdr:cNvPicPr", &[]);
    w.empty_tag("a:picLocks", &[("noChangeAspect", "1")]);
    w.end_tag("xdr:cNvPicPr");
    w.end_tag("xdr:nvPicPr");
    w.start_tag("xdr:blipFill", &[]);
    w.empty_tag("a:blip", &[("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"), ("r:embed", r_id)]);
    w.start_tag("a:stretch", &[]);
    w.empty_tag("a:fillRect", &[]);
    w.end_tag("a:stretch");
    w.end_tag("xdr:blipFill");
    w.start_tag("xdr:spPr", &[]);
    let cx = (scaled_w as u64 * PX_TO_EMU).to_string();
    let cy = (scaled_h as u64 * PX_TO_EMU).to_string();
    w.start_tag("a:xfrm", &[]);
    w.empty_tag("a:off", &[("x", "0"), ("y", "0")]);
    w.empty_tag("a:ext", &[("cx", &cx), ("cy", &cy)]);
    w.end_tag("a:xfrm");
    w.start_tag("a:prstGeom", &[("prst", "rect")]);
    w.empty_tag("a:avLst", &[]);
    w.end_tag("a:prstGeom");
    w.end_tag("xdr:spPr");
    w.end_tag("xdr:pic");
    w.empty_tag("xdr:clientData", &[]);
    w.end_tag("xdr:twoCellAnchor");
}

fn write_marker(w: &mut XmlWriter, tag: &str, col: u16, row: u32, col_off: u64, row_off: u64) {
    w.start_tag(tag, &[]);
    w.text_element("xdr:col", &[], &col.to_string());
    w.text_element("xdr:colOff", &[], &col_off.to_string());
    w.text_element("xdr:row", &[], &row.to_string());
    w.text_element("xdr:rowOff", &[], &row_off.to_string());
    w.end_tag(tag);
}

fn write_two_cell_anchor_chartex(w: &mut XmlWriter, tc: &TreemapChart, r_id: &str, obj_id: u32) {
    w.start_tag("xdr:twoCellAnchor", &[]);
    w.start_tag("xdr:from", &[]);
    let col_s = tc.col.to_string(); let row_s = tc.row.to_string();
    w.text_element("xdr:col", &[], &col_s);
    w.text_element("xdr:colOff", &[], "0");
    w.text_element("xdr:row", &[], &row_s);
    w.text_element("xdr:rowOff", &[], "0");
    w.end_tag("xdr:from");

    let end_col = (tc.col as u64 + tc.width as u64 / 64).to_string();
    let end_row = (tc.row as u64 + tc.height as u64 / 20).to_string();
    w.start_tag("xdr:to", &[]);
    w.text_element("xdr:col", &[], &end_col);
    w.text_element("xdr:colOff", &[], "0");
    w.text_element("xdr:row", &[], &end_row);
    w.text_element("xdr:rowOff", &[], "0");
    w.end_tag("xdr:to");

    let id_s = obj_id.to_string();

    w.start_tag("mc:AlternateContent", &[]);
    w.start_tag("mc:Choice", &[("xmlns:cx1", "http://schemas.microsoft.com/office/drawing/2015/9/8/chartex"), ("Requires", "cx1")]);
    w.start_tag("xdr:graphicFrame", &[("macro", "")]);
    w.start_tag("xdr:nvGraphicFramePr", &[]);
    w.empty_tag("xdr:cNvPr", &[("id", &id_s), ("name", "Treemap Chart")]);
    w.empty_tag("xdr:cNvGraphicFramePr", &[]);
    w.end_tag("xdr:nvGraphicFramePr");
    w.start_tag("xdr:xfrm", &[]);
    w.empty_tag("a:off", &[("x", "0"), ("y", "0")]);
    w.empty_tag("a:ext", &[("cx", "0"), ("cy", "0")]);
    w.end_tag("xdr:xfrm");
    w.start_tag("a:graphic", &[]);
    w.start_tag("a:graphicData", &[("uri", "http://schemas.microsoft.com/office/drawing/2014/chartex")]);
    w.empty_tag("cx:chart", &[
        ("xmlns:cx", "http://schemas.microsoft.com/office/drawing/2014/chartex"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
        ("r:id", r_id),
    ]);
    w.end_tag("a:graphicData");
    w.end_tag("a:graphic");
    w.end_tag("xdr:graphicFrame");
    w.end_tag("mc:Choice");
    w.start_tag("mc:Fallback", &[]);
    w.end_tag("mc:Fallback");
    w.end_tag("mc:AlternateContent");

    w.empty_tag("xdr:clientData", &[]);
    w.end_tag("xdr:twoCellAnchor");
}

fn write_two_cell_anchor_chartex_generic(w: &mut XmlWriter, cex: &ChartExChart, r_id: &str, obj_id: u32) {
    let (row, col, width, height) = (cex.row(), cex.col(), cex.width(), cex.height());
    w.start_tag("xdr:twoCellAnchor", &[]);
    w.start_tag("xdr:from", &[]);
    let col_s = col.to_string(); let row_s = row.to_string();
    w.text_element("xdr:col", &[], &col_s);
    w.text_element("xdr:colOff", &[], "0");
    w.text_element("xdr:row", &[], &row_s);
    w.text_element("xdr:rowOff", &[], "0");
    w.end_tag("xdr:from");

    let end_col = (col as u64 + width as u64 / 64).to_string();
    let end_row = (row as u64 + height as u64 / 20).to_string();
    w.start_tag("xdr:to", &[]);
    w.text_element("xdr:col", &[], &end_col);
    w.text_element("xdr:colOff", &[], "0");
    w.text_element("xdr:row", &[], &end_row);
    w.text_element("xdr:rowOff", &[], "0");
    w.end_tag("xdr:to");

    let id_s = obj_id.to_string();
    let chart_name = match cex {
        ChartExChart::Waterfall(_) => "Waterfall Chart",
        ChartExChart::Funnel(_) => "Funnel Chart",
        ChartExChart::Sunburst(_) => "Sunburst Chart",
        ChartExChart::Histogram(_) => "Histogram Chart",
        ChartExChart::BoxWhisker(_) => "Box & Whisker Chart",
    };

    w.start_tag("mc:AlternateContent", &[]);
    w.start_tag("mc:Choice", &[("xmlns:cx1", "http://schemas.microsoft.com/office/drawing/2015/9/8/chartex"), ("Requires", "cx1")]);
    w.start_tag("xdr:graphicFrame", &[("macro", "")]);
    w.start_tag("xdr:nvGraphicFramePr", &[]);
    w.empty_tag("xdr:cNvPr", &[("id", &id_s), ("name", chart_name)]);
    w.empty_tag("xdr:cNvGraphicFramePr", &[]);
    w.end_tag("xdr:nvGraphicFramePr");
    w.start_tag("xdr:xfrm", &[]);
    w.empty_tag("a:off", &[("x", "0"), ("y", "0")]);
    w.empty_tag("a:ext", &[("cx", "0"), ("cy", "0")]);
    w.end_tag("xdr:xfrm");
    w.start_tag("a:graphic", &[]);
    w.start_tag("a:graphicData", &[("uri", "http://schemas.microsoft.com/office/drawing/2014/chartex")]);
    w.empty_tag("cx:chart", &[
        ("xmlns:cx", "http://schemas.microsoft.com/office/drawing/2014/chartex"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
        ("r:id", r_id),
    ]);
    w.end_tag("a:graphicData");
    w.end_tag("a:graphic");
    w.end_tag("xdr:graphicFrame");
    w.end_tag("mc:Choice");
    w.start_tag("mc:Fallback", &[]);
    w.end_tag("mc:Fallback");
    w.end_tag("mc:AlternateContent");

    w.empty_tag("xdr:clientData", &[]);
    w.end_tag("xdr:twoCellAnchor");
}
