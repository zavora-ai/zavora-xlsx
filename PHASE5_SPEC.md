# Phase 5 Spec — Missing Features & Gaps

**Goal**: Close every gap a real Excel power user (financial consultant, data analyst, operations manager) would hit. Fix corruption issues. Make every generated file open cleanly in Excel, Numbers, and LibreOffice.

---

## 1. Corruption Fix (Priority: CRITICAL)

Before adding features, diagnose and fix why Phase 4 files report as corrupt.

**Investigation checklist:**
- [ ] Test each file type individually in Excel, Numbers, LibreOffice
- [ ] Validate XML element ordering against OOXML ISO 29500 Part 1 §18.3.1.99 (worksheet sequence)
- [ ] Verify `<dimension>` element is present (many readers expect it)
- [ ] Verify `<sheetViews>` is always present (even when no freeze panes)
- [ ] Verify `<sheetFormatPr>` default row height element is present
- [ ] Check that `<sheetProtection>` comes after `<sheetFormatPr>` and before `<mergeCells>`
- [ ] Check that `<pageMargins>` comes after `<dataValidations>` and before `<pageSetup>`
- [ ] Validate all files with Open XML SDK validator or `ooxml-lint`

**Required OOXML worksheet element order** (§18.3.1.99):
```
worksheet
  ├── sheetPr?
  ├── dimension?
  ├── sheetViews?
  ├── sheetFormatPr?
  ├── cols*
  ├── sheetData
  ├── sheetCalcPr?
  ├── sheetProtection?
  ├── autoFilter?
  ├── mergeCells?
  ├── conditionalFormatting*
  ├── dataValidations?
  ├── hyperlinks?
  ├── printOptions?
  ├── pageMargins?
  ├── pageSetup?
  ├── headerFooter?
  ├── rowBreaks?
  ├── colBreaks?
  ├── drawing?
  ├── tableParts?
  └── extLst?
```

**Likely fixes:**
- Add `<dimension ref="A1:XX999"/>` element before `<sheetViews>`
- Add `<sheetViews>` with default view even when no freeze panes
- Add `<sheetFormatPr defaultRowHeight="15"/>` before `<cols>`
- Ensure strict element ordering in `write_sheet()`

---

## 2. Hyperlinks (Priority: HIGH)

**Why**: Every financial model has a TOC sheet linking to detail sheets. Analysts link to source documents. Dashboards link to drill-down views.

**OOXML**: `<hyperlinks>` element in worksheet XML + relationship entries.

**Types:**
| Type | Example | XML |
|------|---------|-----|
| URL | `https://example.com` | External relationship + `<hyperlink ref="A1" r:id="rId1"/>` |
| Sheet reference | `Sheet2!A1` | `<hyperlink ref="A1" location="Sheet2!A1"/>` |
| Email | `mailto:user@example.com` | External relationship |
| File | `../reports/Q1.xlsx` | External relationship |

**API:**
```rust
ws.write_url(row, col, "https://example.com")?;
ws.write_url_with_text(row, col, "https://example.com", "Click here")?;
ws.write_url_with_format(row, col, "https://example.com", "Click here", &fmt)?;
ws.write_internal_link(row, col, "Sheet2!A1", "Go to Sheet2")?;
```

**Implementation:**
- Store `Vec<Hyperlink>` on Worksheet
- Write `<hyperlinks>` element in sheet XML (after `dataValidations`, before `printOptions`)
- For external URLs: add relationship in sheet rels, reference by rId
- For internal links: use `location` attribute, no relationship needed
- Default format: blue underline font (auto-applied if no format specified)

**Files touched:** `worksheet.rs`, `sheet_writer.rs`, `workbook.rs` (rels)

---

## 3. Comments / Notes (Priority: HIGH)

**Why**: Audit trail. "Per client email 3/15", "Adjusted per review", "Source: Bloomberg terminal". Every financial model has comments.

**OOXML**: Comments stored in `xl/comments{N}.xml` + VML drawing for positioning in `xl/drawings/vmlDrawing{N}.vml`.

**API:**
```rust
ws.add_comment(row, col, "This value is estimated")?;
ws.add_comment_with_author(row, col, "Needs review", "James K.")?;
```

**Implementation:**
- Store `Vec<Comment>` on Worksheet (row, col, text, author)
- Write `xl/comments{N}.xml` with `<authors>` + `<commentList>` + `<comment>`
- Write VML drawing XML for comment box positioning
- Add content type + relationship entries
- On edit-mode open: passthrough existing comments (parse later if needed)

**Files touched:** `worksheet.rs`, new `writer/comment_writer.rs`, `workbook.rs`

---

## 4. Hidden Rows & Columns (Priority: HIGH)

**Why**: Hide detail rows in summaries. Hide helper columns with intermediate calculations. Hide assumption inputs from end users.

**OOXML**: `hidden="1"` attribute on `<row>` and `<col>` elements.

**API:**
```rust
ws.set_row_hidden(row, true)?;
ws.set_column_hidden(col, true)?;
```

**Implementation:**
- Store `BTreeSet<RowNum>` for hidden rows, `BTreeSet<ColNum>` for hidden cols on Worksheet
- In `write_sheet()`: add `hidden="1"` to `<row>` elements for hidden rows
- In `write_sheet()`: add `hidden="1"` to `<col>` elements for hidden columns
- Ensure hidden cols appear in `<cols>` even if no custom width set

**Files touched:** `worksheet.rs`, `sheet_writer.rs`

---

## 5. Cell Lock / Unlock (Priority: HIGH)

**Why**: In a protected sheet, formula cells should be locked (default) but input cells should be unlocked so users can type values. This is THE most common protection pattern.

**OOXML**: `<protection locked="0"/>` inside `<xf>` in styles.xml. By default all cells are locked; you unlock specific ones.

**API:**
```rust
let input_fmt = Format::new().unlocked(); // cell can be edited when sheet is protected
let locked_fmt = Format::new().locked();  // explicit lock (default behavior)
let hidden_fmt = Format::new().formula_hidden(); // hide formula in formula bar
```

**Implementation:**
- Add `locked: Option<bool>` and `formula_hidden: bool` to `Format`
- Write `<protection locked="0"/>` or `<protection hidden="1"/>` in xf element
- Only meaningful when sheet protection is enabled

**Files touched:** `format.rs`, `style_registry.rs`, `style_writer.rs`

---

## 6. Print Area (Priority: MEDIUM)

**Why**: Financial reports often have a print area defined so only the formatted output prints, not the scratch calculations to the right.

**OOXML**: Defined name `_xlnm.Print_Area` scoped to sheet index.

**API:**
```rust
ws.set_print_area(0, 0, 49, 5)?; // rows 0-49, cols A-F
```

**Implementation:**
- Convert to defined name: `_xlnm.Print_Area` with value `Sheet1!$A$1:$F$50`
- Add to workbook's defined_names with `localSheetId` attribute
- Write in `<definedNames>` section of workbook.xml

**Files touched:** `worksheet.rs`, `workbook.rs`

---

## 7. Repeat Rows / Columns at Print (Priority: MEDIUM)

**Why**: When printing a 100-row report, the header row should appear on every page.

**OOXML**: Defined name `_xlnm.Print_Titles` scoped to sheet index.

**API:**
```rust
ws.set_repeat_rows(0, 1)?;  // repeat rows 0-1 on every printed page
ws.set_repeat_columns(0, 0)?; // repeat column A on every printed page
```

**Implementation:**
- Convert to defined name: `_xlnm.Print_Titles` with value like `Sheet1!$1:$2`
- Add to workbook's defined_names with `localSheetId`

**Files touched:** `worksheet.rs`, `workbook.rs`

---

## 8. Row / Column Grouping & Outline (Priority: MEDIUM)

**Why**: Financial models use collapsible sections — expand Q1 to see monthly detail, collapse to see just the quarterly total. P&L statements group line items under categories.

**OOXML**: `outlineLevel` attribute on `<row>` and `<col>` elements. `<sheetPr><outlinePr summaryBelow="1" summaryRight="1"/></sheetPr>` for outline direction.

**API:**
```rust
ws.set_row_outline_level(row, level)?;  // level 1-7
ws.group_rows(start_row, end_row, level, collapsed)?;
ws.group_columns(start_col, end_col, level, collapsed)?;
```

**Implementation:**
- Store outline level per row/col (extend existing row_heights/col_widths maps or add parallel maps)
- Write `outlineLevel="N"` on `<row>` and `<col>` elements
- Write `collapsed="1"` and `hidden="1"` for collapsed groups
- Write `<sheetPr><outlinePr/>` element

**Files touched:** `worksheet.rs`, `sheet_writer.rs`

---

## 9. Auto-filter (Standalone) (Priority: MEDIUM)

**Why**: Filter transaction data, journal entries, or any list without creating a formal Table. Most analysts use auto-filter more than Tables.

**OOXML**: `<autoFilter ref="A1:F100"/>` element in worksheet XML.

**API:**
```rust
ws.set_autofilter(first_row, first_col, last_row, last_col)?;
```

**Implementation:**
- Store optional `(RowNum, ColNum, RowNum, ColNum)` on Worksheet
- Write `<autoFilter ref="A1:F100"/>` after `<sheetProtection>`, before `<mergeCells>`
- Dropdown arrows appear on the header row in Excel

**Files touched:** `worksheet.rs`, `sheet_writer.rs`

---

## 10. Default Format Elements (Priority: HIGH — affects corruption)

Several XML elements that Excel expects to always be present, even with default values.

**Elements to always write:**
```xml
<dimension ref="A1"/>
<sheetViews>
  <sheetView workbookViewId="0"/>
</sheetViews>
<sheetFormatPr defaultRowHeight="15"/>
```

**Implementation:**
- In `write_sheet()`: always write `<dimension>` computed from cell range
- Always write `<sheetViews>` (currently only written when freeze panes set)
- Always write `<sheetFormatPr>` with default row height

**Files touched:** `sheet_writer.rs`

---

## 11. Demo Overhaul — Financial Model Validation (Priority: HIGH)

Replace the current scattered demo with a realistic financial model that exercises every feature in context.

**Demo file: "Acme Corp Q1 Financial Report"**

Sheet 1 — **Cover Page**:
- Company logo (image)
- Title in rich text (bold company name, regular subtitle)
- Hyperlinks to each section sheet
- Print settings (landscape, no margins)

Sheet 2 — **P&L Statement**:
- Headers with merge cells, bold, borders, background color
- Revenue/expense line items with number format `$#,##0`
- Percentage column with format `0.0%`
- SUM formulas for subtotals
- Cross-sheet formula referencing assumptions
- Row grouping: expand months under quarters
- Conditional formatting: red for negative values
- Freeze panes on header row
- Print area set, repeat rows at top

Sheet 3 — **Balance Sheet**:
- Similar structure to P&L
- Date column with date format
- Named ranges for key totals
- Comments on estimated values
- Hidden helper column with intermediate calcs

Sheet 4 — **Dashboard**:
- Chart (revenue trend line)
- Table with autofilter
- Sparklines for monthly trends
- Data validation dropdown for period selector
- Conditional formatting color scale on KPIs

Sheet 5 — **Assumptions**:
- Input cells (unlocked format) on protected sheet
- Data validation on input ranges
- Comments explaining each assumption
- Named ranges for each input

Sheet 6 — **Data**:
- 10K rows of transaction data
- Auto-filter enabled
- Hidden detail columns
- Table with totals row

**Validation criteria:**
- File opens in Excel with zero repair prompts
- File opens in Numbers without corruption warning
- All formulas calculate correctly
- All formatting renders correctly
- Protection works (can edit input cells, can't edit formula cells)
- Print preview shows correct layout
- Charts render with correct data
- Hyperlinks navigate correctly

---

## 12. Implementation Order

| # | Feature | Effort | Blocks |
|---|---------|--------|--------|
| 1 | **Corruption fix** (dimension, sheetViews, sheetFormatPr, element ordering) | Small | Everything |
| 2 | **Hidden rows/columns** | Tiny | Demo |
| 3 | **Cell lock/unlock** | Tiny | Demo |
| 4 | **Hyperlinks** | Small | Demo |
| 5 | **Auto-filter** | Tiny | Demo |
| 6 | **Print area + repeat rows** | Small | Demo |
| 7 | **Comments/Notes** | Medium | Demo |
| 8 | **Row/column grouping** | Medium | Demo |
| 9 | **Demo overhaul** | Medium | — |
| 10 | **Validate in Excel/Numbers/LibreOffice** | — | — |

Total estimated effort: ~1 session for features, ~1 session for demo + validation.

---

## 13. Non-Goals (Out of Scope)

These are Excel features we deliberately do not implement:
- **Pivot tables** — extremely complex XML, rarely written programmatically
- **Macros/VBA editing** — we passthrough VBA binary, don't parse/modify it
- **Calculation engine** — we write formulas, Excel evaluates them
- **Conditional formatting with formulas** — complex; cell value rules cover 90% of use cases
- **Slicers / Timelines** — dashboard widgets tied to pivot tables
- **Power Query / Data Model** — separate subsystem
- **Threaded comments** — modern Excel feature, legacy comments cover the need
- **Dynamic arrays / LAMBDA** — formula engine features, not file format
