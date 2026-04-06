# Changelog

All notable changes to zavora-xlsx.

## [Unreleased]

### Sprints 7–12: Feature Parity (40 items)

#### Sprint 7 — Quick Wins
- `wb.set_active_sheet(index)` — controls which sheet opens first
- `ws.write_blank(row, col, &format)` — formatted empty cells
- `ws.clear_cell(row, col)` — remove cell values
- `ws.set_default_row_height(height)` — consistent row sizing
- `ws.set_column_format(col, &format)` — format entire column
- `ws.set_row_format(row, &format)` — format entire row
- `ws.set_hidden()` / `ws.set_very_hidden()` — sheet visibility
- `ws.set_selection(row, col)` — cursor position on open
- `ws.set_top_left_cell(row, col)` — scroll position on open
- `ws.ignore_error(type, range)` — suppress green triangles
- `ws.filter_column(col, values)` — autofilter criteria

#### Sprint 8 — Write Features
- `ws.write_array_formula(r1, c1, r2, c2, formula)` — legacy CSE array formulas
- `ws.write_dynamic_formula(row, col, formula)` — Excel 365 spill formulas
- `ws.write_formula_with_result(row, col, formula, result)` — cached value
- `Workbook::open_readonly_from_buffer(bytes)` / `open_from_buffer(bytes)` — in-memory
- `PrintSettings`: `print_gridlines`, `print_headings`, `center_horizontally`, `center_vertically`, `black_and_white`, `first_page_number`
- `ws.unprotect_range(name, range)` — editable ranges on protected sheets

#### Sprint 9 — Format Additions
- `Format::diagonal_border(style, DiagonalType)` — up/down/both
- `Pattern` enum expanded to 18 types (Solid, MediumGray, DarkGray, etc.)
- `Format::foreground_color()` — two-tone pattern fills
- `RichTextRun::superscript()` / `subscript()` — font script
- `Format::quote_prefix()` — force text display

#### Sprint 10 — Read Enhancements
- Read sheet visibility (hidden/veryHidden) from workbook.xml
- Read merge ranges, column widths, row heights, freeze panes from sheet XML
- `ws.visibility()`, `ws.is_hidden()`, `ws.is_very_hidden()`
- `ws.merge_ranges()`, `ws.column_width(col)`, `ws.row_height(row)`
- `Workbook::pictures()` — extract embedded images

#### Sprint 11 — Chart & Image
- `ws.insert_chart_with_offset(row, col, chart, x_px, y_px)` — pixel positioning
- `Image::set_scale_width()` / `set_scale_height()` — resize images
- `ChartType::Stock` — stock/HLC charts
- `chart.show_data_table(true)` — data table below chart
- JPEG support with dimension detection

#### Sprint 12 — Edit Mode Robustness
- Preserve drawing/chart references when cells modified on sheets with charts
- Preserve comment references on dirty sheets
- Read back merges, widths, heights, freeze during lazy deserialization
- Pass through original sheet rels for dirty sheets
- Content types include passthrough drawings/charts/comments

### Audit Fixes
- **Security**: Zip bomb protection (200MB limit), sheet name validation, string-to-formula injection fix
- **Bugs**: docProps duplicate in edit mode, dead DataLabelPosition removed, blank cells with formatting now written
- **Performance**: `read_cell()` O(log n) via BTreeMap index
- **Completeness**: Streaming mode includes theme

### Phase 6 Sprints 1–3
- Sprint 1: 13 formatting/view/print quick wins
- Sprint 2: 6 new CF types + DXF support
- Sprint 3: Chart enhancements (combo, secondary axis, data labels, trendlines)
- Excel chart compatibility: theme, varyColors, gapWidth, marker, smooth, dLblPos fix

## [0.1.0] — Phases 1–5

### Phase 5 — Corruption Fix & Missing Features
- Fixed ZIP corruption in edit mode
- VBA/macro passthrough, rich text

### Phase 4 — Advanced & Polish
- Streaming write, autofit, sheet/workbook protection, print settings, comments, row/column grouping, hidden rows/cols, hyperlinks, autofilter, cell lock/unlock

### Phase 3 — Features
- Charts (8 types), tables, conditional formatting (5 types), data validation (7 types), images (PNG), sparklines

### Phase 2 — Edit Mode
- Open/modify/save, insert/remove rows & columns, sheet management

### Phase 1 — Core
- Create/save workbooks, cell writing (string/number/bool/formula/date), formatting, merge, freeze panes, defined names, document properties, cell reading
