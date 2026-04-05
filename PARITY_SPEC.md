# Feature Parity Spec: Closing Gaps with Reference Crates

Gap analysis comparing zavora-xlsx against rust_xlsxwriter (v0.94), calamine (v0.34), and umya-spreadsheet (v2.3.3). Organized into sprints by effort and impact.

---

## Sprint 7 — Quick Wins (1–5 lines each)

Estimated: ~80 LOC total. All are single-attribute or trivial method additions.

### 7.1 Set Active Sheet
- **Gap**: No way to control which sheet is selected when workbook opens
- **Reference**: `rust_xlsxwriter` → `worksheet.set_active()`
- **API**: `wb.set_active_sheet(index: usize)`
- **XML**: `<sheet>` gets `state` attr; `<workbookView>` gets `activeTab="N"` in workbook.xml; `<sheetView tabSelected="1">` on the active sheet
- **Impact**: Every financial model opens to the Dashboard tab

### 7.2 Write Blank with Format
- **Gap**: Can't create a formatted empty cell explicitly
- **Reference**: `rust_xlsxwriter` → `write_blank(row, col, &format)`
- **API**: `ws.write_blank(row, col, &format)?`
- **XML**: `<c r="A1" s="3"/>` — cell with style index, no `<v>` element
- **Impact**: Formatted borders/backgrounds on empty cells (common in templates)

### 7.3 Clear Cell
- **Gap**: No way to remove a cell value in edit mode
- **Reference**: `rust_xlsxwriter` → `clear_cell(row, col)`
- **API**: `ws.clear_cell(row, col)?`
- **Impl**: Remove entry from `self.cells` BTreeMap
- **Impact**: Edit mode: delete values while preserving structure

### 7.4 Set Default Row Height
- **Gap**: `<sheetFormatPr>` exists in writer but `defaultRowHeight` not exposed
- **Reference**: `rust_xlsxwriter` → `set_default_row_height(height)`
- **API**: `ws.set_default_row_height(15.0)`
- **XML**: `<sheetFormatPr defaultRowHeight="15"/>` — already written, just wire the value
- **Impact**: Consistent row sizing across sheets

### 7.5 Set Column Format
- **Gap**: Formatting entire column requires `set_range_format` over all rows
- **Reference**: `rust_xlsxwriter` → `set_column_format(col, &format)`
- **API**: `ws.set_column_format(col, &format)?`
- **XML**: `<col>` element gets `style="N"` attribute
- **Impact**: Efficient number format on entire column (currency, dates)

### 7.6 Set Row Format
- **Gap**: No way to format an entire row
- **Reference**: `rust_xlsxwriter` → `set_row_format(row, &format)`
- **API**: `ws.set_row_format(row, &format)?`
- **XML**: `<row>` element gets `s="N" customFormat="1"` attributes
- **Impact**: Header row formatting without per-cell expansion

### 7.7 Very Hidden Sheets
- **Gap**: Can only show/hide sheets, not "very hide" them (inaccessible via UI)
- **Reference**: `rust_xlsxwriter` → `set_very_hidden()`; calamine reads visibility
- **API**: `ws.set_hidden()`, `ws.set_very_hidden()`
- **XML**: `<sheet state="hidden"/>` or `<sheet state="veryHidden"/>` in workbook.xml
- **Impact**: Config/lookup sheets that users shouldn't see or unhide

### 7.8 Set Selection (Active Cell)
- **Gap**: No control over cursor position when sheet opens
- **Reference**: `rust_xlsxwriter` → `set_selection(row, col, last_row, last_col)`
- **API**: `ws.set_selection(row, col)`
- **XML**: `<selection activeCell="B2" sqref="B2"/>` inside `<sheetView>`
- **Impact**: UX polish — cursor on first input cell

### 7.9 Set Top-Left Cell
- **Gap**: No control over scroll position when sheet opens
- **Reference**: `rust_xlsxwriter` → `set_top_left_cell(row, col)`
- **API**: `ws.set_top_left_cell(row, col)`
- **XML**: `<sheetView topLeftCell="A10"/>` attribute on existing element
- **Impact**: Large sheets scroll to relevant area on open

### 7.10 Ignore Error Indicators
- **Gap**: Green triangle error indicators can't be suppressed
- **Reference**: `rust_xlsxwriter` → `ignore_error(type, range)`
- **API**: `ws.ignore_error("numberStoredAsText", "A1:A100")`
- **XML**: `<ignoredErrors><ignoredError sqref="A1:A100" numberStoredAsText="1"/></ignoredErrors>` after `<dataValidations>`
- **Impact**: Clean sheets without distracting green triangles on IDs, zip codes

---

## Sprint 8 — Write Features (10–30 lines each)

Estimated: ~200 LOC total.

### 8.1 Array Formulas
- **Gap**: No support for legacy array formulas (Ctrl+Shift+Enter)
- **Reference**: `rust_xlsxwriter` → `write_array_formula(r1, c1, r2, c2, formula)`
- **API**: `ws.write_array_formula(r1, c1, r2, c2, "MMULT(A1:B2,C1:D2)")?`
- **XML**: `<f t="array" ref="E1:F2">MMULT(A1:B2,C1:D2)</f>` — first cell gets the formula with `t="array"` and `ref` spanning the result range
- **Impact**: MMULT, TRANSPOSE, legacy array operations in financial models

### 8.2 Dynamic Array Formulas
- **Gap**: No support for Excel 365 spill formulas (UNIQUE, SORT, FILTER, XLOOKUP)
- **Reference**: `rust_xlsxwriter` → `write_dynamic_array_formula(r1, c1, r2, c2, formula)`
- **API**: `ws.write_dynamic_formula(row, col, "UNIQUE(A1:A100)")?`
- **XML**: `<f t="array" ref="E1" cm="1">_xlfn.UNIQUE(A1:A100)</f>` + `<extLst>` with `dynamicArrayProperties` metadata
- **Note**: Many Excel 365 functions need `_xlfn.` or `_xlfn._xlws.` prefix
- **Impact**: Modern Excel formulas — XLOOKUP, UNIQUE, SORT, FILTER, SEQUENCE

### 8.3 Set Formula Cached Result
- **Gap**: Formulas show `#VALUE!` or `0` until Excel recalculates
- **Reference**: `rust_xlsxwriter` → `set_formula_result(row, col, "42")`
- **API**: `ws.write_formula_with_result(row, col, "SUM(A1:A10)", 150.0)?`
- **XML**: `<c><f>SUM(A1:A10)</f><v>150</v></c>` — add `<v>` element with cached value
- **Impact**: Formulas display correct values immediately without recalc; critical for PDF export

### 8.4 Open from Buffer
- **Gap**: Can only open from file path, not from `&[u8]` or `Vec<u8>`
- **Reference**: calamine → `open_workbook_from_rs(reader)`; umya → `reader::xlsx::read(reader)`
- **API**: `Workbook::open_from_buffer(bytes: &[u8])?`, `Workbook::open_readonly_from_buffer(bytes: &[u8])?`
- **Impl**: Wrap `std::io::Cursor<&[u8]>` and pass to `ZipReader::new()`
- **Impact**: MCP server processes xlsx from memory without temp files

### 8.5 Print Options
- **Gap**: Missing print gridlines, headings, centering, page order, first page number
- **Reference**: `rust_xlsxwriter` → `set_print_gridlines()`, `set_print_headings()`, `set_print_center_horizontally()`, etc.
- **API**: Extend `PrintSettings` builder:
  ```
  PrintSettings::new()
      .print_gridlines(true)
      .print_headings(true)
      .center_horizontally(true)
      .center_vertically(true)
      .black_and_white(true)
      .page_order_over_then_down(true)
      .first_page_number(1)
  ```
- **XML**: `<printOptions gridLines="1" headings="1" horizontalCentered="1" verticalCentered="1"/>` before `<pageMargins>`; `<pageSetup firstPageNumber="1" useFirstPageNumber="1" pageOrder="overThenDown" blackAndWhite="1"/>`
- **Impact**: Professional PDF/print output for financial reports

### 8.6 Unprotect Range
- **Gap**: Can't allow editing specific ranges on protected sheets
- **Reference**: `rust_xlsxwriter` → `unprotect_range(name, range)`, `unprotect_range_with_options()`
- **API**: `ws.unprotect_range("Inputs", "B2:B20")?`, `ws.unprotect_range_with_password("Inputs", "B2:B20", "pass")?`
- **XML**: `<protectedRanges><protectedRange sqref="B2:B20" name="Inputs"/></protectedRanges>` in sheet XML after `<sheetProtection>`
- **Impact**: Financial models: protect formulas, allow input cells

### 8.7 Filter Column Criteria
- **Gap**: Autofilter exists but can't set specific filter criteria per column
- **Reference**: `rust_xlsxwriter` → `filter_column(col, &criteria)`
- **API**: `ws.filter_column(2, &["Active", "Pending"])?`
- **XML**: `<autoFilter><filterColumn colId="2"><filters><filter val="Active"/><filter val="Pending"/></filters></filterColumn></autoFilter>`
- **Impact**: Pre-filtered views in reports

---

## Sprint 9 — Format Additions (10–40 lines each)

Estimated: ~150 LOC total.

### 9.1 Diagonal Borders
- **Gap**: No diagonal border support
- **Reference**: `rust_xlsxwriter` → `set_border_diagonal()`, `set_border_diagonal_type()`
- **API**: `Format::new().diagonal_border(BorderStyle::Thin, DiagonalType::Up)`
- **Enums**: `DiagonalType { Up, Down, Both }`
- **XML**: `<border diagonalUp="1"><diagonal style="thin"><color rgb="FF000000"/></diagonal></border>` in styles.xml
- **Impact**: Financial statement separators, crossed-out cells

### 9.2 Pattern Fills (Full Set)
- **Gap**: Only 3 patterns (None, Solid, Gray125) vs 18 in Excel
- **Reference**: `rust_xlsxwriter` → `FormatPattern` enum with 18 variants
- **API**: Extend `Pattern` enum:
  ```
  Pattern { None, Solid, Gray125, MediumGray, DarkGray, LightGray,
            DarkHorizontal, DarkVertical, DarkDown, DarkUp, DarkGrid, DarkTrellis,
            LightHorizontal, LightVertical, LightDown, LightUp, LightGrid, LightTrellis }
  ```
- **XML**: `<patternFill patternType="darkDown"/>` — map enum to XML string
- **Impact**: Print-friendly shading, visual distinction without color

### 9.3 Superscript / Subscript
- **Gap**: No font script support
- **Reference**: `rust_xlsxwriter` → `FormatScript::Superscript`, `FormatScript::Subscript`
- **API**: `RichTextRun::new("2").superscript()`, `RichTextRun::new("n").subscript()`
- **XML**: `<vertAlign val="superscript"/>` inside `<rPr>` in rich text runs
- **Impact**: Footnote markers (¹²³), units (m², CO₂), chemical formulas

### 9.4 Foreground Color (Pattern Fill)
- **Gap**: Only background_color, no foreground_color for pattern fills
- **Reference**: `rust_xlsxwriter` → `set_foreground_color()`
- **API**: `Format::new().foreground_color("#FF0000").pattern(Pattern::DarkDown)`
- **XML**: `<patternFill patternType="darkDown"><fgColor rgb="FFFF0000"/><bgColor rgb="FFFFFFFF"/></patternFill>`
- **Impact**: Two-tone pattern fills

### 9.5 Quote Prefix
- **Gap**: No way to force text display of numbers (leading apostrophe)
- **Reference**: `rust_xlsxwriter` → `set_quote_prefix()`
- **API**: `Format::new().quote_prefix()`
- **XML**: `<xf quotePrefix="1"/>` in styles.xml
- **Impact**: Account numbers, zip codes displayed as text

---

## Sprint 10 — Read Enhancements

Estimated: ~100 LOC total.

### 10.1 Open from Buffer
- Covered in 8.4 above (shared with write)

### 10.2 Read Sheet Visibility
- **Gap**: Can't detect hidden or very-hidden sheets
- **Reference**: calamine → `sheets_metadata()` returns `SheetVisible`
- **API**: `ws.is_hidden()`, `ws.is_very_hidden()`, `ws.visibility() -> SheetVisibility`
- **Impl**: Parse `state` attribute from `<sheet>` in workbook.xml during `read_xlsx()`
- **Impact**: Detect config sheets, skip hidden sheets in processing

### 10.3 Read Merge Ranges
- **Gap**: Merge ranges from original file not read back in edit mode
- **API**: `ws.merge_ranges() -> &[(RowNum, ColNum, RowNum, ColNum)]`
- **Impl**: Parse `<mergeCells>` in sheet_reader.rs
- **Impact**: Edit mode preserves merges on dirty sheets

### 10.4 Read Column Widths / Row Heights
- **Gap**: Column widths and row heights from original file not read back
- **API**: `ws.column_width(col) -> Option<f64>`, `ws.row_height(row) -> Option<f64>`
- **Impl**: Parse `<cols>` and `<row ht="">` in sheet_reader.rs
- **Impact**: Edit mode preserves layout on dirty sheets

### 10.5 Read Freeze Panes
- **Gap**: Freeze panes from original file not read back
- **Impl**: Parse `<pane>` in sheet_reader.rs, populate `freeze_row`/`freeze_col`
- **Impact**: Edit mode preserves freeze panes on dirty sheets

### 10.6 Read Hyperlinks
- **Gap**: Hyperlinks from original file not read back
- **Impl**: Parse `<hyperlinks>` in sheet_reader.rs + rels for external URLs
- **Impact**: Edit mode preserves hyperlinks on dirty sheets

### 10.7 Extract Embedded Images
- **Gap**: Can't read images from existing xlsx files
- **Reference**: calamine → `pictures()` returns `Vec<(String, Vec<u8>)>`
- **API**: `wb.pictures() -> Vec<(String, Vec<u8>)>` (filename, bytes)
- **Impl**: Read `xl/media/*` entries from zip
- **Impact**: Image extraction for processing/migration

---

## Sprint 11 — Chart & Image Enhancements

Estimated: ~200 LOC total.

### 11.1 Chart with Pixel Offset
- **Gap**: Charts snap to cell corners, no sub-cell positioning
- **Reference**: `rust_xlsxwriter` → `insert_chart_with_offset(row, col, &chart, x_offset, y_offset)`
- **API**: `ws.insert_chart_with_offset(row, col, &chart, x_px, y_px)?`
- **XML**: `<xdr:colOff>` and `<xdr:rowOff>` in EMU (1 px ≈ 9525 EMU)
- **Impact**: Precise chart placement in dashboards

### 11.2 Image Scale
- **Gap**: Images use original dimensions, no scaling
- **Reference**: `rust_xlsxwriter` → `set_scale_width()`, `set_scale_height()`
- **API**: `Image::from_path("logo.png")?.set_scale_width(0.5).set_scale_height(0.5)`
- **XML**: Adjust `<xdr:to>` coordinates based on scale factor
- **Impact**: Resize logos, screenshots without distortion

### 11.3 Image Fit to Cell
- **Gap**: No auto-sizing image to cell dimensions
- **Reference**: `rust_xlsxwriter` → `insert_image_fit_to_cell()`
- **API**: `ws.insert_image_fit_to_cell(row, col, &image)?`
- **XML**: Use `<xdr:oneCellAnchor>` with `<xdr:ext cx="" cy=""/>` matching cell size
- **Impact**: Product images, logos in header cells

### 11.4 JPEG Support
- **Gap**: Only PNG supported, JPEG magic bytes detected but not fully wired
- **Reference**: `rust_xlsxwriter` → supports PNG, JPEG, GIF, BMP
- **API**: Already `Image::from_path()` — extend to accept JPEG
- **Impl**: JPEG dimension detection (SOF0 marker), content type `image/jpeg`
- **Impact**: Most photos/screenshots are JPEG

### 11.5 Stock Chart
- **Gap**: No stock chart type (OHLC)
- **Reference**: `rust_xlsxwriter` → `Chart::new_stock()`
- **API**: `Chart::new(ChartType::Stock)`
- **XML**: `<c:stockChart>` with High-Low-Close or Open-High-Low-Close series ordering
- **Impact**: Financial data visualization

### 11.6 Chart Data Table
- **Gap**: No data table below chart
- **Reference**: `rust_xlsxwriter` → `chart.set_data_table()`
- **API**: `chart.show_data_table(true)`
- **XML**: `<c:dTable><c:showKeys val="1"/></c:dTable>` in plotArea
- **Impact**: Charts with embedded data for presentations

---

## Sprint 12 — Edit Mode Robustness

Estimated: ~150 LOC total. Fixes the "dirty sheet loses features" problem.

### 12.1 Read and Preserve Drawing References
- **Gap**: When a sheet is dirty, its drawing reference is lost
- **Impl**: Parse `<drawing r:id="..."/>` from sheet XML during lazy deserialization; preserve the relationship and pass through the drawing/chart files
- **Impact**: Editing a cell on a sheet with charts no longer destroys the charts

### 12.2 Read and Preserve Conditional Formatting
- **Gap**: CF rules from original file lost on dirty sheets
- **Impl**: Parse `<conditionalFormatting>` blocks during sheet read; store as raw XML passthrough per sheet
- **Impact**: Edit mode preserves CF on modified sheets

### 12.3 Read and Preserve Data Validation
- **Gap**: DV rules from original file lost on dirty sheets
- **Impl**: Parse `<dataValidations>` during sheet read; store as raw XML passthrough
- **Impact**: Edit mode preserves dropdowns on modified sheets

### 12.4 Read and Preserve Print Settings
- **Gap**: Print settings from original file lost on dirty sheets
- **Impl**: Parse `<pageSetup>`, `<pageMargins>`, `<headerFooter>` during sheet read
- **Impact**: Edit mode preserves print layout

### 12.5 Read and Preserve Comments
- **Gap**: Comments from original file lost on dirty sheets
- **Impl**: Parse comment references during sheet read; pass through comments XML
- **Impact**: Edit mode preserves cell notes

---

## Implementation Order

| Sprint | Scope | Items | Est. LOC | Priority |
|--------|-------|-------|----------|----------|
| **7** | Quick wins | 10 items | ~80 | 🔴 Do first |
| **8** | Write features | 7 items | ~200 | 🔴 Do first |
| **9** | Format additions | 5 items | ~150 | 🟡 High |
| **10** | Read enhancements | 7 items | ~100 | 🟡 High |
| **11** | Chart & image | 6 items | ~200 | 🟡 Medium |
| **12** | Edit mode robustness | 5 items | ~150 | 🟡 Medium |
| **Total** | | **40 items** | **~880** | |

---

## Feature Count After Completion

| Category | Current | After |
|----------|---------|-------|
| Write features | ~50 methods | ~65 methods |
| Read features | ~5 methods | ~15 methods |
| Format options | ~20 builder methods | ~28 builder methods |
| Chart types | 8 | 9 (+ stock) |
| CF types | 11 | 11 (no change) |
| Image formats | PNG only | PNG + JPEG |
| Edit mode fidelity | Cell-only | Cells + layout + drawings |

---

## Acceptance Criteria

Each sprint must:
1. Pass all existing tests (61 lib + 32 demo)
2. Add at least one test per new feature
3. Open cleanly in Microsoft Excel with zero repair errors
4. Be committed with descriptive message
5. Update README.md API reference
