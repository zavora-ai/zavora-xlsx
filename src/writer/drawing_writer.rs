use crate::features::chart::Chart;
use crate::features::chartex::ChartExChart;
use crate::features::image::Image;
use crate::features::treemap::TreemapChart;
use crate::xml::xml_writer::XmlWriter;

/// EMU = English Metric Units. 1 inch = 914400 EMU. 1 pixel ≈ 9525 EMU at 96 DPI.
const PX_TO_EMU: u64 = 9525;

pub fn write_drawing_xml(
    charts: &[Chart],
    treemaps: &[TreemapChart],
    chartex_charts: &[ChartExChart],
    images: &[Image],
    _sheet_idx: usize,
) -> Vec<u8> {
    write_drawing_xml_with_shapes(charts, treemaps, chartex_charts, images, &[], _sheet_idx)
}

pub fn write_drawing_xml_with_shapes(
    charts: &[Chart],
    treemaps: &[TreemapChart],
    chartex_charts: &[ChartExChart],
    images: &[Image],
    shapes: &[crate::features::shape::Shape],
    _sheet_idx: usize,
) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    let root_attrs: Vec<(&str, &str)> = vec![
        (
            "xmlns:xdr",
            "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing",
        ),
        (
            "xmlns:a",
            "http://schemas.openxmlformats.org/drawingml/2006/main",
        ),
    ];
    w.start_tag("xdr:wsDr", &root_attrs);

    let mut rid = 1;
    let mut obj_id = 2u32;

    for chart in charts {
        let r_id = format!("rId{rid}");
        write_two_cell_anchor_chart(&mut w, chart, &r_id);
        rid += 1;
        obj_id += 1;
    }

    for tc in treemaps {
        let r_id = format!("rId{rid}");
        write_two_cell_anchor_chartex(&mut w, tc, &r_id, obj_id);
        rid += 1;
        obj_id += 1;
    }

    for cex in chartex_charts {
        let r_id = format!("rId{rid}");
        write_two_cell_anchor_chartex_generic(&mut w, cex, &r_id, obj_id);
        rid += 1;
        obj_id += 1;
    }

    for image in images {
        let r_id = format!("rId{rid}");
        write_two_cell_anchor_image(&mut w, image, &r_id);
        rid += 1;
    }

    for shape in shapes {
        write_two_cell_anchor_shape(&mut w, shape, obj_id);
        obj_id += 1;
    }

    w.end_tag("xdr:wsDr");
    w.into_bytes()
}

pub fn write_drawing_rels(
    chart_count: usize,
    chartex_count: usize,
    image_count: usize,
    image_types: &[&str],
    global_chart_start: usize,
    global_chartex_start: usize,
    global_image_start: usize,
) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "Relationships",
        &[(
            "xmlns",
            "http://schemas.openxmlformats.org/package/2006/relationships",
        )],
    );
    let mut rid = 1;
    for i in 0..chart_count {
        let id = format!("rId{rid}");
        let target = format!("../charts/chart{}.xml", global_chart_start + i + 1);
        w.empty_tag(
            "Relationship",
            &[
                ("Id", &id),
                (
                    "Type",
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart",
                ),
                ("Target", &target),
            ],
        );
        rid += 1;
    }
    for i in 0..chartex_count {
        let id = format!("rId{rid}");
        let target = format!("../charts/chartEx{}.xml", global_chartex_start + i + 1);
        w.empty_tag(
            "Relationship",
            &[
                ("Id", &id),
                (
                    "Type",
                    "http://schemas.microsoft.com/office/2014/relationships/chartEx",
                ),
                ("Target", &target),
            ],
        );
        rid += 1;
    }
    for (i, img_type) in image_types.iter().enumerate().take(image_count) {
        let id = format!("rId{rid}");
        let target = format!("../media/image{}.{}", global_image_start + i + 1, img_type);
        w.empty_tag(
            "Relationship",
            &[
                ("Id", &id),
                (
                    "Type",
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image",
                ),
                ("Target", &target),
            ],
        );
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
    let end_row = chart.row + (chart.height / 20).max(1); // ~20px per row
    write_marker(w, "xdr:to", end_col, end_row, 0, 0);

    w.start_tag("xdr:graphicFrame", &[("macro", "")]);
    w.start_tag("xdr:nvGraphicFramePr", &[]);
    let mut cnv_attrs: Vec<(&str, &str)> = vec![("id", "2"), ("name", "Chart")];
    if let Some((ref title, ref descr)) = chart.alt_text {
        cnv_attrs.push(("title", title));
        cnv_attrs.push(("descr", descr));
    }
    w.empty_tag("xdr:cNvPr", &cnv_attrs);
    w.empty_tag("xdr:cNvGraphicFramePr", &[]);
    w.end_tag("xdr:nvGraphicFramePr");
    w.start_tag("xdr:xfrm", &[]);
    w.empty_tag("a:off", &[("x", "0"), ("y", "0")]);
    w.empty_tag("a:ext", &[("cx", "0"), ("cy", "0")]);
    w.end_tag("xdr:xfrm");
    w.start_tag("a:graphic", &[]);
    w.start_tag(
        "a:graphicData",
        &[(
            "uri",
            "http://schemas.openxmlformats.org/drawingml/2006/chart",
        )],
    );
    w.empty_tag(
        "c:chart",
        &[
            (
                "xmlns:c",
                "http://schemas.openxmlformats.org/drawingml/2006/chart",
            ),
            (
                "xmlns:r",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
            ),
            ("r:id", r_id),
        ],
    );
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
    let mut cnv_attrs: Vec<(&str, &str)> = vec![("id", "2"), ("name", "Image")];
    if let Some((ref title, ref descr)) = image.alt_text {
        cnv_attrs.push(("title", title));
        cnv_attrs.push(("descr", descr));
    }
    w.empty_tag("xdr:cNvPr", &cnv_attrs);
    w.start_tag("xdr:cNvPicPr", &[]);
    w.empty_tag("a:picLocks", &[("noChangeAspect", "1")]);
    w.end_tag("xdr:cNvPicPr");
    w.end_tag("xdr:nvPicPr");
    w.start_tag("xdr:blipFill", &[]);
    w.empty_tag(
        "a:blip",
        &[
            (
                "xmlns:r",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
            ),
            ("r:embed", r_id),
        ],
    );
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
    let col_s = tc.col.to_string();
    let row_s = tc.row.to_string();
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

    w.start_tag(
        "mc:AlternateContent",
        &[(
            "xmlns:mc",
            "http://schemas.openxmlformats.org/markup-compatibility/2006",
        )],
    );
    w.start_tag(
        "mc:Choice",
        &[
            (
                "xmlns:cx1",
                "http://schemas.microsoft.com/office/drawing/2015/9/8/chartex",
            ),
            ("Requires", "cx1"),
        ],
    );
    w.start_tag("xdr:graphicFrame", &[("macro", "")]);
    w.start_tag("xdr:nvGraphicFramePr", &[]);
    w.start_tag("xdr:cNvPr", &[("id", &id_s), ("name", "Treemap Chart")]);
    w.start_tag("a:extLst", &[]);
    w.start_tag(
        "a:ext",
        &[("uri", "{FF2B5EF4-FFF2-40B4-BE49-F238E27FC236}")],
    );
    let tc_uuid = format!("{{00000000-0000-0000-0000-{:012X}}}", obj_id as u64);
    w.empty_tag(
        "a16:creationId",
        &[
            (
                "xmlns:a16",
                "http://schemas.microsoft.com/office/drawing/2014/main",
            ),
            ("id", &tc_uuid),
        ],
    );
    w.end_tag("a:ext");
    w.end_tag("a:extLst");
    w.end_tag("xdr:cNvPr");
    w.empty_tag("xdr:cNvGraphicFramePr", &[]);
    w.end_tag("xdr:nvGraphicFramePr");
    w.start_tag("xdr:xfrm", &[]);
    w.empty_tag("a:off", &[("x", "0"), ("y", "0")]);
    w.empty_tag("a:ext", &[("cx", "0"), ("cy", "0")]);
    w.end_tag("xdr:xfrm");
    w.start_tag("a:graphic", &[]);
    w.start_tag(
        "a:graphicData",
        &[(
            "uri",
            "http://schemas.microsoft.com/office/drawing/2014/chartex",
        )],
    );
    w.empty_tag(
        "cx:chart",
        &[
            (
                "xmlns:cx",
                "http://schemas.microsoft.com/office/drawing/2014/chartex",
            ),
            (
                "xmlns:r",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
            ),
            ("r:id", r_id),
        ],
    );
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

fn write_two_cell_anchor_chartex_generic(
    w: &mut XmlWriter,
    cex: &ChartExChart,
    r_id: &str,
    obj_id: u32,
) {
    let (row, col, width, height) = (cex.row(), cex.col(), cex.width(), cex.height());
    w.start_tag("xdr:twoCellAnchor", &[]);
    w.start_tag("xdr:from", &[]);
    let col_s = col.to_string();
    let row_s = row.to_string();
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
        ChartExChart::Map(_) => "Map Chart",
    };

    // Map charts require cx4 namespace; other ChartEx types use cx1
    let is_map = matches!(cex, ChartExChart::Map(_));
    w.start_tag(
        "mc:AlternateContent",
        &[(
            "xmlns:mc",
            "http://schemas.openxmlformats.org/markup-compatibility/2006",
        )],
    );
    if is_map {
        w.start_tag(
            "mc:Choice",
            &[
                (
                    "xmlns:cx4",
                    "http://schemas.microsoft.com/office/drawing/2016/5/10/chartex",
                ),
                ("Requires", "cx4"),
            ],
        );
    } else {
        w.start_tag(
            "mc:Choice",
            &[
                (
                    "xmlns:cx1",
                    "http://schemas.microsoft.com/office/drawing/2015/9/8/chartex",
                ),
                ("Requires", "cx1"),
            ],
        );
    }
    w.start_tag("xdr:graphicFrame", &[("macro", "")]);
    w.start_tag("xdr:nvGraphicFramePr", &[]);
    w.start_tag("xdr:cNvPr", &[("id", &id_s), ("name", chart_name)]);
    w.start_tag("a:extLst", &[]);
    w.start_tag(
        "a:ext",
        &[("uri", "{FF2B5EF4-FFF2-40B4-BE49-F238E27FC236}")],
    );
    // Generate a deterministic UUID from the object id
    let uuid = format!("{{00000000-0000-0000-0000-{:012X}}}", obj_id as u64);
    w.empty_tag(
        "a16:creationId",
        &[
            (
                "xmlns:a16",
                "http://schemas.microsoft.com/office/drawing/2014/main",
            ),
            ("id", &uuid),
        ],
    );
    w.end_tag("a:ext");
    w.end_tag("a:extLst");
    w.end_tag("xdr:cNvPr");
    w.empty_tag("xdr:cNvGraphicFramePr", &[]);
    w.end_tag("xdr:nvGraphicFramePr");
    w.start_tag("xdr:xfrm", &[]);
    w.empty_tag("a:off", &[("x", "0"), ("y", "0")]);
    w.empty_tag("a:ext", &[("cx", "0"), ("cy", "0")]);
    w.end_tag("xdr:xfrm");
    w.start_tag("a:graphic", &[]);
    w.start_tag(
        "a:graphicData",
        &[(
            "uri",
            "http://schemas.microsoft.com/office/drawing/2014/chartex",
        )],
    );
    w.empty_tag(
        "cx:chart",
        &[
            (
                "xmlns:cx",
                "http://schemas.microsoft.com/office/drawing/2014/chartex",
            ),
            (
                "xmlns:r",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
            ),
            ("r:id", r_id),
        ],
    );
    w.end_tag("a:graphicData");
    w.end_tag("a:graphic");
    w.end_tag("xdr:graphicFrame");
    w.end_tag("mc:Choice");
    w.start_tag("mc:Fallback", &[]);
    // Fallback placeholder for older Excel versions
    w.start_tag("xdr:sp", &[("macro", ""), ("textlink", "")]);
    w.start_tag("xdr:nvSpPr", &[]);
    w.empty_tag("xdr:cNvPr", &[("id", "0"), ("name", "")]);
    w.start_tag("xdr:cNvSpPr", &[]);
    w.empty_tag("a:spLocks", &[("noTextEdit", "1")]);
    w.end_tag("xdr:cNvSpPr");
    w.end_tag("xdr:nvSpPr");
    w.start_tag("xdr:spPr", &[]);
    w.start_tag("a:xfrm", &[]);
    w.empty_tag("a:off", &[("x", "0"), ("y", "0")]);
    w.empty_tag("a:ext", &[("cx", "0"), ("cy", "0")]);
    w.end_tag("a:xfrm");
    w.start_tag("a:prstGeom", &[("prst", "rect")]);
    w.empty_tag("a:avLst", &[]);
    w.end_tag("a:prstGeom");
    w.start_tag("a:solidFill", &[]);
    w.empty_tag("a:prstClr", &[("val", "white")]);
    w.end_tag("a:solidFill");
    w.start_tag("a:ln", &[("w", "1")]);
    w.start_tag("a:solidFill", &[]);
    w.empty_tag("a:prstClr", &[("val", "green")]);
    w.end_tag("a:solidFill");
    w.end_tag("a:ln");
    w.end_tag("xdr:spPr");
    w.start_tag("xdr:txBody", &[]);
    w.empty_tag(
        "a:bodyPr",
        &[("vertOverflow", "clip"), ("horzOverflow", "clip")],
    );
    w.empty_tag("a:lstStyle", &[]);
    w.start_tag("a:p", &[]);
    w.start_tag("a:r", &[]);
    w.empty_tag("a:rPr", &[("lang", "en-US"), ("sz", "1100")]);
    w.text_element(
        "a:t",
        &[],
        "This chart isn't available in your version of Excel.",
    );
    w.end_tag("a:r");
    w.end_tag("a:p");
    w.end_tag("xdr:txBody");
    w.end_tag("xdr:sp");
    w.end_tag("mc:Fallback");
    w.end_tag("mc:AlternateContent");

    w.empty_tag("xdr:clientData", &[]);
    w.end_tag("xdr:twoCellAnchor");
}

fn write_two_cell_anchor_shape(
    w: &mut XmlWriter,
    shape: &crate::features::shape::Shape,
    obj_id: u32,
) {
    let end_col = shape.col + (shape.width / 64).max(1) as u16;
    let end_row = shape.row + (shape.height / 20).max(1);

    w.start_tag("xdr:twoCellAnchor", &[]);
    write_marker(w, "xdr:from", shape.col, shape.row, 0, 0);
    write_marker(w, "xdr:to", end_col, end_row, 0, 0);

    let id_s = obj_id.to_string();
    let name = format!("Shape {}", obj_id);
    w.start_tag("xdr:sp", &[("macro", ""), ("textlink", "")]);
    w.start_tag("xdr:nvSpPr", &[]);
    w.empty_tag("xdr:cNvPr", &[("id", &id_s), ("name", &name)]);
    w.empty_tag("xdr:cNvSpPr", &[]);
    w.end_tag("xdr:nvSpPr");

    // Shape properties
    w.start_tag("xdr:spPr", &[]);
    let cx = (shape.width as u64 * PX_TO_EMU).to_string();
    let cy = (shape.height as u64 * PX_TO_EMU).to_string();
    w.start_tag("a:xfrm", &[]);
    w.empty_tag("a:off", &[("x", "0"), ("y", "0")]);
    w.empty_tag("a:ext", &[("cx", &cx), ("cy", &cy)]);
    w.end_tag("a:xfrm");

    let prst = shape.shape_type.preset_name();
    w.start_tag("a:prstGeom", &[("prst", prst)]);
    w.empty_tag("a:avLst", &[]);
    w.end_tag("a:prstGeom");

    // Fill
    if let Some(rgb) = shape.fill_color {
        let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        w.start_tag("a:solidFill", &[]);
        w.empty_tag("a:srgbClr", &[("val", &hex)]);
        w.end_tag("a:solidFill");
    }

    // Outline
    if shape.outline_color.is_some() || shape.outline_width.is_some() {
        let width_emu = ((shape.outline_width.unwrap_or(1.0) * 12700.0) as u64).to_string();
        w.start_tag("a:ln", &[("w", &width_emu)]);
        if let Some(rgb) = shape.outline_color {
            let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
            w.start_tag("a:solidFill", &[]);
            w.empty_tag("a:srgbClr", &[("val", &hex)]);
            w.end_tag("a:solidFill");
        }
        w.end_tag("a:ln");
    }

    w.end_tag("xdr:spPr");

    // Text body
    if let Some(ref text) = shape.text {
        w.start_tag("xdr:txBody", &[]);
        w.empty_tag(
            "a:bodyPr",
            &[
                ("vertOverflow", "clip"),
                ("horzOverflow", "clip"),
                ("wrap", "square"),
                ("rtlCol", "0"),
                ("anchor", "ctr"),
            ],
        );
        w.empty_tag("a:lstStyle", &[]);
        w.start_tag("a:p", &[]);
        w.start_tag("a:r", &[]);
        let font_size = ((shape.font_size.unwrap_or(11.0) * 100.0) as u32).to_string();
        if shape.font_bold {
            w.empty_tag(
                "a:rPr",
                &[("lang", "en-US"), ("sz", &font_size), ("b", "1")],
            );
        } else {
            w.empty_tag("a:rPr", &[("lang", "en-US"), ("sz", &font_size)]);
        }
        w.text_element("a:t", &[], text);
        w.end_tag("a:r");
        w.end_tag("a:p");
        w.end_tag("xdr:txBody");
    }

    w.end_tag("xdr:sp");
    w.empty_tag("xdr:clientData", &[]);
    w.end_tag("xdr:twoCellAnchor");
}
