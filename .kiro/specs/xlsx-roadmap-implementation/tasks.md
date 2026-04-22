# Implementation Tasks — zavora-xlsx Roadmap

## Phase 1: Read Parity

- [x] 1. Read Cell Formatting
  - [x] 1.1 Extend `ParsedStyles` with `fonts`, `fills`, `borders`, `xf_records`, and `dxf_records` vectors
  - [x] 1.2 Implement full font/fill/border/alignment parsing in `style_parser.rs`
  - [x] 1.3 Add `resolve_format(xf_index) -> Format` method to `ParsedStyles`
  - [x] 1.4 Expose `cell_format(row, col) -> Option<Format>` on `Worksheet`
  - [x] 1.5 Write tests: read a file with bold, colored, bordered cells and verify Format fields
  - [x] 1.6 Write example (`examples/read_cell_format.rs`): create formatted cells, read back, print resolved formats

- [x] 2. Read Conditional Formatting
  - [x] 2.1 Create `src/reader/cf_reader.rs` module
  - [x] 2.2 Parse `<conditionalFormatting>` elements in sheet XML into `StoredCf` structs
  - [x] 2.3 Resolve DXF indices to font/fill/border components
  - [x] 2.4 Support all 11 CF rule types (cell value, 2/3-color scale, data bar, icon set, formula, top/bottom, text, duplicate, unique, average, date)
  - [x] 2.5 Expose `conditional_formats()` accessor on `Worksheet`
  - [x] 2.6 Write round-trip test: write CF rules, read back, verify equality
  - [x] 2.7 Write example (`examples/read_conditional_formats.rs`): create CF rules, read back, print parsed rules

- [x] 3. Read Data Validation
  - [x] 3.1 Create `src/reader/validation_reader.rs` module
  - [x] 3.2 Parse `<dataValidations>` elements into `DataValidation` structs
  - [x] 3.3 Support all 7 rule types (list, whole number, decimal, date, time, text length, custom)
  - [x] 3.4 Preserve input/error message title and body
  - [x] 3.5 Expose `validations()` accessor on `Worksheet`
  - [x] 3.6 Write round-trip test: write validations, read back, verify equality
  - [x] 3.7 Write example (`examples/read_data_validation.rs`): create validations, read back, print parsed rules

- [x] 4. Read Charts
  - [x] 4.1 Create `src/reader/chart_reader.rs` module
  - [x] 4.2 Parse drawing relationships to discover chart part paths
  - [x] 4.3 Parse `c:chart` namespace XML into `Chart` structs (all 9 types)
  - [x] 4.4 Parse series data (values, categories, names)
  - [x] 4.5 Parse chart title, legend position, axis names
  - [x] 4.6 Parse `cx:chart` namespace (ChartEx/Treemap) into `TreemapChart` structs
  - [x] 4.7 Expose `charts()` accessor on `Worksheet`
  - [x] 4.8 Write round-trip test: write charts, read back, verify type and series count
  - [x] 4.9 Write example (`examples/read_charts.rs`): create charts, read back, print chart properties

- [x] 5. Read Tables
  - [x] 5.1 Create `src/reader/table_reader.rs` module
  - [x] 5.2 Parse table part XML into `Table` structs
  - [x] 5.3 Parse column definitions (header names, total functions)
  - [x] 5.4 Resolve `TableStyle` enum from style name string
  - [x] 5.5 Parse autofilter range and column criteria
  - [x] 5.6 Expose `tables()` accessor on `Worksheet`
  - [x] 5.7 Write round-trip test: write table, read back, verify columns and style
  - [x] 5.8 Write example (`examples/read_tables.rs`): create tables, read back, print table properties

- [x] 6. Read Hyperlinks
  - [x] 6.1 Parse `<hyperlinks>` element and sheet relationship file for external URLs
  - [x] 6.2 Parse internal hyperlinks (location attribute)
  - [x] 6.3 Populate `Hyperlink` structs with URL, location, and display text
  - [x] 6.4 Expose `hyperlinks()` accessor on `Worksheet`
  - [x] 6.5 Write round-trip test: write hyperlinks, read back, verify URLs
  - [x] 6.6 Write example (`examples/read_hyperlinks.rs`): create hyperlinks, read back, print link targets

- [x] 7. Read Comments
  - [x] 7.1 Create `src/reader/comment_reader.rs` module
  - [x] 7.2 Parse legacy comment XML (authors + comment list)
  - [x] 7.3 Map comments to cell coordinates
  - [x] 7.4 Handle missing/malformed comment files gracefully (skip without error)
  - [x] 7.5 Write round-trip test: write comments, read back, verify text and author
  - [x] 7.6 Write example (`examples/read_comments.rs`): create comments, read back, print comment data

- [-] 8. Read Sparklines
  - [ ] 8.1 Create `src/reader/sparkline_reader.rs` module
  - [x] 8.2 Parse sparkline groups from `<extLst>` in sheet XML
  - [x] 8.3 Populate `Sparkline` structs with data range, location, and type
  - [x] 8.4 Expose `sparklines()` accessor on `Worksheet`
  - [x] 8.5 Write round-trip test: write sparklines, read back, verify type and range
  - [x] 8.6 Write example (`examples/read_sparklines.rs`): create sparklines, read back, print sparkline properties

- [x] 9. Read Defined Names with Scope
  - [x] 9.1 Parse `localSheetId` attribute on `<definedName>` elements
  - [x] 9.2 Create `DefinedName` struct with name, formula, and scope (workbook or sheet index)
  - [x] 9.3 Expose `defined_names()` on `Workbook` returning scope information
  - [x] 9.4 Write test: create workbook with sheet-scoped and global names, read back, verify scope
  - [x] 9.5 Write example (`examples/read_defined_names.rs`): create named ranges, read back, print scope info

- [x] 10. Read Print Settings
  - [x] 10.1 Parse `<pageMargins>`, `<pageSetup>`, `<headerFooter>` from sheet XML
  - [x] 10.2 Parse `<rowBreaks>` and `<colBreaks>` for page breaks
  - [x] 10.3 Populate `PrintSettings` struct from parsed elements
  - [x] 10.4 Parse repeat rows/columns from defined names (`_xlnm.Print_Titles`)
  - [x] 10.5 Expose `print_settings()` accessor on `Worksheet`
  - [x] 10.6 Write round-trip test: set print settings, read back, verify margins and orientation
  - [x] 10.7 Write example (`examples/read_print_settings.rs`): set print settings, read back, print parsed values

- [x] 11. Read Sheet Protection
  - [x] 11.1 Parse `<sheetProtection>` attributes from sheet XML
  - [x] 11.2 Populate `SheetProtection` struct with flags and password hash
  - [x] 11.3 Expose `protection()` accessor on `Worksheet`
  - [x] 11.4 Write round-trip test: protect sheet, read back, verify protection state
  - [x] 11.5 Write example (`examples/read_sheet_protection.rs`): protect sheet, read back, print protection state

- [x] 12. Read Row and Column Grouping
  - [x] 12.1 Parse `outlineLevel` attribute from `<row>` elements
  - [x] 12.2 Parse `outlineLevel` attribute from `<col>` elements
  - [x] 12.3 Expose `row_outline_levels()` and `col_outline_levels()` on `Worksheet`
  - [x] 12.4 Write round-trip test: group rows/cols, read back, verify levels
  - [x] 12.5 Write example (`examples/read_grouping.rs`): group rows/cols, read back, print outline levels


## Phase 2: Formula Engine

- [x] 13. Formula Tokenizer
  - [x] 13.1 Create `src/formula_engine/mod.rs` and `src/formula_engine/token.rs`
  - [x] 13.2 Define `Token` enum (CellRef, RangeRef, SheetRef, StructuredRef, Number, StringLiteral, Bool, Error, Function, Operator, Paren, Comma, Colon, Array)
  - [x] 13.3 Implement `tokenize(formula: &str) -> Result<Vec<Token>>` supporting A1-style refs
  - [x] 13.4 Add R1C1-style reference tokenization
  - [x] 13.5 Add structured table reference tokenization (`Table[Column]`)
  - [x] 13.6 Write tests: tokenize complex formulas, verify token sequences
  - [x] 13.7 Write example (`examples/formula_tokenizer.rs`): tokenize sample formulas, print token sequences

- [x] 14. Formula Parser (AST)
  - [x] 14.1 Create `src/formula_engine/ast.rs` with `AstNode` enum
  - [x] 14.2 Create `src/formula_engine/parser.rs` with Pratt parser
  - [x] 14.3 Handle operator precedence (unary minus, ^, *, /, +, -, &, =, <>, <, >, <=, >=)
  - [x] 14.4 Handle function calls with variable argument counts
  - [x] 14.5 Handle array literals `{1,2;3,4}`
  - [x] 14.6 Create `src/formula_engine/printer.rs` for AST → formula string
  - [x] 14.7 Write round-trip property test: parse(print(parse(s))) == parse(s)
  - [x] 14.8 Write example (`examples/formula_parser.rs`): parse formulas into AST, print tree structure

- [x] 15. Formula Dependency Graph
  - [x] 15.1 Create `src/formula_engine/dependency.rs`
  - [x] 15.2 Implement `DependencyGraph::build(workbook)` — extract cell refs from ASTs
  - [x] 15.3 Implement topological sort (Kahn's algorithm)
  - [x] 15.4 Implement cycle detection (Tarjan's algorithm)
  - [x] 15.5 Implement `dependents_of(cell)` for incremental recalc
  - [x] 15.6 Write tests: linear chain, diamond dependency, cycle detection

- [x] 16. Formula Evaluator Core
  - [x] 16.1 Create `src/formula_engine/evaluator.rs`
  - [x] 16.2 Implement arithmetic operators (+, -, *, /, ^)
  - [x] 16.3 Implement comparison operators (=, <>, <, >, <=, >=)
  - [x] 16.4 Implement string concatenation (&)
  - [x] 16.5 Implement cell reference resolution from workbook context
  - [x] 16.6 Implement range expansion for function arguments
  - [x] 16.7 Write tests: evaluate simple arithmetic and comparison formulas

- [x] 17. Core Functions (SUM, IF, VLOOKUP, INDEX, MATCH)
  - [x] 17.1 Create `src/formula_engine/functions/mod.rs` with function registry
  - [x] 17.2 Implement `src/formula_engine/functions/core.rs`: SUM, AVERAGE, COUNT, COUNTA, MIN, MAX
  - [x] 17.3 Implement IF with 2 and 3 arguments
  - [x] 17.4 Implement VLOOKUP (exact and approximate match)
  - [x] 17.5 Implement INDEX and MATCH
  - [x] 17.6 Write tests: verify each function with edge cases
  - [x] 17.7 Write example (`examples/formula_evaluation.rs`): evaluate formulas in a workbook, print results

- [x] 18. Math Functions
  - [x] 18.1 Create `src/formula_engine/functions/math.rs`
  - [x] 18.2 Implement ROUND, ROUNDUP, ROUNDDOWN
  - [x] 18.3 Implement ABS, MOD, POWER, SQRT, LOG, LN, EXP, PI
  - [x] 18.4 Implement #NUM! error for invalid arguments (negative SQRT, zero LOG)
  - [x] 18.5 Write tests: verify math functions with edge cases

- [x] 19. Text Functions
  - [x] 19.1 Create `src/formula_engine/functions/text.rs`
  - [x] 19.2 Implement CONCATENATE, CONCAT, LEFT, RIGHT, MID, LEN, TRIM
  - [x] 19.3 Implement UPPER, LOWER, SUBSTITUTE
  - [x] 19.4 Implement number-to-string coercion for text functions
  - [x] 19.5 Write tests: verify text functions including overflow cases

- [x] 20. Date Functions
  - [x] 20.1 Create `src/formula_engine/functions/date.rs`
  - [x] 20.2 Implement TODAY, NOW, DATE, YEAR, MONTH, DAY
  - [x] 20.3 Implement EDATE, EOMONTH, NETWORKDAYS
  - [x] 20.4 Implement date rollover for out-of-range month/day values
  - [x] 20.5 Write tests: verify date functions with boundary cases

- [x] 21. Lookup Functions
  - [x] 21.1 Create `src/formula_engine/functions/lookup.rs`
  - [x] 21.2 Implement HLOOKUP (exact and approximate match)
  - [x] 21.3 Implement XLOOKUP (match mode, search mode, if-not-found)
  - [x] 21.4 Implement MATCH (exact, less than, greater than)
  - [x] 21.5 Implement INDIRECT (string → cell reference resolution)
  - [x] 21.6 Implement OFFSET (reference shifting)
  - [x] 21.7 Write tests: verify lookup functions with various match modes

- [x] 22. Statistical Functions
  - [x] 22.1 Create `src/formula_engine/functions/statistical.rs`
  - [x] 22.2 Implement STDEV, VAR, MEDIAN, PERCENTILE, RANK
  - [x] 22.3 Implement COUNTIF, SUMIF with criteria parsing (operators and wildcards)
  - [x] 22.4 Implement #DIV/0! for insufficient data points
  - [x] 22.5 Write tests: verify statistical functions with edge cases

- [x] 23. Logical Functions
  - [x] 23.1 Create `src/formula_engine/functions/logical.rs`
  - [x] 23.2 Implement AND, OR, NOT
  - [x] 23.3 Implement IFERROR, IFNA
  - [x] 23.4 Implement SWITCH, IFS
  - [x] 23.5 Write tests: verify logical functions with error propagation

- [x] 24. Array Formula Evaluation
  - [x] 24.1 Extend evaluator to handle CSE array formulas (multi-cell output)
  - [x] 24.2 Implement dynamic array spill range computation
  - [x] 24.3 Implement #SPILL! error when spill range overlaps non-empty cells
  - [x] 24.4 Write tests: CSE array formula, dynamic spill, spill conflict

- [x] 25. Circular Reference Detection and Volatile Functions
  - [x] 25.1 Integrate cycle detection into `recalculate()` flow
  - [x] 25.2 Return `Error::CircularReference` with involved cell addresses
  - [x] 25.3 Mark RAND, RANDBETWEEN, NOW, TODAY, INDIRECT as volatile
  - [x] 25.4 Re-evaluate volatile cells and dependents on each recalculate call
  - [x] 25.5 Write tests: circular ref error, volatile function re-evaluation


## Phase 3: Chart Completeness

- [x] 26. Bubble Charts
  - [x] 26.1 Add `ChartType::Bubble` variant to enum
  - [x] 26.2 Add `bubble_sizes: Option<String>` field to `ChartSeries`
  - [x] 26.3 Implement bubble chart XML serialization in `chart_writer.rs`
  - [x] 26.4 Write example and test: create bubble chart, validate in Excel
  - [x] 26.5 Write example (`examples/bubble_chart.rs`): create bubble chart with sample data

- [x] 27. Waterfall Charts (ChartEx)
  - [x] 27.1 Add `ChartExType::Waterfall` and corresponding struct
  - [x] 27.2 Support data point categorization (total, increase, decrease)
  - [x] 27.3 Implement waterfall ChartEx XML serialization in `chartex_writer.rs`
  - [x] 27.4 Write test: create waterfall chart, verify XML structure

- [x] 28. Funnel Charts (ChartEx)
  - [x] 28.1 Add `ChartExType::Funnel` and corresponding struct
  - [x] 28.2 Implement funnel ChartEx XML serialization
  - [x] 28.3 Write test: create funnel chart, verify XML structure

- [x] 29. Sunburst Charts (ChartEx)
  - [x] 29.1 Add `ChartExType::Sunburst` and corresponding struct
  - [x] 29.2 Support multi-level hierarchy (at least 3 levels)
  - [x] 29.3 Implement sunburst ChartEx XML serialization
  - [x] 29.4 Write test: create sunburst chart with 3 levels, verify XML

- [x] 30. Histogram and Pareto Charts (ChartEx)
  - [x] 30.1 Add `ChartExType::Histogram` and `ChartExType::Pareto` structs
  - [x] 30.2 Support configurable bin count and bin width
  - [x] 30.3 Implement histogram/Pareto ChartEx XML serialization
  - [x] 30.4 Write test: create histogram with custom bins, verify XML

- [x] 31. Box and Whisker Charts (ChartEx)
  - [x] 31.1 Add `ChartExType::BoxWhisker` and corresponding struct
  - [x] 31.2 Support outlier points, mean markers, inner points options
  - [x] 31.3 Implement box & whisker ChartEx XML serialization
  - [x] 31.4 Write test: create box & whisker chart, verify XML

- [x] 32. Map Charts
  - [x] 32.1 Add `ChartType::Map` variant
  - [x] 32.2 Support region-level and country-level granularity
  - [x] 32.3 Implement map chart XML serialization
  - [x] 32.4 Write test: create map chart, verify XML structure

- [x] 33. 3D Chart Variants
  - [x] 33.1 Add `Column3D`, `Bar3D`, `Line3D`, `Pie3D`, `Area3D` to `ChartType`
  - [x] 33.2 Add `View3D` struct (rot_x, rot_y, perspective, right_angle_axes)
  - [x] 33.3 Serialize `<c:view3D>` element for 3D charts
  - [x] 33.4 Write test: create 3D column chart, verify view3D in XML

- [x] 34. Surface Charts
  - [x] 34.1 Add `ChartType::Surface` and `ChartType::WireframeSurface`
  - [x] 34.2 Implement surface chart XML serialization with 3D view
  - [x] 34.3 Write test: create surface chart, verify XML structure

- [x] 35. Chart Style Themes
  - [x] 35.1 Add `style: Option<u8>` field to `Chart`
  - [x] 35.2 Add `set_style(n: u8)` method
  - [x] 35.3 Serialize `<c:style val="N"/>` in chart XML
  - [x] 35.4 Write test: set chart style, verify serialization

- [x] 36. Axis Formatting
  - [x] 36.1 Add `AxisFormat` struct (num_format, font, font_size, tick_marks, gridline_style)
  - [x] 36.2 Add `set_x_axis_format()` and `set_y_axis_format()` methods on `Chart`
  - [x] 36.3 Serialize axis formatting elements (numFmt, txPr, majorTickMark, majorGridlines)
  - [x] 36.4 Write test: format axes, verify XML elements

- [x] 37. Plot Area and Series Formatting
  - [x] 37.1 Add `PlotAreaFormat` struct (fill, border, gradient)
  - [x] 37.2 Add `set_plot_area_format()` method on `Chart`
  - [x] 37.3 Extend `ChartSeries` with line_width, dash_style, gradient fields
  - [x] 37.4 Add `ErrorBar` struct and `set_error_bars()` on `ChartSeries`
  - [x] 37.5 Serialize plot area and series formatting in chart XML
  - [x] 37.6 Write test: format plot area and series, verify XML

- [x] 38. Chart Accessories (Drop Lines, High-Low Lines, Chart Sheets)
  - [x] 38.1 Add drop lines and high-low lines options to line charts
  - [x] 38.2 Serialize `<c:dropLines>` and `<c:hiLowLines>` elements
  - [x] 38.3 Add `ChartSheet` support — create dedicated chartsheet XML part
  - [x] 38.4 Update content types and relationships for chart sheets
  - [x] 38.5 Write test: chart sheet creation, verify file structure


## Phase 4: Advanced Formatting

- [x] 39. Gradient Fills
  - [x] 39.1 Add `GradientFill` and `GradientStop` structs to `format.rs`
  - [x] 39.2 Add `Format::gradient_fill(angle, stops)` builder method
  - [x] 39.3 Extend `StyleRegistry` to register gradient fills
  - [x] 39.4 Serialize `<gradientFill>` element in `style_writer.rs`
  - [x] 39.5 Write test: apply gradient fill, verify styles XML
  - [x] 39.6 Write example (`examples/gradient_fills.rs`): create cells with gradient fills

- [x] 40. Theme Colors
  - [x] 40.1 Add `ThemeColor` and `ThemeColorIndex` enums
  - [x] 40.2 Add `Format::theme_color(index, tint)` builder method
  - [x] 40.3 Serialize theme index and tint in font/fill color elements
  - [x] 40.4 Parse theme colors from `xl/theme/theme1.xml` in reader
  - [x] 40.5 Resolve theme color references to RGB during read
  - [x] 40.6 Write test: use theme colors, verify XML serialization

- [x] 41. Gradient Data Bars
  - [x] 41.1 Add `gradient: bool` field to `ConditionalFormatDataBar`
  - [x] 41.2 Add `set_gradient(true)` method
  - [x] 41.3 Serialize gradient attribute in data bar CF rule XML
  - [x] 41.4 Write test: gradient data bar, verify XML attribute

- [x] 42. Cell Styles
  - [x] 42.1 Add `Format::cell_style(name: &str)` builder method
  - [x] 42.2 Map style names to cellStyleXfs indices in `StyleRegistry`
  - [x] 42.3 Serialize `<cellStyle>` references in styles XML
  - [x] 42.4 Write test: apply named cell style, verify styles XML

- [x] 43. Custom Table Styles
  - [x] 43.1 Add `CustomTableStyle` struct with stripe sizes and element formatting
  - [x] 43.2 Add `Table::set_custom_style(style)` method
  - [x] 43.3 Serialize `<tableStyle>` element in styles XML
  - [x] 43.4 Write test: custom table style, verify XML

- [x] 44. Phonetic Text (Furigana)
  - [x] 44.1 Add `PhoneticRun` struct (start_index, end_index, text)
  - [x] 44.2 Add `Worksheet::set_phonetic(row, col, runs)` method
  - [x] 44.3 Serialize `<phoneticPr>` and `<rPh>` elements in sheet XML
  - [x] 44.4 Write test: add phonetic text, verify XML elements

- [x] 45. Text Effects
  - [x] 45.1 Add `shadow`, `outline`, `emboss`, `engrave` fields to `Format`
  - [x] 45.2 Add corresponding builder methods
  - [x] 45.3 Serialize text effect attributes in font elements
  - [x] 45.4 Write test: apply text effects, verify font XML


## Phase 5: Data & Interactivity

- [x] 46. Slicers
  - [x] 46.1 Create `src/features/slicer.rs` with `Slicer` struct
  - [x] 46.2 Add `Worksheet::add_slicer(row, col, slicer)` method
  - [x] 46.3 Implement slicer XML part serialization (slicerN.xml)
  - [x] 46.4 Implement slicer cache XML serialization
  - [x] 46.5 Add slicer drawing relationship and content type entries
  - [x] 46.6 Support linking slicer to pivot table cache
  - [x] 46.7 Implement slicer reading in reader module
  - [x] 46.8 Write test: add slicer to table, verify file structure
  - [x] 46.9 Write example (`examples/slicers.rs`): add slicers to a table, verify file structure

- [x] 47. Timelines
  - [x] 47.1 Create `src/features/timeline.rs` with `Timeline` struct
  - [x] 47.2 Add `Worksheet::add_timeline(row, col, timeline)` method
  - [x] 47.3 Implement timeline XML part and cache serialization
  - [x] 47.4 Implement timeline reading in reader module
  - [x] 47.5 Write test: add timeline to pivot table, verify file structure

- [x] 48. Named Ranges CRUD
  - [x] 48.1 Add `Workbook::add_named_range(name, formula, scope)` method
  - [x] 48.2 Add `Workbook::update_named_range(name, new_formula)` method
  - [x] 48.3 Add `Workbook::remove_named_range(name, scope)` method
  - [x] 48.4 Serialize updated `<definedNames>` in workbook XML
  - [x] 48.5 Write test: CRUD operations on named ranges, verify XML
  - [x] 48.6 Write example (`examples/named_ranges.rs`): create, update, delete named ranges

- [x] 49. External Data Connections Preservation
  - [x] 49.1 Detect and preserve `xl/connections.xml` during edit-mode save
  - [x] 49.2 Add `Workbook::add_connection(connection_string, command)` method
  - [x] 49.3 Write test: open file with connections, modify, save, verify preservation

- [x] 50. Power Query Preservation
  - [x] 50.1 Detect and preserve `customXml/` parts containing Power Query metadata during save
  - [x] 50.2 Write test: open file with Power Query, modify cells, save, verify PQ parts preserved

- [x] 51. Pivot Table Grouping
  - [x] 51.1 Add `PivotTable::group_by_date(field, levels)` method
  - [x] 51.2 Add `PivotTable::group_by_range(field, start, end, interval)` method
  - [x] 51.3 Serialize `<fieldGroup>` and `<rangePr>` in pivot cache definition XML
  - [x] 51.4 Write test: date grouping and numeric grouping, verify XML

- [x] 52. Pivot Table Calculated Items
  - [x] 52.1 Add `PivotTable::add_calculated_item(name, formula)` method
  - [x] 52.2 Serialize `<calculatedItem>` in pivot table definition XML
  - [x] 52.3 Write test: add calculated item, verify XML

- [x] 53. Sort State
  - [x] 53.1 Preserve `<sortState>` element during edit-mode save
  - [x] 53.2 Add `Worksheet::set_sort(col, direction)` method
  - [x] 53.3 Serialize `<sortState>` and `<sortCondition>` elements
  - [x] 53.4 Write test: set sort, verify XML; round-trip preservation test

- [x] 54. Advanced Autofilter
  - [x] 54.1 Add `FilterRule` enum (Top10, DateFilter, CustomFilter)
  - [x] 54.2 Add `Worksheet::filter_column_advanced(col, rule)` method
  - [x] 54.3 Serialize `<top10>`, `<dateGroupItem>`, and `<customFilters>` elements
  - [x] 54.4 Write test: top-10 filter, date filter, custom AND/OR filter


## Phase 6: Streaming & Performance

- [x] 55. Streaming Read
  - [x] 55.1 Create `src/reader/streaming_reader.rs` with `StreamingReader` struct
  - [x] 55.2 Implement SST upfront loading (shared strings are small relative to data)
  - [x] 55.3 Implement SAX-style row iterator using `quick-xml` event reader
  - [x] 55.4 Implement `StreamingRow` and `StreamingCell` types
  - [x] 55.5 Resolve shared string references during iteration
  - [x] 55.6 Support reading xf indices per cell
  - [x] 55.7 Expose `StreamingReader::open(path)` and `StreamingReader::sheet(index)` API
  - [x] 55.8 Write test: stream-read a large file, verify constant memory (no OOM on 1M rows)
  - [x] 55.9 Write example (`examples/streaming_read.rs`): stream-read a large file, print row count and sample data

- [x] 56. Streaming Write with Charts and Images
  - [x] 56.1 Add `StreamingWorkbook::insert_chart(row, col, chart)` method
  - [x] 56.2 Add `StreamingWorkbook::insert_image(row, col, image)` method
  - [x] 56.3 Buffer chart/image data and serialize drawing parts at finalize
  - [x] 56.4 Write test: streaming write with chart and image, verify file opens in Excel

- [x] 57. Streaming Write with Conditional Formatting
  - [x] 57.1 Add `StreamingWorkbook::add_conditional_format(range, cf)` method
  - [x] 57.2 Buffer CF rules and serialize after `</sheetData>` during finalize
  - [x] 57.3 Write test: streaming write with CF rules, verify XML

- [x] 58. Parallel Sheet Writing
  - [x] 58.1 Wrap `SharedStringTable` and `StyleRegistry` in `Arc<Mutex<_>>`
  - [x] 58.2 Add `Workbook::save_parallel(path)` method using `std::thread::scope`
  - [x] 58.3 Serialize each sheet on a separate thread, join before ZIP assembly
  - [x] 58.4 Write test: parallel save produces identical output to sequential save

- [x] 59. Memory-Mapped Reading
  - [x] 59.1 Add optional `memmap2` dependency behind `mmap` feature flag
  - [x] 59.2 Add `Workbook::open_mmap(path)` method
  - [x] 59.3 Pass mmap slice to ZIP reader instead of reading into heap
  - [x] 59.4 Write test: open large file via mmap, verify cell reads work

- [x] 60. Incremental Save
  - [x] 60.1 Track dirty flag per sheet (already exists)
  - [x] 60.2 During edit-mode save, copy original ZIP entries for non-dirty sheets
  - [x] 60.3 Only re-serialize sheets where `dirty == true`
  - [x] 60.4 Write test: modify one sheet in 10-sheet workbook, verify only that sheet is re-serialized

- [x] 61. Shared String Deduplication Tuning
  - [x] 61.1 Add `Workbook::set_sst_threshold(n: usize)` configuration method
  - [x] 61.2 In finalize, inline strings appearing fewer than N times
  - [x] 61.3 Write test: set threshold=2, verify single-use strings are inlined


## Phase 7: File Format Variants

- [x] 62. XLSM Write
  - [x] 62.1 Add `Workbook::save_as_xlsm(path, vba_project: &[u8])` method
  - [x] 62.2 Set macro-enabled content types (`vnd.ms-excel.sheet.macroEnabled.main+xml`)
  - [x] 62.3 Include `xl/vbaProject.bin` in ZIP output
  - [x] 62.4 Return error if no VBA project provided
  - [x] 62.5 Write test: save as XLSM with VBA binary, verify content types
  - [x] 62.6 Write example (`examples/xlsm_write.rs`): create macro-enabled workbook

- [x] 63. Template Formats (XLTX/XLTM)
  - [x] 63.1 Add `Workbook::save_as_template(path)` and `save_as_template_macro(path)` methods
  - [x] 63.2 Set template content types
  - [x] 63.3 Write test: save as XLTX, verify content type in `[Content_Types].xml`

- [x] 64. XLSB Read
  - [x] 64.1 Create `src/formats/xlsb_reader.rs` behind `xlsb` feature flag
  - [x] 64.2 Detect XLSB via content types in ZIP
  - [x] 64.3 Implement binary record stream parser (record type ID + length + payload)
  - [x] 64.4 Parse BrtCellBlank, BrtCellReal, BrtCellSt, BrtCellBool, BrtFmla records
  - [x] 64.5 Map parsed records to `CellValue` and `SheetMeta`
  - [x] 64.6 Skip unsupported records gracefully
  - [x] 64.7 Write test: read XLSB file, verify cell values match expected

- [x] 65. XLS Read (BIFF8)
  - [x] 65.1 Create `src/formats/xls_reader.rs` behind `xls` feature flag
  - [x] 65.2 Implement OLE2 compound document header and directory parsing
  - [x] 65.3 Locate Workbook stream in OLE2 directory
  - [x] 65.4 Parse BIFF8 records (BOF, SHEET, SST, LABELSST, NUMBER, FORMULA, EOF)
  - [x] 65.5 Map parsed records to `CellValue` and sheet names
  - [x] 65.6 Skip unsupported records gracefully
  - [x] 65.7 Write test: read XLS file, verify cell values

- [x] 66. CSV/TSV Export
  - [x] 66.1 Create `src/formats/csv_export.rs`
  - [x] 66.2 Add `CsvOptions` struct (delimiter, quote, line_ending, date_format)
  - [x] 66.3 Add `Worksheet::to_csv(options) -> String` method
  - [x] 66.4 Add `Worksheet::to_csv_file(path, options)` method
  - [x] 66.5 Format date cells as ISO 8601 by default
  - [x] 66.6 Escape fields containing delimiter, quote, or newlines
  - [x] 66.7 Write test: export sheet to CSV, verify output matches expected
  - [x] 66.8 Write example (`examples/csv_export.rs`): create workbook, export to CSV, print output

- [x] 67. ODS Read/Write
  - [x] 67.1 Create `src/formats/ods.rs` behind `ods` feature flag
  - [x] 67.2 Implement ODS reader: parse `content.xml` `<table:table>` elements
  - [x] 67.3 Map ODS cell types to `CellValue`
  - [x] 67.4 Implement ODS writer: serialize workbook model to ODS XML
  - [x] 67.5 Skip unsupported ODS features gracefully
  - [x] 67.6 Write test: round-trip simple workbook through ODS format

- [x] 68. Strict OOXML
  - [x] 68.1 Create `src/formats/strict_ooxml.rs`
  - [x] 68.2 Detect Strict namespace URIs during read
  - [x] 68.3 Map Strict URIs to Transitional namespace handlers
  - [x] 68.4 Add `Workbook::save_strict(path)` for Strict OOXML output
  - [x] 68.5 Write test: read Strict OOXML file, verify cell values


## Phase 8: Document Features

- [x] 69. Threaded Comments
  - [x] 69.1 Add `ThreadedComment` struct (author, timestamp, text, replies)
  - [x] 69.2 Add `Worksheet::add_threaded_comment(row, col, comment)` method
  - [x] 69.3 Serialize ThreadedComments XML part (`xl/threadedComments/threadedComment1.xml`)
  - [x] 69.4 Add person list XML part (`xl/persons/person.xml`)
  - [x] 69.5 Implement threaded comment reading in `comment_reader.rs`
  - [x] 69.6 Write test: add threaded comment with reply, read back, verify chain

- [x] 70. Form Controls
  - [x] 70.1 Add `FormControl` enum (Checkbox, Dropdown, Button, Spinner)
  - [x] 70.2 Add `Worksheet::add_form_control(row, col, control)` method
  - [x] 70.3 Serialize form control XML in VML drawing part
  - [x] 70.4 Support cell link for checkbox and spinner
  - [x] 70.5 Implement form control reading
  - [x] 70.6 Write test: add checkbox with cell link, verify XML

- [x] 71. ActiveX and OLE Preservation
  - [x] 71.1 Detect ActiveX parts (`xl/activeX/`) during edit-mode open
  - [x] 71.2 Preserve ActiveX binary parts and relationships during save
  - [x] 71.3 Detect OLE object parts during edit-mode open
  - [x] 71.4 Preserve OLE binary parts and relationships during save
  - [x] 71.5 Write test: open file with ActiveX/OLE, modify cells, save, verify parts preserved

- [x] 72. Drawing Shapes
  - [x] 72.1 Create `src/features/shape.rs` with `Shape` and `ShapeType` structs
  - [x] 72.2 Add `Worksheet::add_shape(row, col, shape)` method
  - [x] 72.3 Serialize shapes as `<xdr:sp>` elements in drawing XML
  - [x] 72.4 Support fill color, outline color, outline width
  - [x] 72.5 Support text body with font and paragraph formatting
  - [x] 72.6 Write test: add rectangle with text, verify drawing XML
  - [x] 72.7 Write example (`examples/drawing_shapes.rs`): add various shapes to a worksheet

- [x] 73. SmartArt and Equation Preservation
  - [x] 73.1 Detect SmartArt diagram parts (`xl/diagrams/`) during edit-mode open
  - [x] 73.2 Preserve SmartArt XML parts and relationships during save
  - [x] 73.3 Preserve OMML equation elements during save
  - [x] 73.4 Write test: open file with SmartArt, modify cells, save, verify diagram parts preserved

- [x] 74. Digital Signatures
  - [x] 74.1 Add `Workbook::sign(certificate)` method behind `crypto` feature flag
  - [x] 74.2 Implement XML Digital Signature (XMLDSig) generation
  - [x] 74.3 Add `Workbook::verify_signature() -> Result<bool>` method
  - [x] 74.4 Invalidate signature when workbook is modified
  - [x] 74.5 Write test: sign workbook, verify signature, modify, verify invalidation

- [x] 75. Custom XML Parts
  - [x] 75.1 Add `Workbook::add_custom_xml(namespace, content)` method
  - [x] 75.2 Add `Workbook::read_custom_xml(namespace) -> Option<&[u8]>` method
  - [x] 75.3 Serialize custom XML in `customXml/` directory with content type entries
  - [x] 75.4 Write test: add custom XML, save, read back, verify content

- [x] 76. Custom Document Properties
  - [x] 76.1 Add `CustomProperty` struct (name, type, value)
  - [x] 76.2 Add `Workbook::set_custom_property(name, value)` method
  - [x] 76.3 Parse `docProps/custom.xml` in reader
  - [x] 76.4 Serialize `docProps/custom.xml` in writer
  - [x] 76.5 Write test: set custom properties, read back, verify values


## Phase 9: Security & Compliance

- [x] 77. File Encryption (Read)
  - [x] 77.1 Add `aes`, `sha2`, `hmac`, `cbc`, `pbkdf2` dependencies behind `crypto` feature flag
  - [x] 77.2 Create `src/crypto/mod.rs` and `src/crypto/standard.rs`
  - [x] 77.3 Detect encrypted files by OLE2 magic bytes (`D0 CF 11 E0`)
  - [x] 77.4 Parse EncryptionInfo stream from OLE2 compound document
  - [x] 77.5 Implement ECMA-376 Standard Encryption decryption (AES-128-CBC, SHA-1)
  - [x] 77.6 Create `src/crypto/agile.rs`
  - [x] 77.7 Implement ECMA-376 Agile Encryption decryption (AES-256-CBC, SHA-512, HMAC)
  - [x] 77.8 Add `Workbook::open_with_password(path, password)` method
  - [x] 77.9 Return `Error::Authentication` for incorrect passwords
  - [x] 77.10 Write test: decrypt Standard-encrypted file, verify cell values
  - [x] 77.11 Write test: decrypt Agile-encrypted file, verify cell values
  - [x] 77.12 Write example (`examples/encrypted_read.rs`): open password-protected file, print cell values

- [x] 78. File Encryption (Write)
  - [x] 78.1 Implement Agile Encryption packaging (encrypt xlsx bytes into OLE2 container)
  - [x] 78.2 Add `Workbook::save_encrypted(path, password)` method
  - [x] 78.3 Write test: encrypt file, decrypt with same password, verify round-trip

- [x] 79. Sheet Protection Granularity
  - [x] 79.1 Extend `SheetProtection` with individual flags (sort, autoFilter, pivotTables, insertColumns, insertRows, deleteColumns, deleteRows, formatCells, formatColumns, formatRows, insertHyperlinks, selectLockedCells, selectUnlockedCells)
  - [x] 79.2 Add builder methods for each flag on `SheetProtection`
  - [x] 79.3 Serialize each flag as attribute on `<sheetProtection>` element
  - [x] 79.4 Write test: set granular protection flags, verify XML attributes

- [x] 80. VBA Project Signing
  - [x] 80.1 Add `Workbook::sign_vba(certificate)` method behind `crypto` feature flag
  - [x] 80.2 Generate digital signature for VBA project binary
  - [x] 80.3 Serialize signature in `xl/vbaProjectSignature.bin`
  - [x] 80.4 Write test: sign VBA project, verify signature part exists

- [x] 81. IRM Metadata Preservation
  - [x] 81.1 Detect IRM-related parts during edit-mode open
  - [x] 81.2 Preserve IRM parts and relationships during save
  - [x] 81.3 Write test: open IRM file, modify, save, verify IRM parts preserved

- [x] 82. Accessibility Metadata
  - [x] 82.1 Add `alt_text: Option<(String, String)>` (title, description) to `Chart`, `Image`, `Table`
  - [x] 82.2 Add `set_alt_text(title, description)` methods
  - [x] 82.3 Serialize `descr` and `title` attributes on `<xdr:cNvPr>` elements
  - [x] 82.4 Write test: set alt text on chart/image/table, verify XML attributes


## Phase 10: Ecosystem

- [x] 83. Serde Integration
  - [x] 83.1 Add `serde` and `serde_derive` dependencies behind `serde` feature flag
  - [x] 83.2 Create `src/serde_support/mod.rs` and `src/serde_support/ser.rs`
  - [x] 83.3 Implement custom `Serializer` that maps struct fields to columns
  - [x] 83.4 Add `Workbook::write_rows<T: Serialize>(sheet, data)` method
  - [x] 83.5 Auto-generate header row from field names
  - [x] 83.6 Create `src/serde_support/de.rs`
  - [x] 83.7 Implement custom `Deserializer` that reads rows using header mapping
  - [x] 83.8 Add `Workbook::read_rows<T: DeserializeOwned>(sheet) -> Result<Vec<T>>` method
  - [x] 83.9 Return descriptive error on type mismatch
  - [x] 83.10 Write round-trip property test: serialize Vec<T>, deserialize, verify equality
  - [x] 83.11 Write example (`examples/serde_roundtrip.rs`): serialize structs to xlsx, deserialize back, print results

- [x] 84. Derive Macro (#[derive(ExcelRow)])
  - [x] 84.1 Create `zavora-xlsx-derive` workspace member crate
  - [x] 84.2 Implement proc macro parsing struct fields and attributes
  - [x] 84.3 Support `#[excel(header = "...")]` attribute for column names
  - [x] 84.4 Support `#[excel(format = "...")]` attribute for number formats
  - [x] 84.5 Handle `Option<T>` fields (empty cells → None)
  - [x] 84.6 Generate `Serialize` and `Deserialize` trait implementations
  - [x] 84.7 Write test: derive macro on struct, write and read rows, verify

- [x] 85. Async I/O
  - [x] 85.1 Add `tokio` dependency behind `async-tokio` feature flag
  - [x] 85.2 Add `Workbook::open_async(path).await` method
  - [x] 85.3 Add `Workbook::save_async(path).await` method
  - [x] 85.4 Perform file I/O on tokio's blocking thread pool
  - [x] 85.5 Write test: async open and save, verify file contents

- [x] 86. WASM Target
  - [x] 86.1 Add `#[cfg(not(target_arch = "wasm32"))]` guards on file system operations
  - [x] 86.2 Ensure `save_to_buffer()` and `open_from_buffer()` work without fs
  - [x] 86.3 Add `wasm-bindgen` exports for buffer-based API behind `wasm` feature flag
  - [x] 86.4 Verify compilation: `cargo build --target wasm32-unknown-unknown --no-default-features`
  - [x] 86.5 Write test: WASM build succeeds, buffer round-trip works

- [x] 87. Python Bindings (PyO3)
  - [x] 87.1 Create `zavora-xlsx-python` workspace member crate with PyO3
  - [x] 87.2 Expose `Workbook`, `Worksheet`, `Format`, `Chart`, `Table` Python classes
  - [x] 87.3 Implement cell writing methods (write_string, write_number, write_formula)
  - [x] 87.4 Implement format application methods
  - [x] 87.5 Implement save to file and save to bytes
  - [x] 87.6 Configure `maturin` for `pip install` packaging
  - [x] 87.7 Write Python test: create workbook, write data, save, verify

- [x] 88. Node.js Bindings (napi-rs)
  - [x] 88.1 Create `zavora-xlsx-node` workspace member crate with napi-rs
  - [x] 88.2 Expose `Workbook`, `Worksheet`, `Format`, `Chart`, `Table` JS classes
  - [x] 88.3 Implement cell writing methods
  - [x] 88.4 Implement save to file and save to Buffer
  - [x] 88.5 Configure `napi-build` for `npm install` packaging
  - [x] 88.6 Write Node.js test: create workbook, write data, save, verify

- [x] 89. C FFI
  - [x] 89.1 Create `src/cffi.rs` behind `cffi` feature flag
  - [x] 89.2 Implement opaque pointer handle pattern for Workbook
  - [x] 89.3 Expose `zavora_workbook_new`, `zavora_worksheet_write_*`, `zavora_workbook_save`, `zavora_workbook_free`
  - [x] 89.4 Implement `zavora_last_error()` for error message retrieval
  - [x] 89.5 Generate C header file using `cbindgen`
  - [x] 89.6 Write C test: create workbook, write cells, save

- [x] 90. CLI Tool
  - [x] 90.1 Create `zavora-xlsx-cli` workspace member crate with `clap`
  - [x] 90.2 Implement `inspect` subcommand (sheet names, row/col counts, metadata)
  - [x] 90.3 Implement `export` subcommand (sheet to CSV with options)
  - [x] 90.4 Implement `convert` subcommand (xlsx ↔ xlsm ↔ csv)
  - [x] 90.5 Print descriptive errors to stderr, exit with non-zero status
  - [x] 90.6 Write integration test: CLI inspect and export commands
  - [x] 90.7 Write example usage in CLI README: demonstrate inspect, export, convert commands
