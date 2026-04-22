/// Tests for Tasks 69-76: Threaded Comments, Form Controls, ActiveX/OLE Preservation,
/// Drawing Shapes, SmartArt/Equation Preservation, Digital Signatures,
/// Custom XML Parts, Custom Document Properties.
use zavora_xlsx::{
    CustomProperty, CustomPropertyValue, FormControl, Shape, ShapeType, ThreadedComment, Workbook,
};

// ── Task 69: Threaded Comments ──

#[test]
fn test_threaded_comment_struct() {
    let mut tc = ThreadedComment::new("Alice", "Initial comment");
    tc.add_reply("Bob", "Reply from Bob");
    tc.add_reply_with_timestamp("Charlie", "Reply from Charlie", "2024-06-15T10:30:00.000");

    assert_eq!(tc.author, "Alice");
    assert_eq!(tc.text, "Initial comment");
    assert_eq!(tc.replies.len(), 2);
    assert_eq!(tc.replies[0].author, "Bob");
    assert_eq!(tc.replies[0].text, "Reply from Bob");
    assert_eq!(tc.replies[1].author, "Charlie");
    assert_eq!(tc.replies[1].timestamp, "2024-06-15T10:30:00.000");
}

#[test]
fn test_threaded_comment_add_to_worksheet() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut tc = ThreadedComment::new("Alice", "Hello thread");
    tc.add_reply("Bob", "Hi Alice!");

    ws.add_threaded_comment(0, 0, tc);
    assert_eq!(ws.threaded_comments().len(), 1);
    assert_eq!(ws.threaded_comments()[0].row, 0);
    assert_eq!(ws.threaded_comments()[0].col, 0);
    assert_eq!(ws.threaded_comments()[0].text, "Hello thread");
    assert_eq!(ws.threaded_comments()[0].replies.len(), 1);
}

#[test]
fn test_threaded_comment_save_roundtrip() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Cell A1").unwrap();

    let mut tc = ThreadedComment::new("Alice", "Review this cell");
    tc.add_reply("Bob", "Looks good!");
    ws.add_threaded_comment(0, 0, tc);

    let buf = wb.save_to_buffer().unwrap();
    // Verify the file can be saved without errors
    assert!(!buf.is_empty());

    // Verify the ZIP contains threaded comments parts
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(names.iter().any(|n| n.contains("threadedComment")));
    assert!(names.iter().any(|n| n.contains("persons/person.xml")));
}

// ── Task 70: Form Controls ──

#[test]
fn test_form_control_checkbox() {
    let ctrl = FormControl::checkbox("Accept Terms");
    match &ctrl {
        FormControl::Checkbox {
            text,
            checked,
            cell_link,
        } => {
            assert_eq!(text, "Accept Terms");
            assert!(!checked);
            assert!(cell_link.is_none());
        }
        _ => panic!("Expected Checkbox"),
    }
}

#[test]
fn test_form_control_checkbox_with_cell_link() {
    let ctrl = FormControl::checkbox_with_link("Enable", "$A$1");
    match &ctrl {
        FormControl::Checkbox {
            text, cell_link, ..
        } => {
            assert_eq!(text, "Enable");
            assert_eq!(cell_link.as_deref(), Some("$A$1"));
        }
        _ => panic!("Expected Checkbox"),
    }
}

#[test]
fn test_form_control_spinner_with_cell_link() {
    let ctrl = FormControl::spinner_with_link(0, 100, 50, "$B$1");
    match &ctrl {
        FormControl::Spinner {
            min_value,
            max_value,
            current_value,
            cell_link,
            ..
        } => {
            assert_eq!(*min_value, 0);
            assert_eq!(*max_value, 100);
            assert_eq!(*current_value, 50);
            assert_eq!(cell_link.as_deref(), Some("$B$1"));
        }
        _ => panic!("Expected Spinner"),
    }
}

#[test]
fn test_form_control_add_to_worksheet() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.add_form_control(0, 0, FormControl::checkbox_with_link("Check", "$C$1"));
    assert_eq!(ws.form_controls().len(), 1);
}

#[test]
fn test_form_control_save() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Label").unwrap();
    ws.add_form_control(1, 0, FormControl::checkbox_with_link("Accept", "$A$2"));

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Verify VML drawing is present
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(names.iter().any(|n| n.contains("vmlDrawing")));
}

// ── Task 71: ActiveX and OLE Preservation ──

#[test]
fn test_activex_ole_preservation_via_passthrough() {
    let mut wb = Workbook::new();
    // Simulate ActiveX parts being preserved via passthrough
    wb.add_passthrough_entry("xl/activeX/activeX1.xml", b"<activex/>".to_vec());
    wb.add_passthrough_entry("xl/activeX/activeX1.bin", b"\x00\x01\x02".to_vec());
    // Simulate OLE parts
    wb.add_passthrough_entry("xl/embeddings/oleObject1.bin", b"\x00\x01\x02\x03".to_vec());

    let buf = wb.save_to_buffer().unwrap();
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();

    assert!(names.iter().any(|n| n.contains("activeX/activeX1.xml")));
    assert!(names.iter().any(|n| n.contains("activeX/activeX1.bin")));
    assert!(
        names
            .iter()
            .any(|n| n.contains("embeddings/oleObject1.bin"))
    );
}

// ── Task 72: Drawing Shapes ──

#[test]
fn test_shape_struct() {
    let shape = Shape::new(ShapeType::Rectangle, 200, 100)
        .text("Hello")
        .fill_color([0xFF, 0x00, 0x00])
        .outline_color([0x00, 0x00, 0xFF])
        .outline_width(2.0)
        .font_size(14.0)
        .bold();

    assert_eq!(shape.shape_type, ShapeType::Rectangle);
    assert_eq!(shape.width, 200);
    assert_eq!(shape.height, 100);
    assert_eq!(shape.text.as_deref(), Some("Hello"));
    assert_eq!(shape.fill_color, Some([0xFF, 0x00, 0x00]));
    assert_eq!(shape.outline_color, Some([0x00, 0x00, 0xFF]));
    assert_eq!(shape.outline_width, Some(2.0));
    assert!(shape.font_bold);
}

#[test]
fn test_shape_preset_names() {
    assert_eq!(ShapeType::Rectangle.preset_name(), "rect");
    assert_eq!(ShapeType::RoundedRectangle.preset_name(), "roundRect");
    assert_eq!(ShapeType::Ellipse.preset_name(), "ellipse");
    assert_eq!(ShapeType::Triangle.preset_name(), "triangle");
    assert_eq!(ShapeType::Diamond.preset_name(), "diamond");
    assert_eq!(ShapeType::Arrow.preset_name(), "rightArrow");
    assert_eq!(ShapeType::Callout.preset_name(), "wedgeRoundRectCallout");
    assert_eq!(ShapeType::TextBox.preset_name(), "rect");
}

#[test]
fn test_shape_add_to_worksheet() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    let shape = Shape::new(ShapeType::Rectangle, 200, 100).text("Box");
    ws.add_shape(2, 1, &shape).unwrap();
    assert_eq!(ws.shapes().len(), 1);
    assert_eq!(ws.shapes()[0].row, 2);
    assert_eq!(ws.shapes()[0].col, 1);
}

#[test]
fn test_shape_save_drawing_xml() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Data").unwrap();
    let shape = Shape::new(ShapeType::Rectangle, 200, 100)
        .text("Hello Shape")
        .fill_color([0x44, 0x72, 0xC4]);
    ws.add_shape(2, 2, &shape).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Verify drawing XML is present
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(names.iter().any(|n| n.contains("drawings/drawing1.xml")));

    // Read the drawing XML and verify it contains xdr:sp
    let mut drawing_file = archive.by_name("xl/drawings/drawing1.xml").unwrap();
    let mut drawing_xml = String::new();
    std::io::Read::read_to_string(&mut drawing_file, &mut drawing_xml).unwrap();
    assert!(drawing_xml.contains("xdr:sp"));
    assert!(drawing_xml.contains("a:prstGeom"));
    assert!(drawing_xml.contains("rect"));
    assert!(drawing_xml.contains("Hello Shape"));
}

// ── Task 73: SmartArt and Equation Preservation ──

#[test]
fn test_smartart_preservation_via_passthrough() {
    let mut wb = Workbook::new();
    // Simulate SmartArt diagram parts
    wb.add_passthrough_entry("xl/diagrams/data1.xml", b"<dgm:dataModel/>".to_vec());
    wb.add_passthrough_entry("xl/diagrams/colors1.xml", b"<dgm:colorsDef/>".to_vec());
    wb.add_passthrough_entry("xl/diagrams/layout1.xml", b"<dgm:layoutDef/>".to_vec());
    wb.add_passthrough_entry("xl/diagrams/drawing1.xml", b"<dsp:drawing/>".to_vec());

    let buf = wb.save_to_buffer().unwrap();
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();

    assert!(names.iter().any(|n| n.contains("diagrams/data1.xml")));
    assert!(names.iter().any(|n| n.contains("diagrams/colors1.xml")));
    assert!(names.iter().any(|n| n.contains("diagrams/layout1.xml")));
    assert!(names.iter().any(|n| n.contains("diagrams/drawing1.xml")));
}

// ── Task 74: Digital Signatures ──

#[test]
fn test_digital_signature_skeleton_sign() {
    let mut wb = Workbook::new();
    let result = wb.sign(b"fake-certificate");
    assert!(result.is_err());
    match result.unwrap_err() {
        zavora_xlsx::Error::UnsupportedFormat(msg) => {
            assert!(msg.contains("Digital signatures"));
        }
        other => panic!("Expected UnsupportedFormat, got: {:?}", other),
    }
}

#[test]
fn test_digital_signature_skeleton_verify() {
    let wb = Workbook::new();
    let result = wb.verify_signature();
    assert!(result.is_err());
    match result.unwrap_err() {
        zavora_xlsx::Error::UnsupportedFormat(msg) => {
            assert!(msg.contains("Digital signature verification"));
        }
        other => panic!("Expected UnsupportedFormat, got: {:?}", other),
    }
}

// ── Task 75: Custom XML Parts ──

#[test]
fn test_custom_xml_add_and_read() {
    let mut wb = Workbook::new();
    let ns = "http://example.com/custom";
    let content = b"<root><item>Hello</item></root>";
    wb.add_custom_xml(ns, content);

    let read_back = wb.read_custom_xml(ns);
    assert!(read_back.is_some());
    assert_eq!(read_back.unwrap(), content);
}

#[test]
fn test_custom_xml_read_nonexistent() {
    let wb = Workbook::new();
    assert!(wb.read_custom_xml("http://nonexistent.com").is_none());
}

#[test]
fn test_custom_xml_save_roundtrip() {
    let mut wb = Workbook::new();
    let ns = "http://example.com/schema";
    let content = b"<data><value>42</value></data>";
    wb.add_custom_xml(ns, content);

    let buf = wb.save_to_buffer().unwrap();
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();

    assert!(names.iter().any(|n| n.contains("customXml/item1.xml")));
    assert!(names.iter().any(|n| n.contains("customXml/itemProps1.xml")));

    // Read back the custom XML content
    let mut item_file = archive.by_name("customXml/item1.xml").unwrap();
    let mut item_content = Vec::new();
    std::io::Read::read_to_end(&mut item_file, &mut item_content).unwrap();
    assert_eq!(item_content, content);
}

// ── Task 76: Custom Document Properties ──

#[test]
fn test_custom_property_struct() {
    let prop = CustomProperty::text("Project", "Alpha");
    assert_eq!(prop.name, "Project");
    assert_eq!(prop.value, CustomPropertyValue::Text("Alpha".to_string()));

    let prop_num = CustomProperty::number("Version", 1.5);
    assert_eq!(prop_num.name, "Version");
    assert_eq!(prop_num.value, CustomPropertyValue::Number(1.5));

    let prop_int = CustomProperty::integer("Count", 42);
    assert_eq!(prop_int.name, "Count");
    assert_eq!(prop_int.value, CustomPropertyValue::Integer(42));

    let prop_bool = CustomProperty::bool("Approved", true);
    assert_eq!(prop_bool.name, "Approved");
    assert_eq!(prop_bool.value, CustomPropertyValue::Bool(true));
}

#[test]
fn test_custom_property_set_and_get() {
    let mut wb = Workbook::new();
    wb.set_custom_property("Project", CustomPropertyValue::Text("Alpha".to_string()));
    wb.set_custom_property("Version", CustomPropertyValue::Number(2.0));

    assert_eq!(wb.custom_properties().len(), 2);
    assert_eq!(wb.custom_properties()[0].name, "Project");
    assert_eq!(wb.custom_properties()[1].name, "Version");
}

#[test]
fn test_custom_property_replace_existing() {
    let mut wb = Workbook::new();
    wb.set_custom_property("Key", CustomPropertyValue::Text("Old".to_string()));
    wb.set_custom_property("Key", CustomPropertyValue::Text("New".to_string()));

    assert_eq!(wb.custom_properties().len(), 1);
    assert_eq!(
        wb.custom_properties()[0].value,
        CustomPropertyValue::Text("New".to_string())
    );
}

#[test]
fn test_custom_property_save_roundtrip() {
    let mut wb = Workbook::new();
    wb.set_custom_property("Project", CustomPropertyValue::Text("Test".to_string()));
    wb.set_custom_property("Version", CustomPropertyValue::Number(1.0));
    wb.set_custom_property("Approved", CustomPropertyValue::Bool(true));

    let buf = wb.save_to_buffer().unwrap();
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();

    assert!(names.iter().any(|n| n.contains("docProps/custom.xml")));

    // Read and parse the custom.xml
    let mut custom_file = archive.by_name("docProps/custom.xml").unwrap();
    let mut custom_xml = Vec::new();
    std::io::Read::read_to_end(&mut custom_file, &mut custom_xml).unwrap();

    let props = zavora_xlsx::properties::parse_custom_xml(&custom_xml);
    assert_eq!(props.len(), 3);
    assert_eq!(props[0].name, "Project");
    assert_eq!(
        props[0].value,
        CustomPropertyValue::Text("Test".to_string())
    );
    assert_eq!(props[1].name, "Version");
    assert_eq!(props[1].value, CustomPropertyValue::Number(1.0));
    assert_eq!(props[2].name, "Approved");
    assert_eq!(props[2].value, CustomPropertyValue::Bool(true));
}

#[test]
fn test_custom_property_parse_xml() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
  <property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="2" name="Department">
    <vt:lpwstr>Engineering</vt:lpwstr>
  </property>
  <property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="3" name="Score">
    <vt:r8>95.5</vt:r8>
  </property>
  <property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="4" name="Active">
    <vt:bool>true</vt:bool>
  </property>
</Properties>"#;

    let props = zavora_xlsx::properties::parse_custom_xml(xml);
    assert_eq!(props.len(), 3);
    assert_eq!(props[0].name, "Department");
    assert_eq!(
        props[0].value,
        CustomPropertyValue::Text("Engineering".to_string())
    );
    assert_eq!(props[1].name, "Score");
    assert_eq!(props[1].value, CustomPropertyValue::Number(95.5));
    assert_eq!(props[2].name, "Active");
    assert_eq!(props[2].value, CustomPropertyValue::Bool(true));
}

// ── Integration: Multiple features combined ──

#[test]
fn test_combined_features_save() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Data").unwrap();

    // Add a threaded comment
    let mut tc = ThreadedComment::new("Alice", "Check this");
    tc.add_reply("Bob", "Done!");
    ws.add_threaded_comment(0, 0, tc);

    // Add a shape
    let shape = Shape::new(ShapeType::Ellipse, 150, 150).fill_color([0x00, 0xFF, 0x00]);
    ws.add_shape(3, 3, &shape).unwrap();

    // Add custom properties
    wb.set_custom_property(
        "Author",
        CustomPropertyValue::Text("Test Suite".to_string()),
    );

    // Add custom XML
    wb.add_custom_xml("http://test.com/ns", b"<test/>");

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Verify all parts are present
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();

    assert!(names.iter().any(|n| n.contains("threadedComment")));
    assert!(names.iter().any(|n| n.contains("drawings/drawing1.xml")));
    assert!(names.iter().any(|n| n.contains("docProps/custom.xml")));
    assert!(names.iter().any(|n| n.contains("customXml/item1.xml")));
}
