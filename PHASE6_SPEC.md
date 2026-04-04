# Phase 6 Spec: Excel Ribbon Parity

Gap analysis against Excel's ribbon UI. Organized into tiers by impact for financial modeling workflows.

---

## Tier 1 — Quick Wins (single attributes / minor XML)

These are small additions — typically one field on Format or one attribute on an existing XML element.

### 1.1 Wrap Text
- **Ribbon**: Home → Alignment → Wrap Text
- **API**: `Format::new().wrap_text()`
- **XML**: `<alignment wrapText="1"/>` inside `<xf>` in styles.xml
- **Impact**: Every financial model uses multi-line headers

### 1.2 Indent Level
- **Ribbon**: Home → Alignment → Increase/Decrease Indent
- **API**: `Format::new().indent(2)`
- **XML**: `<alignment indent="2"/>` inside `<xf>`
- **Impact**: P&L hierarchies indent sub-line items (COGS, OpEx breakdown)

### 1.3 Shrink to Fit
- **Ribbon**: Home → Alignment → Format Cells → Shrink to Fit
- **API**: `Format::new().shrink_to_fit()`
- **XML**: `<alignment shrinkToFit="1"/>` inside `<xf>`

### 1.4 Text Rotation
- **Ribbon**: Home → Alignment → Orientation
- **API**: `Format::new().rotation(45)` — degrees 0-180 (0=horizontal, 90=up, 255=vertical text)
- **XML**: `<alignment textRotation="45"/>` inside `<xf>`

### 1.5 Strikethrough
- **Ribbon**: Home → Font → Strikethrough
- **API**: `Format::new().strikethrough()`
- **XML**: `<strike/>` inside `<font>` in styles.xml

### 1.6 Zoom Level
- **Ribbon**: View → Zoom
- **API**: `ws.set_zoom(85)` — percentage 10-400
- **XML**: `<sheetView zoomScale="85"/>` — attribute on existing sheetView element

### 1.7 Show/Hide Gridlines
- **Ribbon**: View → Show → Gridlines
- **API**: `ws.hide_gridlines()` / `ws.show_gridlines()`
- **XML**: `<sheetView showGridLines="0"/>` — attribute on existing sheetView element
- **Impact**: Dashboards always hide gridlines for clean look

### 1.8 Show/Hide Row & Column Headings
- **Ribbon**: View → Show → Headings
- **API**: `ws.hide_headings()`
- **XML**: `<sheetView showRowColHeaders="0"/>`

### 1.9 Tab Color
- **Ribbon**: Right-click sheet tab → Tab Color
- **API**: `ws.set_tab_color("#FF0000")`
- **XML**: `<sheetPr><tabColor rgb="FFFF0000"/></sheetPr>` — before `<dimension>` in sheet XML
- **Impact**: Color-coding sheets (blue=inputs, white=calcs, green=outputs)

### 1.10 Right-to-Left Sheet
- **API**: `ws.set_right_to_left()`
- **XML**: `<sheetView rightToLeft="1"/>`

### 1.11 Calculation Mode
- **API**: `wb.set_calc_mode(CalcMode::Manual)` / `CalcMode::Auto` / `CalcMode::AutoNoTable`
- **XML**: `<calcPr calcMode="manual"/>` in workbook.xml after definedNames
- **Impact**: Large models set to manual calc to avoid slowdowns

### 1.12 Repeat Columns (Print)
- **Ribbon**: Page Layout → Print Titles → Columns to repeat
- **API**: `ws.set_repeat_columns(0, 1)` — first_col, last_col
- **XML**: `_xlnm.Print_Titles` defined name, e.g. `'Sheet1'!$A:$B`
- **Note**: Repeat rows already implemented; extend to support columns or both

### 1.13 Scale to Fit / Fit to Page
- **Ribbon**: Page Layout → Scale to Fit
- **API**: `ws.set_print_scale(75)` — percentage; `ws.fit_to_pages(1, 1)` — width_pages, height_pages
- **XML**: `<pageSetup scale="75"/>` or `<pageSetup fitToWidth="1" fitToHeight="1"/>` + `<sheetPr><pageSetUpPr fitToPage="1"/></sheetPr>`
- **Impact**: Printing financial reports to PDF — very common

---

## Tier 2 — Moderate Effort (new CF types, chart features, format additions)

### 2.1 Formula-Based Conditional Formatting
- **Ribbon**: Home → Conditional Formatting → New Rule → Use a formula
- **API**: `ConditionalFormatFormula::new("AND($B2>0,$C2>$D2)")`
- **XML**: `<cfRule type="expression"><formula>AND($B2>0,$C2>$D2)</formula></cfRule>`
- **Impact**: Most powerful CF type — used for alternating row colors, cross-column logic, etc.

### 2.2 Top/Bottom N Conditional Formatting
- **Ribbon**: Home → Conditional Formatting → Top/Bottom Rules
- **API**: `ConditionalFormatTopBottom::new(TopBottomType::Top, 10)` / `TopBottomType::TopPercent`
- **XML**: `<cfRule type="top10" rank="10" percent="0"/>` (percent="1" for percentage)

### 2.3 Text Contains / Begins With / Ends With CF
- **Ribbon**: Home → Conditional Formatting → Highlight Cell Rules → Text that Contains
- **API**: `ConditionalFormatText::new(TextOperator::Contains, "overdue")`
- **XML**: `<cfRule type="containsText" text="overdue"><formula>NOT(ISERROR(SEARCH("overdue",A1)))</formula></cfRule>`

### 2.4 Duplicate / Unique Values CF
- **Ribbon**: Home → Conditional Formatting → Highlight Cell Rules → Duplicate Values
- **API**: `ConditionalFormatDuplicate::new()` / `ConditionalFormatUnique::new()`
- **XML**: `<cfRule type="duplicateValues"/>` / `<cfRule type="uniqueValues"/>`

### 2.5 Above/Below Average CF
- **Ribbon**: Home → Conditional Formatting → Top/Bottom Rules → Above Average
- **API**: `ConditionalFormatAverage::new(AverageType::Above)` / `AverageType::Below`
- **XML**: `<cfRule type="aboveAverage"/>` / `<cfRule type="aboveAverage" aboveAverage="0"/>`

### 2.6 Date Occurring CF
- **Ribbon**: Home → Conditional Formatting → Highlight Cell Rules → A Date Occurring
- **API**: `ConditionalFormatDate::new(DateOccurring::Yesterday)` / `ThisWeek` / `LastMonth` etc.
- **XML**: `<cfRule type="timePeriod" timePeriod="yesterday"/>`

### 2.7 Chart Secondary Axis
- **Ribbon**: Chart → Format Data Series → Secondary Axis
- **API**: `series.set_secondary_axis(true)`
- **XML**: `<c:axId val="..."/>` referencing a second value axis; add `<c:valAx>` with `<c:crossAx>` and `<c:crosses val="max"/>`
- **Impact**: Revenue bars + margin % line on same chart — standard in finance

### 2.8 Chart Data Labels
- **Ribbon**: Chart → Add Chart Element → Data Labels
- **API**: `series.set_data_labels(true)` / `chart.set_data_labels(DataLabelPosition::OutsideEnd)`
- **XML**: `<c:dLbls><c:showVal val="1"/><c:dLblPos val="outEnd"/></c:dLbls>` inside series or chart

### 2.9 Chart Trendlines
- **Ribbon**: Chart → Add Chart Element → Trendline
- **API**: `series.add_trendline(TrendlineType::Linear)` / `Exponential` / `MovingAverage(2)`
- **XML**: `<c:trendline><c:trendlineType val="linear"/></c:trendline>` inside series

### 2.10 Combo Charts (Dual Chart Type)
- **Ribbon**: Insert → Charts → Combo
- **API**: `Chart::new(ChartType::Combo)` with per-series chart type override
- **XML**: Multiple `<c:barChart>` / `<c:lineChart>` plot areas in one `<c:plotArea>`
- **Impact**: Bar + line combos are the most common financial dashboard chart

### 2.11 JPEG / GIF Image Support
- **Ribbon**: Insert → Pictures
- **API**: Already `insert_image()` — extend to detect JPEG/GIF magic bytes
- **XML**: Add content type entries for jpeg/gif; update `[Content_Types].xml` Default extensions
- **Note**: PNG dimension detection exists; add JPEG (SOF0 marker) and GIF (header) parsers

### 2.12 Subscript / Superscript
- **Ribbon**: Home → Font → Format Cells → Effects
- **API**: `RichTextRun { superscript: true, .. }` or on Format
- **XML**: `<vertAlign val="superscript"/>` inside `<rPr>` in rich text; `<vertAlign val="subscript"/>`

### 2.13 Gradient Fill
- **Ribbon**: Home → Fill Color → Gradient
- **API**: `Format::new().gradient_fill(GradientFill { angle: 90.0, stops: vec![...] })`
- **XML**: `<gradientFill degree="90"><stop position="0"><color rgb="FF0000"/></stop><stop position="1"><color rgb="00FF00"/></stop></gradientFill>` inside `<fill>` in styles.xml

### 2.14 Pattern Fill
- **Ribbon**: Home → Fill Color → Pattern (Format Cells → Fill → Pattern Style)
- **API**: `Format::new().pattern_fill(PatternType::DarkDown, fg_color, bg_color)`
- **XML**: `<patternFill patternType="darkDown"><fgColor rgb="..."/><bgColor rgb="..."/></patternFill>`

### 2.15 Diagonal Borders
- **Ribbon**: Home → Borders → More Borders → Diagonal
- **API**: `Format::new().diagonal_border(FormatBorder::Thin, DiagonalType::Up)`
- **XML**: `<border diagonalUp="1"><diagonal style="thin"><color rgb="..."/></diagonal></border>`

---

## Tier 3 — Significant Effort (new subsystems)

### 3.1 Waterfall Chart
- **Ribbon**: Insert → Charts → Waterfall
- **API**: `Chart::new(ChartType::Waterfall)` with `series.set_subtotal_points(&[4, 8])`
- **XML**: Uses `<cx:chart>` namespace (chart extensions) — different from classic `<c:chart>`
- **Impact**: Bridge charts are standard in finance presentations (revenue walk, cost walk)
- **Complexity**: High — requires chartEx part, not classic chart XML

### 3.2 Treemap / Sunburst / Histogram / Box & Whisker / Funnel
- **Ribbon**: Insert → Charts → Statistical / Hierarchy
- **XML**: All use `<cx:chart>` extended chart namespace
- **Complexity**: Same subsystem as waterfall — implement one, get all

### 3.3 Stock Chart
- **Ribbon**: Insert → Charts → Stock
- **API**: `Chart::new(ChartType::Stock)` — requires High-Low-Close or Open-High-Low-Close series
- **XML**: `<c:stockChart>` in classic chart namespace
- **Complexity**: Moderate — classic chart XML but specific series ordering

### 3.4 Shapes / Drawing Objects
- **Ribbon**: Insert → Shapes
- **API**: `ws.insert_shape(row, col, Shape::new(ShapeType::Rectangle).width(200).height(100).text("Label"))`
- **XML**: `<xdr:sp>` elements in drawing XML with `<a:prstGeom prst="rect"/>`
- **Complexity**: Large — many shape types, text inside shapes, connectors

### 3.5 PivotTable
- **Ribbon**: Insert → PivotTable
- **API**: `ws.add_pivot_table(source_range, PivotConfig { rows: [...], cols: [...], values: [...] })`
- **XML**: `pivotTable.xml` + `pivotCacheDefinition.xml` + `pivotCacheRecords.xml` — 3 new part types
- **Complexity**: Very high — most complex Excel feature. Separate cache, field definitions, calculated items
- **Impact**: Extremely high for analysts, but rarely generated programmatically

### 3.6 Threaded Comments
- **Ribbon**: Review → New Comment (modern Excel)
- **API**: `ws.add_threaded_comment("A1", "reply text", "author")`
- **XML**: `threadedComments/threadedComment1.xml` + `commentsExtensible.xml` — uses `<ext>` namespace
- **Complexity**: Moderate — new part type, person list, thread IDs

### 3.7 Slicers
- **Ribbon**: Insert → Slicer (for Tables/PivotTables)
- **API**: `ws.add_slicer(table, "Column1", row, col)`
- **XML**: `slicers/slicer1.xml` + `slicerCache1.xml` — 2 new part types
- **Complexity**: High — tied to tables/pivots, cache mechanism

### 3.8 Named Cell Styles
- **Ribbon**: Home → Cell Styles
- **API**: `wb.add_cell_style("MyStyle", Format::new().bold().font_size(14))`
- **XML**: `<cellStyles>` + `<cellStyleXfs>` in styles.xml — named xf records
- **Complexity**: Moderate — extends existing style system

### 3.9 Split Panes
- **Ribbon**: View → Split
- **API**: `ws.split_panes(row_split_pt, col_split_pt)` — in twips (1/20 of a point)
- **XML**: `<pane xSplit="6000" ySplit="3000" topLeftCell="..." activePane="bottomRight" state="split"/>`

### 3.10 Protect Specific Ranges
- **Ribbon**: Review → Allow Users to Edit Ranges
- **API**: `ws.allow_edit_range("InputCells", "A1:B10", Some("password"))`
- **XML**: `<protectedRanges><protectedRange sqref="A1:B10" name="InputCells"/></protectedRanges>` in sheet XML

### 3.11 Array Formulas
- **API**: `ws.write_array_formula(row, col, last_row, last_col, "MMULT(A1:B2,C1:D2)")`
- **XML**: `<f t="array" ref="E1:F2">MMULT(A1:B2,C1:D2)</f>`
- **Complexity**: Low XML, but need to handle the ref attribute and multi-cell spill

---

## Tier 4 — Low Priority / Niche

### 4.1 SmartArt
- Very complex XML (drawingML diagrams). Rarely generated programmatically.

### 4.2 Text Box
- Subset of Shapes (3.4). `<xdr:sp>` with text body.

### 4.3 WordArt
- Legacy feature. `<a:prstTxWarp>` in drawing XML.

### 4.4 Equations
- OMML (Office Math Markup Language). Niche use case.

### 4.5 Symbols
- Just Unicode characters — no special XML needed. Document in API guide.

### 4.6 Chart Error Bars
- `<c:errBars>` inside series. Niche — mostly scientific/statistical.

### 4.7 3D Maps / Power Map
- Separate binary add-in. Cannot be generated from XML.

### 4.8 Power Query / Get & Transform
- Connection XML + query tables. Very complex, runtime-oriented.

### 4.9 Track Changes
- Legacy feature being replaced by co-authoring. `revisionLog` parts.

---

## Implementation Order (Recommended)

### Sprint 1 — Quick Wins (Tier 1)
All 13 items. Estimated: ~200 lines of code total. Each is 5-20 lines.
1. Wrap text, indent, shrink to fit, text rotation (all alignment attrs)
2. Strikethrough (font attr)
3. Zoom, gridlines, headings, right-to-left (sheetView attrs)
4. Tab color (sheetPr element)
5. Calc mode (workbook.xml calcPr)
6. Repeat columns, scale to fit (print settings)

### Sprint 2 — Conditional Formatting (Tier 2.1–2.6)
6 new CF types. Estimated: ~150 lines. All follow same cfRule pattern.

### Sprint 3 — Chart Enhancements (Tier 2.7–2.10)
Secondary axis, data labels, trendlines, combo charts. Estimated: ~300 lines.

### Sprint 4 — Format & Image (Tier 2.11–2.15)
JPEG/GIF, subscript/superscript, gradient fill, pattern fill, diagonal borders. Estimated: ~200 lines.

### Sprint 5 — Major Features (Tier 3, selected)
Pick based on demand:
- Array formulas (3.11) — low effort, high value
- Protect ranges (3.10) — low effort, high value
- Stock chart (3.3) — moderate effort
- Waterfall + extended charts (3.1–3.2) — high effort, high value for finance
- Shapes (3.4) — high effort, broad utility

### Sprint 6 — Complex Subsystems (Tier 3, remaining)
- PivotTable (3.5) — if demand warrants
- Threaded comments (3.6)
- Slicers (3.7)

---

## Feature Count Summary

| Tier | Items | Effort | Impact |
|---|---|---|---|
| Tier 1 — Quick Wins | 13 | ~200 LOC | High — covers most daily Excel formatting |
| Tier 2 — Moderate | 15 | ~650 LOC | High — CF + charts + fills |
| Tier 3 — Significant | 11 | ~2000+ LOC | Mixed — some critical (waterfall, arrays), some niche |
| Tier 4 — Low Priority | 9 | Large | Low — niche or impossible from XML |
| **Total** | **48** | | |

---

## Demo Update Plan

After each sprint, update the Acme Corp financial model to exercise new features:
- Sprint 1: Wrapped headers, indented P&L rows, hidden gridlines on Dashboard, tab colors, zoom 85% on Dashboard
- Sprint 2: Formula-based CF for alternating rows, above-average highlighting
- Sprint 3: Combo chart (revenue bars + margin line), data labels on charts
- Sprint 4: JPEG logo on cover page, gradient header fills
- Sprint 5: Array formulas for matrix calcs, waterfall chart for revenue bridge
