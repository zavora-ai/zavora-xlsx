use crate::features::chart::Chart;
use crate::features::image::Image;
use crate::xml::xml_writer::XmlWriter;

/// EMU = English Metric Units. 1 inch = 914400 EMU. 1 pixel ≈ 9525 EMU at 96 DPI.
const PX_TO_EMU: u64 = 9525;

pub fn write_drawing_xml(charts: &[Chart], images: &[Image], _sheet_idx: usize) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("xdr:wsDr", &[
        ("xmlns:xdr", "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing"),
        ("xmlns:a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
    ]);

    let mut rid = 1;

    // Charts
    for chart in charts {
        let r_id = format!("rId{rid}");
        write_two_cell_anchor_chart(&mut w, chart, &r_id);
        rid += 1;
    }

    // Images
    for image in images {
        let r_id = format!("rId{rid}");
        write_two_cell_anchor_image(&mut w, image, &r_id);
        rid += 1;
    }

    w.end_tag("xdr:wsDr");
    w.into_bytes()
}

pub fn write_drawing_rels(chart_count: usize, image_count: usize, image_types: &[&str], global_chart_start: usize, global_image_start: usize) -> Vec<u8> {
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
    // From
    write_marker(w, "xdr:from", chart.col, chart.row, 0, 0);
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
