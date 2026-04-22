use zavora_xlsx::*;

/// Helper: create a workbook, write a cell with a format, save, reopen readonly, return the Format.
fn roundtrip_format(fmt: &Format) -> Option<Format> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fmt_test.xlsx");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write_with_format(0, 0, "test", fmt).unwrap();
        wb.save(&path).unwrap();
    }

    let mut wb = Workbook::open_readonly(&path).unwrap();
    let ws = wb.worksheet(0).unwrap();
    ws.cell_format(0, 0)
}

#[test]
fn test_bold_italic_font() {
    let fmt = Format::new().bold().italic();
    let read = roundtrip_format(&fmt).expect("expected Some(Format) for bold+italic cell");
    assert!(read.is_bold(), "bold should be true");
    assert!(read.is_italic(), "italic should be true");
}

#[test]
fn test_font_color() {
    let fmt = Format::new().font_color((255u8, 0u8, 0u8));
    let read = roundtrip_format(&fmt).expect("expected Some(Format) for font_color cell");
    assert_eq!(
        read.get_font_color(),
        Some([255, 0, 0]),
        "font_color should be red [255, 0, 0]"
    );
}

#[test]
fn test_border_styles() {
    let fmt = Format::new().border(BorderStyle::Thin);
    let read = roundtrip_format(&fmt).expect("expected Some(Format) for bordered cell");
    assert_eq!(
        read.get_border_left(),
        BorderStyle::Thin,
        "border_left should be Thin"
    );
    assert_eq!(
        read.get_border_right(),
        BorderStyle::Thin,
        "border_right should be Thin"
    );
    assert_eq!(
        read.get_border_top(),
        BorderStyle::Thin,
        "border_top should be Thin"
    );
    assert_eq!(
        read.get_border_bottom(),
        BorderStyle::Thin,
        "border_bottom should be Thin"
    );
}

#[test]
fn test_fill_background_color() {
    let fmt = Format::new().background_color((0u8, 128u8, 255u8));
    let read = roundtrip_format(&fmt).expect("expected Some(Format) for bg_color cell");
    // background_color sets both fg_color and bg_color with solid pattern
    assert_eq!(
        read.get_fg_color(),
        Some([0, 128, 255]),
        "fg_color should match the background color"
    );
    assert_eq!(
        read.get_bg_color(),
        Some([0, 128, 255]),
        "bg_color should match the background color"
    );
}

#[test]
fn test_number_format() {
    let fmt = Format::new().num_format("#,##0.00");
    let read = roundtrip_format(&fmt).expect("expected Some(Format) for num_format cell");
    assert_eq!(
        read.get_num_format(),
        "#,##0.00",
        "num_format should be '#,##0.00'"
    );
}

#[test]
fn test_alignment_center_and_wrap() {
    let fmt = Format::new().align(Align::Center).text_wrap();
    let read = roundtrip_format(&fmt).expect("expected Some(Format) for aligned cell");
    assert_eq!(read.get_h_align(), 2, "h_align should be 2 (center)");
    assert!(read.is_wrap_text(), "wrap_text should be true");
}

#[test]
fn test_plain_cell_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plain_test.xlsx");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "plain text").unwrap();
        wb.save(&path).unwrap();
    }

    let mut wb = Workbook::open_readonly(&path).unwrap();
    let ws = wb.worksheet(0).unwrap();
    assert!(
        ws.cell_format(0, 0).is_none(),
        "cell_format should return None for a plain cell with no format"
    );
}
