# Phase 6 Spec: Excel Ribbon Parity

Gap analysis against Excel's ribbon UI. Organized into tiers by impact for financial modeling workflows.

## Status

| Sprint | Scope | Status |
|--------|-------|--------|
| Sprint 1 | 13 Tier 1 Quick Wins | ✅ Complete |
| Sprint 2 | 6 New CF Types + DXF | ✅ Complete |
| Sprint 3 | Chart Enhancements | ✅ Complete (Excel-compatible) |
| Sprint 4 | Format & Image additions | 🔲 Not started |
| Sprint 5 | Major features (Tier 3) | 🔲 Not started |

---

## Sprint 1 — Quick Wins ✅

All 13 items implemented and tested.

| # | Feature | API | Status |
|---|---------|-----|--------|
| 1.1 | Wrap Text | `Format::new().text_wrap()` | ✅ |
| 1.2 | Indent Level | `Format::new().indent(2)` | ✅ |
| 1.3 | Shrink to Fit | `Format::new().shrink_to_fit()` | ✅ |
| 1.4 | Text Rotation | `Format::new().rotation(45)` | ✅ |
| 1.5 | Strikethrough | `Format::new().strikethrough()` | ✅ |
| 1.6 | Zoom Level | `ws.set_zoom(85)` | ✅ |
| 1.7 | Hide Gridlines | `ws.hide_gridlines()` | ✅ |
| 1.8 | Hide Headings | `ws.hide_headings()` | ✅ |
| 1.9 | Tab Color | `ws.set_tab_color("#FF0000")` | ✅ |
| 1.10 | Right-to-Left | `ws.set_right_to_left()` | ✅ |
| 1.11 | Calc Mode | `wb.set_calc_mode(CalcMode::Manual)` | ✅ |
| 1.12 | Repeat Columns | `ws.set_repeat_columns(0, 1)` | ✅ |
| 1.13 | Print Scale / Fit to Page | `ws.set_print_scale(75)` | ✅ |

---

## Sprint 2 — Conditional Formatting ✅

6 new CF types + differential formatting (DXF) support.

| # | Feature | API | Status |
|---|---------|-----|--------|
| 2.1 | Formula CF | `ConditionalFormatFormula::new("$B2>100")` | ✅ |
| 2.2 | Top/Bottom N | `ConditionalFormatTopBottom::new(TopBottomType::Top, 10)` | ✅ |
| 2.3 | Text Contains/Begins/Ends | `ConditionalFormatText::new(TextOperator::Contains, "text")` | ✅ |
| 2.4 | Duplicate Values | `ConditionalFormatDuplicate::new()` | ✅ |
| 2.5 | Unique Values | `ConditionalFormatUnique::new()` | ✅ |
| 2.6 | Above/Below Average | `ConditionalFormatAverage::new(AverageType::Above)` | ✅ |
| 2.7 | Date Occurring | `ConditionalFormatDate::new(DateOccurring::ThisWeek)` | ✅ |
| — | DXF Support | `DxfData` struct, `register_dxf()`, `<dxfs>` in styles.xml | ✅ |

---

## Sprint 3 — Chart Enhancements ✅

All features working in Excel (validated with zero repair errors).

| # | Feature | API | Status |
|---|---------|-----|--------|
| 2.7 | Secondary Axis | `series.set_secondary_axis(true)` | ✅ |
| 2.8 | Data Labels | `series.set_data_labels(true)` | ✅ |
| 2.9 | Trendlines (6 types) | `series.set_trendline(TrendlineType::Linear)` | ✅ |
| 2.10 | Combo Charts | `series.set_chart_type(ChartType::Line)` | ✅ |

**Bug fixes applied for Excel compatibility:**
- Office theme (`theme1.xml`) — required for chart rendering
- Chart-type elements: `varyColors`, `gapWidth`, `overlap`, `marker`, `smooth`
- Series elements: `invertIfNegative` (bar/col), `marker`/`smooth` (line)
- Data labels: removed `dLblPos` (causes Excel to reject drawing), added `showLegendKey`/`showBubbleSize`
- Drawing XML: correct namespace declarations
- Axes: full element set matching OOXML spec

---

## Sprint 4 — Format & Image (Not Started)

| # | Feature | API | Effort |
|---|---------|-----|--------|
| 2.11 | JPEG/GIF Images | Extend `Image` magic byte detection | Low |
| 2.12 | Subscript/Superscript | `RichTextRun { superscript: true }` | Low |
| 2.13 | Gradient Fill | `Format::new().gradient_fill(...)` | Medium |
| 2.14 | Pattern Fill | `Format::new().pattern_fill(...)` | Low |
| 2.15 | Diagonal Borders | `Format::new().diagonal_border(...)` | Low |

---

## Sprint 5 — Major Features (Not Started)

Selected Tier 3 items by value/effort ratio:

| # | Feature | Effort | Impact |
|---|---------|--------|--------|
| 3.11 | Array Formulas | Low | High |
| 3.10 | Protect Specific Ranges | Low | High |
| 3.3 | Stock Chart | Medium | Medium |
| 3.1 | Waterfall Chart | High | High (finance) |
| 3.4 | Shapes | High | Broad |
| 3.5 | PivotTable | Very High | Very High |

---

## Tier 4 — Low Priority / Niche

SmartArt, Text Box, WordArt, Equations, Chart Error Bars, 3D Maps, Power Query, Track Changes. Not planned unless demand warrants.
