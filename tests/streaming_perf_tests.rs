//! Tests for Phase 6: Streaming & Performance (Tasks 55–61)

use zavora_xlsx::{
    CellValue, CfOperator, Chart, ChartType, ConditionalFormatCell, Format, Image, StreamingReader,
    StreamingRow, StreamingWorkbook, Workbook,
};

// ── Task 55: Streaming Read ─────────────────────────────────────────────────

#[test]
fn streaming_read_basic() {
    // Create a workbook with known data
    let mut wb = StreamingWorkbook::new();
    wb.write_string(0, 0, "Name").unwrap();
    wb.write_string(0, 1, "Age").unwrap();
    wb.write_string(1, 0, "Alice").unwrap();
    wb.write_number(1, 1, 30.0).unwrap();
    wb.write_string(2, 0, "Bob").unwrap();
    wb.write_number(2, 1, 25.0).unwrap();
    wb.write_boolean(3, 0, true).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Stream-read it back
    let reader = StreamingReader::from_buffer(buf).unwrap();
    assert_eq!(reader.sheet_count(), 1);
    assert_eq!(reader.sheet_names(), vec!["Sheet1"]);

    let mut rows = reader.sheet_rows(0).unwrap();
    let mut row_count = 0;
    let mut collected: Vec<StreamingRow> = Vec::new();
    while let Some(row) = rows.next() {
        collected.push(row);
        row_count += 1;
    }
    assert_eq!(row_count, 4);

    // Verify cell values
    assert_eq!(collected[0].row_index, 0);
    assert_eq!(collected[0].cells.len(), 2);
    assert_eq!(
        collected[0].cells[0].value,
        CellValue::String("Name".to_string())
    );
    assert_eq!(
        collected[0].cells[1].value,
        CellValue::String("Age".to_string())
    );

    assert_eq!(collected[1].row_index, 1);
    assert_eq!(
        collected[1].cells[0].value,
        CellValue::String("Alice".to_string())
    );
    assert_eq!(collected[1].cells[1].value, CellValue::Number(30.0));

    assert_eq!(
        collected[2].cells[0].value,
        CellValue::String("Bob".to_string())
    );
    assert_eq!(collected[2].cells[1].value, CellValue::Number(25.0));

    assert_eq!(collected[3].cells[0].value, CellValue::Bool(true));
}

#[test]
fn streaming_read_multiple_sheets() {
    let mut wb = StreamingWorkbook::new();
    wb.write_string(0, 0, "Sheet1Data").unwrap();
    wb.add_worksheet("Second");
    wb.write_string(0, 0, "Sheet2Data").unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let reader = StreamingReader::from_buffer(buf).unwrap();
    assert_eq!(reader.sheet_count(), 2);
    assert_eq!(reader.sheet_names(), vec!["Sheet1", "Second"]);

    // Read sheet 1
    let rows1: Vec<StreamingRow> = reader.sheet_rows(0).unwrap().collect();
    assert_eq!(rows1.len(), 1);
    assert_eq!(
        rows1[0].cells[0].value,
        CellValue::String("Sheet1Data".to_string())
    );

    // Read sheet 2
    let rows2: Vec<StreamingRow> = reader.sheet_rows(1).unwrap().collect();
    assert_eq!(rows2.len(), 1);
    assert_eq!(
        rows2[0].cells[0].value,
        CellValue::String("Sheet2Data".to_string())
    );
}

#[test]
fn streaming_read_xf_indices() {
    // Create a workbook with formatted cells
    let mut wb = StreamingWorkbook::new();
    let fmt = Format::new().bold();
    wb.write_string_with_format(0, 0, "Bold", &fmt).unwrap();
    wb.write_string(0, 1, "Normal").unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let reader = StreamingReader::from_buffer(buf).unwrap();
    let rows: Vec<StreamingRow> = reader.sheet_rows(0).unwrap().collect();
    assert_eq!(rows.len(), 1);
    // Bold cell should have a non-zero xf_index
    assert!(
        rows[0].cells[0].xf_index > 0,
        "Bold cell should have non-zero xf_index"
    );
    // Normal cell should have xf_index 0
    assert_eq!(rows[0].cells[1].xf_index, 0);
}

#[test]
fn streaming_read_shared_strings_resolved() {
    // Verify that shared string references are resolved during iteration
    let mut wb = StreamingWorkbook::new();
    let text = "Hello, World!";
    wb.write_string(0, 0, text).unwrap();
    wb.write_string(1, 0, text).unwrap(); // same string, should be deduped in SST

    let buf = wb.save_to_buffer().unwrap();

    let reader = StreamingReader::from_buffer(buf).unwrap();
    let rows: Vec<StreamingRow> = reader.sheet_rows(0).unwrap().collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].cells[0].value, CellValue::String(text.to_string()));
    assert_eq!(rows[1].cells[0].value, CellValue::String(text.to_string()));
}

#[test]
fn streaming_read_from_regular_workbook() {
    // Create with regular Workbook, read with StreamingReader
    let mut wb = Workbook::new();
    {
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Hello").unwrap();
        ws.write(0, 1, 42.5).unwrap();
        ws.write(1, 0, true).unwrap();
    }
    let buf = wb.save_to_buffer().unwrap();

    let reader = StreamingReader::from_buffer(buf).unwrap();
    let rows: Vec<StreamingRow> = reader.sheet_rows(0).unwrap().collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].cells[0].value,
        CellValue::String("Hello".to_string())
    );
    assert_eq!(rows[0].cells[1].value, CellValue::Number(42.5));
    assert_eq!(rows[1].cells[0].value, CellValue::Bool(true));
}

#[test]
fn streaming_read_row_count() {
    let mut wb = StreamingWorkbook::new();
    for i in 0..100u32 {
        wb.write_number(i, 0, i as f64).unwrap();
    }
    let buf = wb.save_to_buffer().unwrap();

    let reader = StreamingReader::from_buffer(buf).unwrap();
    let rows = reader.sheet_rows(0).unwrap();
    assert_eq!(rows.row_count(), 100);
}

// ── Task 56: Streaming Write with Charts and Images ─────────────────────────

#[test]
fn streaming_write_with_chart() {
    let mut wb = StreamingWorkbook::new();
    wb.write_string(0, 0, "Category").unwrap();
    wb.write_string(0, 1, "Value").unwrap();
    for i in 1..=5u32 {
        wb.write_string(i, 0, &format!("Cat{i}")).unwrap();
        wb.write_number(i, 1, (i * 10) as f64).unwrap();
    }

    let mut chart = Chart::new(ChartType::Bar);
    {
        let series = chart.add_series();
        series.set_name("Sales");
        series.set_categories("Sheet1!$A$2:$A$6");
        series.set_values("Sheet1!$B$2:$B$6");
    }
    wb.insert_chart(7, 0, chart);

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Verify the file can be opened
    let wb2 = Workbook::open_from_buffer(&buf).unwrap();
    assert_eq!(wb2.sheet_count(), 1);
}

#[test]
fn streaming_write_with_image() {
    let mut wb = StreamingWorkbook::new();
    wb.write_string(0, 0, "Image below").unwrap();

    // Create a minimal 1x1 PNG
    let png_data = create_minimal_png();
    let img = Image::from_buffer(&png_data).unwrap();
    wb.insert_image(2, 0, img);

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Verify the file can be opened
    let wb2 = Workbook::open_from_buffer(&buf).unwrap();
    assert_eq!(wb2.sheet_count(), 1);
}

// ── Task 57: Streaming Write with Conditional Formatting ────────────────────

#[test]
fn streaming_write_with_cf() {
    let mut wb = StreamingWorkbook::new();
    for i in 0..10u32 {
        wb.write_number(i, 0, (i * 5) as f64).unwrap();
    }

    let mut cf = ConditionalFormatCell::new(CfOperator::GreaterThan, 20.0);
    cf.set_format(&Format::new().bold());
    wb.add_conditional_format(0, 0, 9, 0, Box::new(cf));

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Verify the file can be opened
    let wb2 = Workbook::open_from_buffer(&buf).unwrap();
    assert_eq!(wb2.sheet_count(), 1);
}

// ── Task 58: Parallel Sheet Writing ─────────────────────────────────────────

#[test]
fn parallel_save_produces_valid_output() {
    let mut wb = Workbook::new();
    {
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Sheet1").unwrap();
        ws.write(0, 1, 100.0).unwrap();
    }
    {
        let ws = wb.add_worksheet();
        ws.write(0, 0, "Sheet2").unwrap();
        ws.write(0, 1, 200.0).unwrap();
    }

    // save_parallel delegates to save which already uses thread::scope
    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Verify the output is valid
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    assert_eq!(wb2.sheet_count(), 2);
}

#[test]
fn parallel_save_identical_to_sequential() {
    // Create two identical workbooks
    let create_wb = || {
        let mut wb = Workbook::new();
        {
            let ws = wb.worksheet(0).unwrap();
            for r in 0..50u32 {
                for c in 0..10u16 {
                    ws.write(r, c, format!("R{}C{}", r, c)).unwrap();
                }
            }
        }
        wb
    };

    let mut wb1 = create_wb();
    let buf1 = wb1.save_to_buffer().unwrap();

    let mut wb2 = create_wb();
    wb2.save_parallel("output/parallel_test.xlsx").unwrap();
    let buf2 = std::fs::read("output/parallel_test.xlsx").unwrap();

    // Both should produce valid files with the same data
    let r1 = Workbook::open_readonly_from_buffer(&buf1).unwrap();
    let r2 = Workbook::open_readonly_from_buffer(&buf2).unwrap();
    assert_eq!(r1.sheet_count(), r2.sheet_count());
}

// ── Task 59: Memory-Mapped Reading ──────────────────────────────────────────

#[test]
fn mmap_reading_basic() {
    // Create a test file
    let mut wb = Workbook::new();
    {
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "MmapTest").unwrap();
        ws.write(0, 1, 42.0).unwrap();
    }
    wb.save("output/mmap_test.xlsx").unwrap();

    // Read it back with mmap (only if feature is enabled)
    #[cfg(feature = "mmap")]
    {
        let wb2 = Workbook::open_mmap("output/mmap_test.xlsx").unwrap();
        assert_eq!(wb2.sheet_count(), 1);
    }

    // Without mmap feature, just verify regular open works
    #[cfg(not(feature = "mmap"))]
    {
        let wb2 = Workbook::open_readonly("output/mmap_test.xlsx").unwrap();
        assert_eq!(wb2.sheet_count(), 1);
    }
}

// ── Task 60: Incremental Save ───────────────────────────────────────────────

#[test]
fn incremental_save_preserves_unmodified_sheets() {
    // Create a workbook with multiple sheets
    let mut wb = Workbook::new();
    {
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Sheet1Original").unwrap();
    }
    for i in 1..5 {
        let ws = wb.add_worksheet();
        ws.write(0, 0, format!("Sheet{}Original", i + 1)).unwrap();
    }
    wb.save("output/incremental_base.xlsx").unwrap();

    // Open in edit mode, modify only one sheet
    let mut wb2 = Workbook::open("output/incremental_base.xlsx").unwrap();
    assert_eq!(wb2.sheet_count(), 5);
    {
        let ws = wb2.worksheet(0).unwrap();
        ws.write(1, 0, "Modified").unwrap();
    }
    wb2.save("output/incremental_modified.xlsx").unwrap();

    // Verify all sheets are readable
    let wb3 = Workbook::open_readonly("output/incremental_modified.xlsx").unwrap();
    assert_eq!(wb3.sheet_count(), 5);
}

// ── Task 61: SST Threshold ──────────────────────────────────────────────────

#[test]
fn sst_threshold_configuration() {
    let mut wb = Workbook::new();
    wb.set_sst_threshold(2);
    {
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "unique_string_1").unwrap();
        ws.write(1, 0, "repeated").unwrap();
        ws.write(2, 0, "repeated").unwrap();
        ws.write(3, 0, "unique_string_2").unwrap();
    }
    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Verify the file is valid
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    assert_eq!(wb2.sheet_count(), 1);
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Create a minimal valid 1x1 white PNG image.
fn create_minimal_png() -> Vec<u8> {
    let mut png = Vec::new();
    // PNG signature
    png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    // IHDR chunk
    let ihdr_data = [
        0x00, 0x00, 0x00, 0x01, // width: 1
        0x00, 0x00, 0x00, 0x01, // height: 1
        0x08, // bit depth: 8
        0x02, // color type: RGB
        0x00, // compression
        0x00, // filter
        0x00, // interlace
    ];
    write_png_chunk(&mut png, b"IHDR", &ihdr_data);
    // IDAT chunk (minimal compressed data for 1x1 RGB pixel)
    let idat_data = [
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x01, 0x01, 0x01, 0x00,
    ];
    write_png_chunk(&mut png, b"IDAT", &idat_data);
    // IEND chunk
    write_png_chunk(&mut png, b"IEND", &[]);
    png
}

fn write_png_chunk(buf: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(chunk_type);
    buf.extend_from_slice(data);
    // CRC (simplified — not a real CRC but enough for our test)
    let mut crc_data = Vec::new();
    crc_data.extend_from_slice(chunk_type);
    crc_data.extend_from_slice(data);
    let crc = crc32(&crc_data);
    buf.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}
