# Changelog

All notable changes to zavora-xlsx.

## [Unreleased] — Phase 6 In Progress

### Phase 6 Sprint 3 — Chart Enhancements
- **Combo charts**: Per-series `chart_type_override` groups series into separate chart type blocks (e.g., Column + Line in one chart)
- **Secondary axis**: `series.set_secondary_axis(true)` adds second valAx/catAx pair with `crosses="max"` positioning
- **Data labels**: Per-series `set_data_labels(true)` with full show* element set
- **Trendlines**: 6 types — Linear, Exponential, Polynomial, Power, Logarithmic, MovingAverage
- **Excel chart compatibility fixes**:
  - Added Office theme (`theme1.xml`) — required for chart rendering in Excel
  - Added `varyColors`, `gapWidth`, `overlap` to bar/column chart blocks
  - Added `invertIfNegative` to bar/column series
  - Added `marker`, `smooth` to line chart series and chart-type blocks
  - Removed `dLblPos` from per-series data labels (causes Excel to reject entire drawing)
  - Added `showLegendKey`, `showBubbleSize` to data label elements
  - Fixed drawing XML namespace declarations
  - Rewrote chart axes with full OOXML-compliant element set

### Phase 6 Sprint 2 — Conditional Formatting Expansion
- **6 new CF types**: Formula (`type=expression`), Top/Bottom N (`type=top10`), Text Contains/BeginsWith/EndsWith/NotContains (`type=containsText` etc.), Duplicate Values, Unique Values, Above/Below Average (`type=aboveAverage`), Date Occurring (`type=timePeriod`)
- **DXF support**: `DxfData` struct, `register_dxf()`, `<dxfs>` section in styles.xml for differential formatting
- **Bug fix**: `<dxfs>` must come after `<cellStyles>` in styles.xml per OOXML spec

### Phase 6 Sprint 1 — Quick Wins (13 features)
- `Format::text_wrap()`, `shrink_to_fit()`, `strikethrough()`, `indent()`, `rotation()`
- `Worksheet::set_zoom()`, `hide_gridlines()`, `hide_headings()`, `set_right_to_left()`, `set_tab_color()`
- `Workbook::set_calc_mode()` (Auto, Manual, AutoNoTable)
- `Worksheet::set_repeat_columns()`, `set_print_scale()`

## [0.1.0] — Phases 1–5

### Phase 5 — Corruption Fix & Missing Features
- Fixed ZIP corruption in edit mode
- Fixed comments: legacyDrawing ref, workbookProtection ordering
- Added VBA/macro passthrough in edit mode
- Added rich text (`RichText`, `RichTextRun`) — multiple fonts/colors in one cell

### Phase 4 — Advanced & Polish
- **Streaming write**: `StreamingWorkbook` for constant-memory 100K+ row files
- **Autofit**: `ws.autofit()` — auto-size columns from content
- **Sheet protection**: `ws.protect()`, `ws.protect_with_password()`
- **Workbook protection**: `wb.protect()`, `wb.protect_with_password()`
- **Print settings**: `PrintSettings` builder — orientation, paper size, margins, headers/footers, page breaks, repeat rows, print area
- **Comments**: `ws.add_comment()`, `ws.add_comment_with_author()` with VML rendering
- **Row/column grouping**: `ws.group_rows()`, `ws.group_columns()` with outline levels
- **Hidden rows/columns**: `ws.set_row_hidden()`, `ws.set_column_hidden()`
- **Hyperlinks**: `ws.write_url()`, `ws.write_internal_link()`
- **Autofilter**: `ws.set_autofilter()`
- **Cell lock/unlock**: `Format::unlocked()`, `Format::locked()`, `Format::formula_hidden()`

### Phase 3 — Features
- **Charts**: 8 types (Column, Bar, Line, Pie, Scatter, Area, Doughnut, Radar) with series, titles, axes, legends
- **Tables**: Headers, 20+ styles, autofilter, total row
- **Conditional formatting**: Cell value, 2-color scale, 3-color scale, data bars, icon sets
- **Data validation**: Dropdown list, whole number, decimal, date, time, text length, custom formula — with input/error messages
- **Images**: PNG with auto dimension detection
- **Sparklines**: Line, column, win/loss

### Phase 2 — Edit Mode
- `Workbook::open()` — read existing xlsx, modify cells, save
- `Workbook::open_readonly()` — read-only access
- Insert/remove rows and columns with formula reference shifting
- Sheet management: add, remove, rename, reorder
- Passthrough of unknown ZIP entries (preserves macros, etc.)

### Phase 1 — Core
- `Workbook::new()`, `wb.save()`, `wb.save_to_buffer()`
- Cell writing: strings, numbers, booleans, formulas, dates
- `Format` builder: font (bold, italic, underline, size, name, color), background color, number format, borders, alignment
- Row/column batch writes
- Column widths, row heights, freeze panes
- Merge cells
- Multi-sheet workbooks
- Defined names
- Document properties
- Cell reading: `ws.read_cell()`, `ws.used_range()`
- Shared string table (SST) for string deduplication
