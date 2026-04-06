# zavora-xlsx Feature Parity Spec

Gap analysis against rust_xlsxwriter, umya-spreadsheet, and calamine.
Prioritized for financial consulting workflows (Excel compatibility paramount).

## Current State

- **~7,400 lines** of Rust, 4 dependencies
- **61 lib tests + 44 demo tests**, all passing
- Read / Write / Edit with VBA passthrough
- First Rust crate with pivot table + pivot chart creation

---

## Sprint 13: DateTime + Page Setup + Header/Footer

**Goal:** Financial reports need dates, printing, and page branding.

### 13a. DateTime Type (~40 LOC)

Accountants use dates in every model. Need a proper `write_datetime` that converts
calendar dates to Excel serial numbers (days since 1900-01-01, with the Lotus 1-2-3
Feb 29 1900 bug for compatibility).

```rust
// API
ws.write_datetime(row, col, 2024, 6, 15)?;                    // date only
ws.write_datetime_hms(row, col, 2024, 6, 15, 14, 30, 0)?;    // date + time
ws.write_time(row, col, 14, 30, 0)?;                          // time only

// With format
let fmt = Format::new().set_num_format("yyyy-mm-dd");
ws.write_datetime_with_format(row, col, 2024, 6, 15, &fmt)?;
```

**Implementation:**
- `fn date_to_serial(year, month, day) -> f64` in `utility.rs`
- `fn time_to_fraction(hour, min, sec) -> f64` in `utility.rs`
- Write as `CellType::Number(serial)` — Excel interprets via number format
- Handle the 1900 leap year bug (serial 60 = Feb 29 1900, which doesn't exist)
- Validate ranges: year 1900–9999, month 1–12, day 1–31, hour 0–23, min/sec 0–59

### 13b. Page Setup (~50 LOC)

Financial reports are printed constantly. Need orientation, margins, paper size,
and fit-to-page scaling.

```rust
// API
ws.set_landscape();                              // default is portrait
ws.set_paper_size(PaperSize::Letter);            // Letter, A4, Legal, etc.
ws.set_margins(0.75, 0.75, 1.0, 1.0);           // left, right, top, bottom (inches)
ws.set_fit_to_pages(1, 0);                       // width_pages, height_pages (0 = auto)
ws.set_print_scale(75);                          // percentage 10–400
ws.set_print_area("A1:H50")?;                   // range to print
ws.set_repeat_rows(0, 2);                        // repeat rows 1–3 on every page
ws.set_repeat_columns(0, 1);                     // repeat columns A–B on every page
```

**Implementation:**
- Add fields to Worksheet: `landscape`, `paper_size`, `margins`, `fit_to_page`, `print_scale`, `print_area`, `repeat_rows`, `repeat_cols`
- `PaperSize` enum: Letter(1), A4(9), Legal(5), A3(8), Tabloid(3), Executive(7), B4(12), B5(13)
- Write `<pageSetup>` attributes: `orientation`, `paperSize`, `scale`, `fitToWidth`, `fitToHeight`
- Write `<pageMargins>` with left/right/top/bottom/header/footer
- Print area and repeat rows/cols go into defined names in workbook.xml:
  - `_xlnm.Print_Area` for print area
  - `_xlnm.Print_Titles` for repeat rows/cols

### 13c. Header/Footer (~30 LOC)

Company name, page numbers, dates, "CONFIDENTIAL" stamps on printed pages.

```rust
// API
ws.set_header("&C&\"Arial,Bold\"&14Monthly Report");
ws.set_footer("&L&D&RPage &P of &N");

// Convenience methods
ws.set_header_center("Monthly Report");
ws.set_header_left("CONFIDENTIAL");
ws.set_footer_center("Page &P of &N");
ws.set_footer_right("&D");                       // current date
```

**Excel header/footer codes:**
- `&L` left, `&C` center, `&R` right section
- `&P` page number, `&N` total pages
- `&D` date, `&T` time, `&F` filename, `&A` sheet name
- `&"Font,Style"` font, `&nn` font size, `&B` bold, `&I` italic

**Implementation:**
- Add `header: Option<String>`, `footer: Option<String>` to Worksheet
- Write `<headerFooter><oddHeader>` and `<oddFooter>` in sheet XML
- Convenience methods build the `&L`/`&C`/`&R` format string

### Sprint 13 Estimate: ~120 LOC

---

## Sprint 14: Row/Column Grouping (Outline)

**Goal:** Financial statements use expandable/collapsible detail sections.

```rust
// API — group rows
ws.group_rows(5, 20, 1)?;                        // rows 5–20, outline level 1
ws.group_rows(8, 15, 2)?;                        // nested: rows 8–15, level 2
ws.set_row_collapsed(5, 20)?;                     // collapse the group

// API — group columns
ws.group_columns(2, 5, 1)?;                      // columns C–F, level 1
ws.set_column_collapsed(2, 5)?;                   // collapse

// Control outline direction
ws.set_outline_settings(true, true);              // summary_below, summary_right
```

**Implementation (~80 LOC):**
- Add `row_outlines: Vec<(u32, u32, u8, bool)>` (start, end, level, collapsed) to Worksheet
- Add `col_outlines: Vec<(u16, u16, u8, bool)>` to Worksheet
- Write `outlineLevelRow` / `outlineLevelCol` attributes on `<row>` and `<col>` elements
- Write `<sheetPr><outlinePr summaryBelow="1" summaryRight="1"/>` when outlines exist
- Collapsed groups: set `hidden="1"` on grouped rows/cols, `collapsed="1"` on summary row/col

### Sprint 14 Estimate: ~80 LOC

---

## Sprint 15: Read Enhancements

**Goal:** Round-trip fidelity — read back everything we write.

### 15a. Named Ranges Reading (~20 LOC)

```rust
// API
let names = wb.defined_names();                   // Vec<(String, String)> — (name, formula)
let val = wb.defined_name("TaxRate");             // Option<&str>
```

**Implementation:**
- Parse `<definedName>` elements from workbook.xml during read
- Store in `Workbook.defined_names: Vec<(String, String)>`

### 15b. Comment Reading (~25 LOC)

```rust
// API
let comments = ws.comments();                     // &[(u32, u16, String, String)] — (row, col, author, text)
let comment = ws.get_comment(row, col);           // Option<(&str, &str)> — (author, text)
```

**Implementation:**
- Parse `xl/comments{N}.xml` during sheet read
- Match `<comment ref="A1" authorId="0">` to cells
- Store in Worksheet read data

### 15c. Conditional Formatting Reading (~40 LOC)

```rust
// API
let cfs = ws.conditional_formats();               // read-only access to CF rules
```

**Implementation:**
- Parse `<conditionalFormatting>` elements during sheet read
- Store as lightweight structs (range, type, operator, values, priority)

### Sprint 15 Estimate: ~85 LOC

---

## Sprint 16: Auto-filter + Workbook Protection + CSV Export

### 16a. Full Auto-filter API (~25 LOC)

```rust
// API
ws.set_autofilter(0, 0, 100, 5)?;                // first_row, first_col, last_row, last_col
ws.autofilter_column(0, FilterCriteria::Equal("East"))?;
ws.autofilter_column(3, FilterCriteria::GreaterThan(10000.0))?;
ws.autofilter_column(1, FilterCriteria::List(vec!["Widget", "Gadget"]))?;
```

**Implementation:**
- We already have filter column criteria — extend with `set_autofilter` range
- Write `<autoFilter ref="A1:F101">` with `<filterColumn>` children

### 16b. Workbook Protection (~15 LOC)

```rust
// API
wb.protect("password");                           // prevent structural changes
wb.protect_with_options("password", WorkbookProtection {
    lock_structure: true,                         // prevent add/delete/rename sheets
    lock_windows: false,                          // prevent window resize
});
```

**Implementation:**
- Add `<workbookProtection>` to workbook.xml with hashed password
- Use the legacy Excel password hash (not strong crypto — matches Excel behavior)

### 16c. CSV Export (~30 LOC)

```rust
// API
ws.to_csv(writer, b',')?;                        // write sheet as CSV to any Write impl
ws.to_csv_with_options(writer, CsvOptions {
    delimiter: b',',
    quote: b'"',
    line_ending: "\r\n",
})?;
```

**Implementation:**
- Iterate cells in row/col order
- Quote strings containing delimiter/newline/quote
- Handle formula cells (use cached result if available)

### Sprint 16 Estimate: ~70 LOC

---

## Summary

| Sprint | Features | LOC | Priority |
|--------|----------|-----|----------|
| 13 | DateTime, page setup, header/footer | ~120 | 🔴 High |
| 14 | Row/column grouping (outline) | ~80 | 🔴 High |
| 15 | Read: named ranges, comments, CF | ~85 | 🟡 Medium |
| 16 | Auto-filter, workbook protection, CSV | ~70 | ⚪ Low |
| **Total** | | **~355 LOC** | |

### Execution Order

1. Sprint 13 — accountants can't work without dates and printing
2. Sprint 14 — financial statements need expandable sections
3. Sprint 15 — read-back completeness for edit workflows
4. Sprint 16 — convenience features, can add incrementally

After these 4 sprints, zavora-xlsx will have **100% feature parity** with all three
reference crates combined, plus pivot tables and pivot charts that none of them have.

**Total crate size after completion: ~7,750 lines.**
