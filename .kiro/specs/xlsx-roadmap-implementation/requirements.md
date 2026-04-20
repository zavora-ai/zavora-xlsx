# Requirements Document — zavora-xlsx Roadmap Implementation

## Introduction

This document specifies the requirements for completing the zavora-xlsx roadmap: a comprehensive set of features that bring the crate from its current v0.1.0 baseline to full Excel OOXML feature parity. The roadmap spans 10 phases covering read parity, a formula engine, chart completeness, advanced formatting, data and interactivity features, streaming and performance, file format variants, document features, security and compliance, and ecosystem integrations.

Requirements are organized by phase and prioritized according to the roadmap's recommended launch order: Phase 1 (Read Parity), Phase 10.1–10.2 (serde), Phase 6.1 (Streaming read), Phase 3 (Chart completeness), Phase 9.1 (Encryption), then remaining phases.

## Glossary

- **Workbook**: The top-level container representing an Excel `.xlsx` file, containing one or more Worksheets
- **Worksheet**: A single sheet within a Workbook, containing cells, formatting, and features
- **Reader**: The module responsible for parsing `.xlsx` ZIP archives and XML into in-memory structs
- **Writer**: The module responsible for serializing in-memory structs into `.xlsx` ZIP archives and XML
- **StreamingReader**: A SAX-style row-by-row reader that processes large files with constant memory
- **StreamingWriter**: The existing constant-memory writer for large file generation
- **FormulaEngine**: The subsystem that parses, evaluates, and manages cell formula calculations
- **DependencyGraph**: A directed acyclic graph tracking cell-to-cell formula dependencies for calculation order
- **ChartEngine**: The subsystem responsible for creating and serializing chart XML (both c:chart and cx:chart namespaces)
- **StyleRegistry**: The internal registry mapping xf indices to font, fill, border, alignment, and number format records
- **SharedStringTable (SST)**: The deduplicated table of string values shared across cells
- **ParsedStyles**: The struct holding parsed style information from `xl/styles.xml`
- **SheetMeta**: Metadata parsed from sheet XML including merges, widths, heights, and freeze panes
- **CellValue**: The enum representing possible cell contents (String, Number, Bool, Formula, DateTime, Error, Empty)
- **Format**: The builder struct for cell formatting (font, fill, border, alignment, number format)
- **OOXML**: Office Open XML, the ISO/IEC 29500 standard for Office documents
- **ChartEx**: The extended chart namespace (`cx:chart`) used for modern chart types (treemap, waterfall, funnel, sunburst, histogram, box & whisker)
- **DXF**: Differential formatting record used by conditional formatting rules
- **CF**: Conditional formatting
- **SST**: Shared String Table
- **CSE**: Ctrl+Shift+Enter array formula
- **XLSM**: Macro-enabled Excel workbook format
- **XLTX**: Excel template format
- **XLTM**: Macro-enabled Excel template format
- **XLSB**: Excel binary workbook format
- **BIFF8**: Binary Interchange File Format version 8 (legacy `.xls`)
- **ODS**: OpenDocument Spreadsheet format
- **Serde**: The Rust serialization/deserialization framework
- **PyO3**: Rust-Python binding framework
- **napi-rs**: Rust-Node.js binding framework
- **WASM**: WebAssembly compilation target
- **IRM**: Information Rights Management

---

## Requirements

### Requirement 1: Read Cell Formatting

**User Story:** As a developer, I want to read cell formatting from existing xlsx files, so that I can analyze or migrate spreadsheet styles programmatically.

#### Acceptance Criteria

1. WHEN an xlsx file is opened for reading, THE Reader SHALL parse each cell's xf index back to its corresponding font, fill, border, alignment, and number format records
2. WHEN a cell has a non-default format, THE Reader SHALL populate a Format struct accessible via the Worksheet API
3. WHEN a cell references a shared style (xf index), THE Reader SHALL resolve the full style chain including font, fill, border, alignment, and number format
4. IF a cell references an xf index that does not exist in the styles, THEN THE Reader SHALL fall back to the default format without returning an error

### Requirement 2: Read Conditional Formatting

**User Story:** As a developer, I want to read conditional formatting rules from existing xlsx files, so that I can inspect or replicate conditional formatting logic.

#### Acceptance Criteria

1. WHEN an xlsx file containing conditional formatting is opened, THE Reader SHALL parse all CF rules from the sheet XML into ConditionalFormat structs
2. WHEN a CF rule uses a DXF style, THE Reader SHALL resolve the DXF index to its font, fill, border, and number format components
3. THE Reader SHALL support reading all 11 CF rule types that the Writer currently supports (cell value, 2-color scale, 3-color scale, data bar, icon set, formula, top/bottom, text, duplicate, unique, average, date)

### Requirement 3: Read Data Validation

**User Story:** As a developer, I want to read data validation rules from existing xlsx files, so that I can inspect or replicate validation constraints.

#### Acceptance Criteria

1. WHEN an xlsx file containing data validation is opened, THE Reader SHALL parse all validation rules from the sheet XML into DataValidation structs
2. THE Reader SHALL support reading all 7 validation rule types that the Writer currently supports (list, whole number, decimal, date, time, text length, custom formula)
3. WHEN a validation rule includes input or error messages, THE Reader SHALL preserve the message title and body text

### Requirement 4: Read Charts

**User Story:** As a developer, I want to read chart definitions from existing xlsx files, so that I can inspect, modify, or migrate chart configurations.

#### Acceptance Criteria

1. WHEN an xlsx file containing charts is opened, THE Reader SHALL parse chart XML (both `c:chart` and `cx:chart` namespaces) into Chart structs
2. WHEN a chart contains series data, THE Reader SHALL parse series names, values references, and categories references
3. WHEN a chart has a title, legend, or axis names, THE Reader SHALL populate the corresponding Chart struct fields
4. THE Reader SHALL support reading all 9 chart types that the Writer currently supports (Column, Bar, Line, Pie, Scatter, Area, Doughnut, Radar, Stock)
5. WHEN a chart uses the ChartEx namespace, THE Reader SHALL parse it into the corresponding ChartEx struct (Treemap)

### Requirement 5: Read Tables

**User Story:** As a developer, I want to read table definitions from existing xlsx files, so that I can inspect table structure, styles, and autofilter settings.

#### Acceptance Criteria

1. WHEN an xlsx file containing tables is opened, THE Reader SHALL parse table part XML into Table structs
2. WHEN a table has column definitions, THE Reader SHALL populate TableColumn structs with header names and total row formulas
3. WHEN a table has a style applied, THE Reader SHALL resolve the TableStyle enum value
4. WHEN a table has an autofilter, THE Reader SHALL parse the filter range and column criteria

### Requirement 6: Read Hyperlinks

**User Story:** As a developer, I want to read hyperlinks from existing xlsx files, so that I can extract or migrate link targets.

#### Acceptance Criteria

1. WHEN an xlsx file containing hyperlinks is opened, THE Reader SHALL parse hyperlink relationships from the sheet XML and relationship files
2. WHEN a hyperlink targets an external URL, THE Reader SHALL populate the Hyperlink struct with the URL and display text
3. WHEN a hyperlink targets an internal sheet location, THE Reader SHALL populate the Hyperlink struct with the sheet reference

### Requirement 7: Read Comments and Notes

**User Story:** As a developer, I want to read comments from existing xlsx files, so that I can extract annotations and review notes.

#### Acceptance Criteria

1. WHEN an xlsx file containing legacy comments is opened, THE Reader SHALL parse comment XML into Comment structs with author and text
2. WHEN a comment is associated with a specific cell, THE Reader SHALL map the comment to the correct cell coordinates
3. IF a comment XML file is missing or malformed, THEN THE Reader SHALL skip comments for that sheet without returning an error

### Requirement 8: Read Sparklines

**User Story:** As a developer, I want to read sparkline definitions from existing xlsx files, so that I can inspect or replicate sparkline configurations.

#### Acceptance Criteria

1. WHEN an xlsx file containing sparklines is opened, THE Reader SHALL parse sparkline groups from the extLst element in the sheet XML
2. WHEN a sparkline group is parsed, THE Reader SHALL populate Sparkline structs with the data range, location, and SparklineType (line, column, win/loss)

### Requirement 9: Read Defined Names with Scope

**User Story:** As a developer, I want to read defined names with their scope from existing xlsx files, so that I can distinguish workbook-scoped names from sheet-scoped names.

#### Acceptance Criteria

1. WHEN an xlsx file containing defined names is opened, THE Reader SHALL parse both workbook-scoped and sheet-scoped defined names
2. WHEN a defined name has a `localSheetId` attribute, THE Reader SHALL associate the name with the corresponding sheet index
3. THE Reader SHALL expose defined names through the Workbook API with scope information (workbook-level or sheet-level with sheet index)

### Requirement 10: Read Print Settings

**User Story:** As a developer, I want to read print settings from existing xlsx files, so that I can inspect or replicate page layout configurations.

#### Acceptance Criteria

1. WHEN an xlsx file is opened, THE Reader SHALL parse print settings including margins, orientation, paper size, headers, footers, and page breaks from the sheet XML
2. WHEN print settings are parsed, THE Reader SHALL populate a PrintSettings struct accessible via the Worksheet API
3. WHEN a sheet has repeat rows or columns defined, THE Reader SHALL parse the corresponding defined names and expose them through the Worksheet API

### Requirement 11: Read Sheet Protection

**User Story:** As a developer, I want to read sheet protection settings from existing xlsx files, so that I can inspect protection state and flags.

#### Acceptance Criteria

1. WHEN an xlsx file containing protected sheets is opened, THE Reader SHALL parse protection attributes from the sheet XML
2. WHEN a sheet has protection enabled, THE Reader SHALL populate a SheetProtection struct with the protection flags (sheet, objects, scenarios, password hash)

### Requirement 12: Read Row and Column Grouping

**User Story:** As a developer, I want to read row and column grouping (outline levels) from existing xlsx files, so that I can inspect the document structure.

#### Acceptance Criteria

1. WHEN an xlsx file containing grouped rows is opened, THE Reader SHALL parse outline level attributes from row elements
2. WHEN an xlsx file containing grouped columns is opened, THE Reader SHALL parse outline level attributes from col elements
3. THE Reader SHALL expose grouping information through the Worksheet API with row/column ranges and outline levels


### Requirement 13: Formula Parser

**User Story:** As a developer, I want the crate to parse Excel formula syntax into an AST, so that formulas can be evaluated, analyzed, and transformed programmatically.

#### Acceptance Criteria

1. WHEN a formula string is provided, THE FormulaEngine SHALL tokenize it into a sequence of tokens (cell references, operators, function calls, literals, parentheses)
2. THE FormulaEngine SHALL support A1-style references, R1C1-style references, and structured table references
3. THE FormulaEngine SHALL parse the token stream into an abstract syntax tree representing the formula's evaluation order
4. WHEN a formula contains syntax errors, THE FormulaEngine SHALL return a descriptive parse error indicating the position and nature of the error
5. THE FormulaEngine SHALL format parsed AST nodes back into valid Excel formula strings (round-trip property: parse then print then parse produces an equivalent AST)

### Requirement 14: Formula Dependency Graph

**User Story:** As a developer, I want the crate to build a dependency graph from cell formulas, so that cells can be recalculated in the correct order.

#### Acceptance Criteria

1. WHEN formulas are present in a Workbook, THE FormulaEngine SHALL build a directed acyclic graph of cell dependencies
2. WHEN the DependencyGraph is built, THE FormulaEngine SHALL produce a topological sort order for evaluation
3. WHEN a circular reference exists, THE FormulaEngine SHALL detect the cycle and report the involved cells
4. WHEN a cell value changes, THE FormulaEngine SHALL identify all transitively dependent cells that require recalculation

### Requirement 15: Core Formula Functions

**User Story:** As a developer, I want the crate to evaluate common Excel functions, so that I can generate computed reports without Excel.

#### Acceptance Criteria

1. THE FormulaEngine SHALL evaluate SUM, AVERAGE, COUNT, COUNTA, MIN, and MAX over cell ranges
2. THE FormulaEngine SHALL evaluate IF with two or three arguments, returning the correct branch value
3. THE FormulaEngine SHALL evaluate VLOOKUP with exact and approximate match modes
4. THE FormulaEngine SHALL evaluate INDEX and MATCH for single-cell lookups
5. WHEN a function receives arguments of incorrect type, THE FormulaEngine SHALL return a #VALUE! error

### Requirement 16: Math Formula Functions

**User Story:** As a developer, I want the crate to evaluate mathematical Excel functions, so that numeric computations produce correct results.

#### Acceptance Criteria

1. THE FormulaEngine SHALL evaluate ROUND, ROUNDUP, and ROUNDDOWN to the specified number of decimal places
2. THE FormulaEngine SHALL evaluate ABS, MOD, POWER, SQRT, LOG, LN, EXP, and PI
3. WHEN a math function receives an invalid argument (negative SQRT, zero LOG), THE FormulaEngine SHALL return a #NUM! error

### Requirement 17: Text Formula Functions

**User Story:** As a developer, I want the crate to evaluate text manipulation functions, so that string processing works without Excel.

#### Acceptance Criteria

1. THE FormulaEngine SHALL evaluate CONCATENATE (and CONCAT), LEFT, RIGHT, MID, LEN, TRIM, UPPER, LOWER, and SUBSTITUTE
2. WHEN text functions receive numeric arguments, THE FormulaEngine SHALL coerce numbers to strings following Excel's coercion rules
3. WHEN MID or LEFT receives a count exceeding the string length, THE FormulaEngine SHALL return the available characters without error

### Requirement 18: Date Formula Functions

**User Story:** As a developer, I want the crate to evaluate date functions, so that date arithmetic works in generated reports.

#### Acceptance Criteria

1. THE FormulaEngine SHALL evaluate TODAY, NOW, DATE, YEAR, MONTH, DAY, EDATE, EOMONTH, and NETWORKDAYS
2. WHEN TODAY or NOW is evaluated, THE FormulaEngine SHALL use the current system date and time
3. WHEN DATE receives out-of-range month or day values, THE FormulaEngine SHALL roll over correctly (e.g., month 13 becomes January of the next year)

### Requirement 19: Lookup Formula Functions

**User Story:** As a developer, I want the crate to evaluate lookup functions, so that data retrieval formulas produce correct results.

#### Acceptance Criteria

1. THE FormulaEngine SHALL evaluate HLOOKUP with exact and approximate match modes
2. THE FormulaEngine SHALL evaluate XLOOKUP with match mode, search mode, and if-not-found arguments
3. THE FormulaEngine SHALL evaluate MATCH returning the relative position within a range
4. THE FormulaEngine SHALL evaluate INDIRECT, resolving string references to cell values
5. THE FormulaEngine SHALL evaluate OFFSET, returning a reference shifted by the specified rows and columns

### Requirement 20: Statistical Formula Functions

**User Story:** As a developer, I want the crate to evaluate statistical functions, so that data analysis formulas produce correct results.

#### Acceptance Criteria

1. THE FormulaEngine SHALL evaluate STDEV, VAR, MEDIAN, PERCENTILE, and RANK
2. THE FormulaEngine SHALL evaluate COUNTIF and SUMIF with criteria strings supporting operators (">", "<", ">=", "<=", "<>", "=") and wildcards (* and ?)
3. WHEN a statistical function receives fewer data points than required (e.g., STDEV with one value), THE FormulaEngine SHALL return a #DIV/0! error

### Requirement 21: Logical Formula Functions

**User Story:** As a developer, I want the crate to evaluate logical functions, so that conditional logic in formulas works correctly.

#### Acceptance Criteria

1. THE FormulaEngine SHALL evaluate AND, OR, and NOT with correct short-circuit semantics
2. THE FormulaEngine SHALL evaluate IFERROR and IFNA, returning the alternate value when the first argument produces the corresponding error
3. THE FormulaEngine SHALL evaluate SWITCH and IFS, matching the first true condition

### Requirement 22: Array Formula Evaluation

**User Story:** As a developer, I want the crate to evaluate array formulas, so that CSE and dynamic array spill formulas produce correct multi-cell results.

#### Acceptance Criteria

1. WHEN a CSE array formula spans a defined range, THE FormulaEngine SHALL evaluate the formula and populate all cells in the range
2. WHEN a dynamic array formula is evaluated, THE FormulaEngine SHALL compute the spill range and populate all result cells
3. WHEN a spill range overlaps with existing non-empty cells, THE FormulaEngine SHALL return a #SPILL! error

### Requirement 23: Circular Reference Detection

**User Story:** As a developer, I want the crate to detect circular references, so that infinite evaluation loops are prevented.

#### Acceptance Criteria

1. WHEN the DependencyGraph contains a cycle, THE FormulaEngine SHALL detect the cycle before evaluation begins
2. WHEN a circular reference is detected, THE FormulaEngine SHALL return an error listing the cells involved in the cycle

### Requirement 24: Volatile Function Handling

**User Story:** As a developer, I want the crate to handle volatile functions correctly, so that functions like RAND and NOW recalculate appropriately.

#### Acceptance Criteria

1. THE FormulaEngine SHALL mark RAND, RANDBETWEEN, NOW, TODAY, and INDIRECT as volatile functions
2. WHEN recalculation is triggered, THE FormulaEngine SHALL re-evaluate all volatile functions and their dependents


### Requirement 25: Bubble Charts

**User Story:** As a developer, I want to create bubble charts, so that I can visualize three-dimensional data (X, Y, size).

#### Acceptance Criteria

1. WHEN a Chart is created with ChartType::Bubble, THE ChartEngine SHALL serialize valid bubble chart XML with x-values, y-values, and bubble-size series
2. WHEN a bubble chart is opened in Excel, THE Writer SHALL produce zero repair errors

### Requirement 26: Waterfall Charts (ChartEx)

**User Story:** As a developer, I want to create waterfall charts, so that I can visualize incremental positive and negative contributions to a total.

#### Acceptance Criteria

1. WHEN a waterfall chart is created, THE ChartEngine SHALL serialize valid ChartEx waterfall XML
2. THE ChartEngine SHALL support marking individual data points as total, increase, or decrease categories

### Requirement 27: Funnel Charts (ChartEx)

**User Story:** As a developer, I want to create funnel charts, so that I can visualize progressive reduction across stages.

#### Acceptance Criteria

1. WHEN a funnel chart is created, THE ChartEngine SHALL serialize valid ChartEx funnel XML
2. WHEN a funnel chart is opened in Excel 2016 or later, THE Writer SHALL produce zero repair errors

### Requirement 28: Sunburst Charts (ChartEx)

**User Story:** As a developer, I want to create sunburst charts, so that I can visualize hierarchical data as concentric rings.

#### Acceptance Criteria

1. WHEN a sunburst chart is created, THE ChartEngine SHALL serialize valid ChartEx sunburst XML with multi-level category data
2. THE ChartEngine SHALL support at least 3 levels of hierarchy in sunburst charts

### Requirement 29: Histogram and Pareto Charts (ChartEx)

**User Story:** As a developer, I want to create histogram and Pareto charts, so that I can visualize frequency distributions.

#### Acceptance Criteria

1. WHEN a histogram chart is created, THE ChartEngine SHALL serialize valid ChartEx histogram XML
2. WHEN a Pareto chart is created, THE ChartEngine SHALL include both the histogram bars and the cumulative percentage line
3. THE ChartEngine SHALL support configurable bin count or bin width for histograms

### Requirement 30: Box and Whisker Charts (ChartEx)

**User Story:** As a developer, I want to create box and whisker charts, so that I can visualize statistical distributions.

#### Acceptance Criteria

1. WHEN a box and whisker chart is created, THE ChartEngine SHALL serialize valid ChartEx box and whisker XML
2. THE ChartEngine SHALL support showing outlier points, mean markers, and inner points

### Requirement 31: Map Charts

**User Story:** As a developer, I want to create map charts, so that I can visualize geographic data.

#### Acceptance Criteria

1. WHEN a map chart is created, THE ChartEngine SHALL serialize valid chart XML with geographic data bindings
2. THE ChartEngine SHALL support region-level and country-level geographic granularity

### Requirement 32: 3D Chart Variants

**User Story:** As a developer, I want to create 3D chart variants, so that I can produce visually rich presentations.

#### Acceptance Criteria

1. THE ChartEngine SHALL support 3D variants for Column, Bar, Line, Pie, and Area chart types
2. WHEN a 3D chart is created, THE ChartEngine SHALL serialize the view3D element with rotation, elevation, and perspective attributes

### Requirement 33: Surface Charts

**User Story:** As a developer, I want to create surface charts, so that I can visualize 3D data surfaces.

#### Acceptance Criteria

1. THE ChartEngine SHALL support surface and wireframe surface chart types
2. WHEN a surface chart is created, THE ChartEngine SHALL serialize valid surface chart XML with 3D view settings

### Requirement 34: Chart Style Themes

**User Story:** As a developer, I want to apply built-in chart styles, so that charts match Excel's predefined visual themes.

#### Acceptance Criteria

1. WHEN a chart style number (1–48) is set, THE ChartEngine SHALL apply the corresponding color palette and formatting
2. THE ChartEngine SHALL serialize the style element in the chart XML

### Requirement 35: Axis Formatting

**User Story:** As a developer, I want to format chart axes, so that I can control number formats, fonts, tick marks, and gridline styles.

#### Acceptance Criteria

1. THE ChartEngine SHALL support setting number format, font, and font size on category and value axes
2. THE ChartEngine SHALL support configuring major and minor tick mark positions (inside, outside, cross, none)
3. THE ChartEngine SHALL support configuring major and minor gridline styles (color, width, dash style)

### Requirement 36: Plot Area and Series Formatting

**User Story:** As a developer, I want to format chart plot areas and individual series, so that I can customize chart appearance.

#### Acceptance Criteria

1. THE ChartEngine SHALL support setting fill color, border color, and gradient fills on the plot area
2. THE ChartEngine SHALL support setting line width, dash style, fill patterns, and gradient fills on individual series
3. THE ChartEngine SHALL support error bars with standard error, percentage, fixed value, and custom value modes

### Requirement 37: Chart Accessories

**User Story:** As a developer, I want to add drop lines, high-low lines, and chart sheets, so that I can create complete chart presentations.

#### Acceptance Criteria

1. THE ChartEngine SHALL support drop lines and high-low lines on line chart types
2. WHEN a chart is designated as a chart sheet, THE Writer SHALL create a dedicated chartsheet XML part instead of embedding the chart in a worksheet drawing

### Requirement 38: Gradient Fills

**User Story:** As a developer, I want to apply gradient fills to cells, so that I can create visually rich spreadsheets.

#### Acceptance Criteria

1. THE Format SHALL support two-color linear gradient fills with configurable angle
2. THE Format SHALL support multi-stop gradient fills with position and color for each stop
3. WHEN a gradient fill is applied, THE Writer SHALL serialize the gradient element in the styles XML

### Requirement 39: Theme Colors

**User Story:** As a developer, I want to reference theme colors, so that spreadsheets adapt to the workbook's color theme.

#### Acceptance Criteria

1. THE Format SHALL support referencing theme color indices (accent1–accent6, dk1, dk2, lt1, lt2)
2. WHEN a theme color is used, THE Writer SHALL serialize the theme index and optional tint/shade value in the styles XML
3. WHEN reading a file, THE Reader SHALL resolve theme color references to RGB values using the theme XML

### Requirement 40: Gradient Data Bars

**User Story:** As a developer, I want to create gradient-fill data bars in conditional formatting, so that data bars have a modern appearance.

#### Acceptance Criteria

1. THE ConditionalFormatDataBar SHALL support a gradient fill mode in addition to the existing solid fill mode
2. WHEN gradient mode is enabled, THE Writer SHALL serialize the gradient attribute in the data bar CF rule

### Requirement 41: Cell Styles

**User Story:** As a developer, I want to apply named cell styles, so that I can use Excel's built-in styles like Normal, Heading 1, and Currency.

#### Acceptance Criteria

1. THE Format SHALL support referencing named cell styles by name (Normal, Heading 1, Heading 2, Currency, Percent, etc.)
2. WHEN a named style is applied, THE Writer SHALL reference the corresponding cellStyleXfs entry in the styles XML

### Requirement 42: Custom Table Styles

**User Story:** As a developer, I want to create custom table styles, so that tables can have unique visual designs beyond the built-in styles.

#### Acceptance Criteria

1. THE Table SHALL support defining custom table styles with stripe sizes, header formatting, and total row formatting
2. WHEN a custom table style is applied, THE Writer SHALL serialize the tableStyle element in the styles XML

### Requirement 43: Phonetic Text (Furigana)

**User Story:** As a developer, I want to add phonetic guide text to cells, so that East Asian text displays pronunciation annotations.

#### Acceptance Criteria

1. THE Worksheet SHALL support adding phonetic runs (rPh elements) to cells containing East Asian text
2. WHEN phonetic text is present, THE Writer SHALL serialize the phoneticPr and rPh elements in the sheet XML

### Requirement 44: Text Effects

**User Story:** As a developer, I want to apply text effects to fonts, so that I can create shadow, outline, emboss, and engrave effects.

#### Acceptance Criteria

1. THE Format SHALL support shadow, outline, emboss, and engrave text effects on font definitions
2. WHEN a text effect is applied, THE Writer SHALL serialize the corresponding font element attributes in the styles XML


### Requirement 45: Slicers

**User Story:** As a developer, I want to add slicers to tables and pivot tables, so that users can visually filter data.

#### Acceptance Criteria

1. WHEN a slicer is added to a table, THE Writer SHALL serialize the slicer XML part, slicer cache, and drawing relationship
2. WHEN a slicer is added to a pivot table, THE Writer SHALL link the slicer cache to the pivot cache
3. THE Reader SHALL parse slicer definitions and their associated caches when reading xlsx files

### Requirement 46: Timelines

**User Story:** As a developer, I want to add timelines to pivot tables, so that users can filter data by date ranges visually.

#### Acceptance Criteria

1. WHEN a timeline is added to a pivot table, THE Writer SHALL serialize the timeline XML part and timeline cache
2. THE Reader SHALL parse timeline definitions when reading xlsx files

### Requirement 47: Named Ranges CRUD

**User Story:** As a developer, I want to create, update, and delete named ranges programmatically, so that I can manage named references in workbooks.

#### Acceptance Criteria

1. THE Workbook SHALL support creating named ranges with a name, reference formula, and optional sheet scope
2. THE Workbook SHALL support updating the reference formula of an existing named range
3. THE Workbook SHALL support deleting a named range by name and scope
4. WHEN a named range is modified, THE Writer SHALL serialize the updated definedNames element in the workbook XML

### Requirement 48: External Data Connections

**User Story:** As a developer, I want to preserve external data connections, so that query tables and ODBC/OLEDB connections survive round-trip editing.

#### Acceptance Criteria

1. WHEN an xlsx file containing external data connections is opened in edit mode, THE Workbook SHALL preserve the connections XML part during save
2. THE Workbook SHALL support creating new query table connections with connection string and command text

### Requirement 49: Power Query Preservation

**User Story:** As a developer, I want to preserve Power Query definitions, so that M-language queries survive round-trip editing.

#### Acceptance Criteria

1. WHEN an xlsx file containing Power Query definitions is opened in edit mode, THE Workbook SHALL preserve the customXml parts containing Power Query metadata during save

### Requirement 50: Pivot Table Grouping

**User Story:** As a developer, I want to group pivot table fields by date or numeric ranges, so that pivot tables can summarize data at different granularities.

#### Acceptance Criteria

1. THE PivotTable SHALL support date grouping by year, quarter, month, day, hour, minute, and second
2. THE PivotTable SHALL support numeric range grouping with configurable start, end, and interval values
3. WHEN grouping is applied, THE Writer SHALL serialize the fieldGroup and rangePr elements in the pivot cache definition

### Requirement 51: Pivot Table Calculated Items

**User Story:** As a developer, I want to add calculated items to pivot tables, so that custom computations can appear alongside data items.

#### Acceptance Criteria

1. THE PivotTable SHALL support defining calculated items with a name and formula
2. WHEN a calculated item is added, THE Writer SHALL serialize the calculatedItem element in the pivot table definition

### Requirement 52: Sort State Preservation

**User Story:** As a developer, I want to preserve and set sort order on columns, so that sorted data maintains its order after editing.

#### Acceptance Criteria

1. WHEN an xlsx file with sorted columns is opened in edit mode, THE Workbook SHALL preserve the sortState element during save
2. THE Worksheet SHALL support setting sort order on one or more columns with ascending or descending direction

### Requirement 53: Advanced Autofilter

**User Story:** As a developer, I want to create advanced autofilter rules, so that I can apply top-10, date, and color filters.

#### Acceptance Criteria

1. THE Worksheet SHALL support top-10 filters (top N items, top N percent, bottom N items, bottom N percent)
2. THE Worksheet SHALL support date filters (year, month, quarter, date range)
3. THE Worksheet SHALL support custom filters with two criteria combined by AND or OR operators

### Requirement 54: Streaming Read

**User Story:** As a developer, I want to read large xlsx files row-by-row with constant memory, so that I can process files that exceed available RAM.

#### Acceptance Criteria

1. THE StreamingReader SHALL provide a SAX-style iterator that yields one row at a time from the sheet XML
2. WHILE iterating rows, THE StreamingReader SHALL maintain constant memory usage regardless of file size
3. WHEN the StreamingReader encounters a shared string reference, THE StreamingReader SHALL resolve it using the SST
4. THE StreamingReader SHALL support reading cell values, formulas, and xf indices for each cell in a row

### Requirement 55: Streaming Write with Charts and Images

**User Story:** As a developer, I want to include charts and images in streaming write mode, so that large files can contain visual elements.

#### Acceptance Criteria

1. THE StreamingWriter SHALL support inserting charts at specified cell positions during streaming write
2. THE StreamingWriter SHALL support inserting images at specified cell positions during streaming write
3. WHEN charts or images are added in streaming mode, THE Writer SHALL serialize the drawing relationships and parts correctly

### Requirement 56: Streaming Write with Conditional Formatting

**User Story:** As a developer, I want to apply conditional formatting in streaming write mode, so that large files can contain CF rules.

#### Acceptance Criteria

1. THE StreamingWriter SHALL support adding conditional formatting rules to cell ranges during streaming write
2. WHEN CF rules are added in streaming mode, THE Writer SHALL serialize the conditionalFormatting elements in the sheet XML

### Requirement 57: Parallel Sheet Writing

**User Story:** As a developer, I want to write multiple sheets concurrently, so that multi-sheet workbooks generate faster.

#### Acceptance Criteria

1. THE Writer SHALL support writing multiple sheets in parallel using separate threads
2. WHEN parallel writing is used, THE Writer SHALL produce output identical to sequential writing
3. THE Writer SHALL ensure thread-safe access to shared resources (SST, style registry) during parallel writing

### Requirement 58: Memory-Mapped Reading

**User Story:** As a developer, I want to use memory-mapped I/O for reading large files, so that random access to sheet data is efficient.

#### Acceptance Criteria

1. THE Reader SHALL support opening xlsx files using memory-mapped I/O via the `mmap` system call
2. WHEN memory-mapped reading is used, THE Reader SHALL avoid loading the entire file into heap memory

### Requirement 59: Incremental Save

**User Story:** As a developer, I want to save only modified sheets in edit mode, so that saving large workbooks with small changes is fast.

#### Acceptance Criteria

1. WHEN a Workbook is saved in edit mode, THE Writer SHALL only rewrite sheets that have been modified (dirty flag)
2. WHEN unmodified sheets exist, THE Writer SHALL copy their original ZIP entries without re-serialization

### Requirement 60: Shared String Deduplication Tuning

**User Story:** As a developer, I want to configure the threshold for inline vs shared strings, so that I can optimize file size and write speed.

#### Acceptance Criteria

1. THE Workbook SHALL support a configuration option to set the shared string deduplication threshold
2. WHEN a string appears fewer times than the threshold, THE Writer SHALL inline the string instead of adding it to the SST

### Requirement 61: XLSM Write

**User Story:** As a developer, I want to save workbooks as macro-enabled XLSM files, so that I can create new workbooks with VBA macros.

#### Acceptance Criteria

1. THE Workbook SHALL support saving as XLSM format with a VBA project binary part
2. WHEN saving as XLSM, THE Writer SHALL set the correct content types for macro-enabled workbooks
3. IF no VBA project is provided, THEN THE Writer SHALL return an error when attempting to save as XLSM

### Requirement 62: Template Formats (XLTX/XLTM)

**User Story:** As a developer, I want to save workbooks as Excel template files, so that I can create reusable spreadsheet templates.

#### Acceptance Criteria

1. THE Workbook SHALL support saving as XLTX (template) and XLTM (macro-enabled template) formats
2. WHEN saving as a template format, THE Writer SHALL set the correct content types and relationships

### Requirement 63: XLSB Read

**User Story:** As a developer, I want to read XLSB (binary) Excel files, so that I can process files in the binary format.

#### Acceptance Criteria

1. THE Reader SHALL detect XLSB format by inspecting the ZIP content types
2. THE Reader SHALL parse the binary record stream to extract cell values, formulas, and sheet metadata
3. IF an XLSB file uses features not supported by the Reader, THEN THE Reader SHALL skip unsupported records without returning an error

### Requirement 64: XLS Read (BIFF8)

**User Story:** As a developer, I want to read legacy XLS files, so that I can process files in the older binary format.

#### Acceptance Criteria

1. THE Reader SHALL detect XLS format by inspecting the OLE2 compound file header
2. THE Reader SHALL parse BIFF8 records to extract cell values, formulas, and sheet metadata
3. IF an XLS file uses features not supported by the Reader, THEN THE Reader SHALL skip unsupported records without returning an error

### Requirement 65: CSV/TSV Export

**User Story:** As a developer, I want to export sheets to CSV or TSV format, so that I can produce delimited text output.

#### Acceptance Criteria

1. THE Worksheet SHALL support exporting cell data to CSV format with configurable delimiter, quote character, and line ending
2. WHEN exporting to CSV, THE Worksheet SHALL format date cells using ISO 8601 format by default
3. WHEN exporting to CSV, THE Worksheet SHALL escape fields containing the delimiter, quote character, or newlines

### Requirement 66: ODS Read/Write

**User Story:** As a developer, I want to read and write ODS files, so that I can interoperate with LibreOffice and OpenOffice.

#### Acceptance Criteria

1. THE Reader SHALL support reading ODS files, extracting cell values, formulas, and basic formatting
2. THE Writer SHALL support saving workbooks in ODS format with cell values, formulas, and basic formatting
3. IF an ODS file uses features not supported by the crate, THEN THE Reader SHALL skip unsupported elements without returning an error

### Requirement 67: Strict OOXML

**User Story:** As a developer, I want to read and write Strict OOXML files, so that I can handle ISO 29500 strict-namespace documents.

#### Acceptance Criteria

1. THE Reader SHALL detect Strict OOXML namespace URIs and map them to the corresponding Transitional namespace handlers
2. THE Writer SHALL support saving in Strict OOXML mode with ISO 29500 strict namespace URIs


### Requirement 68: Threaded Comments

**User Story:** As a developer, I want to create and read threaded comments, so that I can use the modern Excel comment model with replies.

#### Acceptance Criteria

1. THE Worksheet SHALL support adding threaded comments with author, timestamp, and reply chain
2. THE Writer SHALL serialize threaded comments using the ThreadedComments XML part
3. THE Reader SHALL parse threaded comments and their reply chains from xlsx files

### Requirement 69: Form Controls

**User Story:** As a developer, I want to add form controls to worksheets, so that I can create interactive spreadsheets with checkboxes, dropdowns, buttons, and spinners.

#### Acceptance Criteria

1. THE Worksheet SHALL support adding checkbox, dropdown, button, and spinner form controls
2. WHEN a form control is linked to a cell, THE Writer SHALL serialize the cell link relationship
3. THE Reader SHALL parse form control definitions when reading xlsx files

### Requirement 70: ActiveX Controls

**User Story:** As a developer, I want to preserve ActiveX controls during round-trip editing, so that embedded ActiveX objects survive file modification.

#### Acceptance Criteria

1. WHEN an xlsx file containing ActiveX controls is opened in edit mode, THE Workbook SHALL preserve the ActiveX binary parts and relationships during save

### Requirement 71: OLE Objects

**User Story:** As a developer, I want to preserve OLE embedded objects during round-trip editing, so that embedded documents survive file modification.

#### Acceptance Criteria

1. WHEN an xlsx file containing OLE objects is opened in edit mode, THE Workbook SHALL preserve the OLE binary parts and relationships during save

### Requirement 72: Drawing Shapes

**User Story:** As a developer, I want to add drawing shapes to worksheets, so that I can create rectangles, arrows, callouts, and text boxes.

#### Acceptance Criteria

1. THE Worksheet SHALL support adding rectangle, rounded rectangle, arrow, callout, and text box shapes
2. THE Writer SHALL serialize shapes as SpAutoShape elements in the drawing XML with position, size, fill, and outline properties
3. WHEN a shape contains text, THE Writer SHALL serialize the text body with font and paragraph formatting

### Requirement 73: SmartArt Preservation

**User Story:** As a developer, I want to preserve SmartArt diagrams during round-trip editing, so that SmartArt survives file modification.

#### Acceptance Criteria

1. WHEN an xlsx file containing SmartArt is opened in edit mode, THE Workbook SHALL preserve the SmartArt diagram XML parts and relationships during save

### Requirement 74: Equation Object Preservation

**User Story:** As a developer, I want to preserve equation objects during round-trip editing, so that OMML equations survive file modification.

#### Acceptance Criteria

1. WHEN an xlsx file containing OMML equations is opened in edit mode, THE Workbook SHALL preserve the equation XML elements during save

### Requirement 75: Digital Signatures

**User Story:** As a developer, I want to sign and verify workbook digital signatures, so that document authenticity can be established.

#### Acceptance Criteria

1. THE Workbook SHALL support signing a workbook with an X.509 certificate
2. THE Workbook SHALL support verifying existing digital signatures and reporting validity status
3. WHEN a signed workbook is modified, THE Workbook SHALL invalidate the existing signature

### Requirement 76: Custom XML Parts

**User Story:** As a developer, I want to add and read custom XML parts, so that I can store application-specific data in workbooks.

#### Acceptance Criteria

1. THE Workbook SHALL support adding custom XML parts with a namespace URI and XML content
2. THE Workbook SHALL support reading custom XML parts by namespace URI
3. WHEN custom XML parts exist, THE Writer SHALL serialize them in the customXml directory with correct content type entries

### Requirement 77: Metadata and Custom Properties

**User Story:** As a developer, I want to set and read custom document properties, so that I can store application-specific metadata.

#### Acceptance Criteria

1. THE Workbook SHALL support setting custom document properties with name, type (string, number, date, boolean), and value
2. THE Reader SHALL parse custom properties from the `docProps/custom.xml` part
3. THE Writer SHALL serialize custom properties in the `docProps/custom.xml` part

### Requirement 78: File Encryption

**User Story:** As a developer, I want to read and write password-encrypted xlsx files, so that I can handle enterprise files protected with Standard or Agile encryption.

#### Acceptance Criteria

1. WHEN a password is provided, THE Reader SHALL decrypt xlsx files encrypted with ECMA-376 Standard Encryption
2. WHEN a password is provided, THE Reader SHALL decrypt xlsx files encrypted with ECMA-376 Agile Encryption
3. THE Writer SHALL support encrypting xlsx files with Agile Encryption using a user-provided password
4. IF an incorrect password is provided, THEN THE Reader SHALL return a descriptive authentication error

### Requirement 79: Sheet Protection Granularity

**User Story:** As a developer, I want to set per-feature protection flags on sheets, so that I can allow specific operations (sort, filter, pivot) while protecting others.

#### Acceptance Criteria

1. THE SheetProtection SHALL support individual flags for: sort, autoFilter, pivotTables, insertColumns, insertRows, deleteColumns, deleteRows, formatCells, formatColumns, formatRows, insertHyperlinks, selectLockedCells, selectUnlockedCells
2. WHEN protection flags are set, THE Writer SHALL serialize each flag as an attribute on the sheetProtection element

### Requirement 80: VBA Project Signing

**User Story:** As a developer, I want to sign VBA projects, so that macro code can be trusted by Excel's security model.

#### Acceptance Criteria

1. THE Workbook SHALL support signing a VBA project binary part with an X.509 certificate
2. WHEN a VBA project is signed, THE Writer SHALL serialize the digital signature in the vbaProjectSignature part

### Requirement 81: IRM Metadata Preservation

**User Story:** As a developer, I want to preserve IRM metadata during round-trip editing, so that rights-managed files survive modification.

#### Acceptance Criteria

1. WHEN an xlsx file containing IRM metadata is opened in edit mode, THE Workbook SHALL preserve the IRM-related parts during save

### Requirement 82: Accessibility Metadata

**User Story:** As a developer, I want to set alt text on charts, images, and tables, so that spreadsheets meet accessibility requirements.

#### Acceptance Criteria

1. THE Chart SHALL support setting alt text (title and description) for screen readers
2. THE Image SHALL support setting alt text (title and description) for screen readers
3. THE Table SHALL support setting alt text (title and description) for screen readers
4. WHEN alt text is set, THE Writer SHALL serialize the cNvPr descr and title attributes in the drawing XML

### Requirement 83: Serde Integration

**User Story:** As a developer, I want to serialize and deserialize Rust structs to and from Excel rows, so that I can use typed data access with the serde ecosystem.

#### Acceptance Criteria

1. THE Workbook SHALL support writing a Vec of serde-serializable structs as rows with automatic header generation
2. THE Workbook SHALL support reading rows into a Vec of serde-deserializable structs using header-based field mapping
3. WHEN a struct field type does not match the cell value type, THE Workbook SHALL return a descriptive deserialization error
4. FOR ALL serializable structs, serializing then deserializing SHALL produce an equivalent struct (round-trip property)

### Requirement 84: Derive Macro for Typed Row Mapping

**User Story:** As a developer, I want a `#[derive(ExcelRow)]` proc macro, so that I can map struct fields to Excel columns with minimal boilerplate.

#### Acceptance Criteria

1. THE derive macro SHALL generate serialization and deserialization implementations for annotated structs
2. THE derive macro SHALL support `#[excel(header = "...")]` attributes to customize column header names
3. THE derive macro SHALL support `#[excel(format = "...")]` attributes to set number formats on columns
4. WHEN a struct field is an Option type, THE derive macro SHALL handle empty cells by producing None

### Requirement 85: Async I/O

**User Story:** As a developer, I want to read and write xlsx files using async I/O, so that file operations integrate with async runtimes.

#### Acceptance Criteria

1. THE Workbook SHALL support async save and open operations compatible with tokio and async-std runtimes
2. WHEN async I/O is used, THE Workbook SHALL perform file reads and writes on the async runtime's I/O threads
3. THE async API SHALL be gated behind an optional feature flag to avoid adding async runtime dependencies by default

### Requirement 86: WASM Target

**User Story:** As a developer, I want to compile zavora-xlsx to WebAssembly, so that I can generate xlsx files in the browser.

#### Acceptance Criteria

1. THE crate SHALL compile to the `wasm32-unknown-unknown` target without errors
2. WHEN compiled to WASM, THE Workbook SHALL support save_to_buffer returning a byte array usable from JavaScript
3. THE WASM build SHALL exclude file system operations and expose only buffer-based APIs

### Requirement 87: Python Bindings (PyO3)

**User Story:** As a developer, I want to use zavora-xlsx from Python, so that I can generate xlsx files in Python applications with Rust performance.

#### Acceptance Criteria

1. THE Python binding SHALL expose Workbook, Worksheet, Format, Chart, and Table classes
2. THE Python binding SHALL support writing cells, applying formats, adding charts, and saving to file or bytes
3. THE Python binding SHALL be installable via `pip install zavora-xlsx`

### Requirement 88: Node.js Bindings (napi-rs)

**User Story:** As a developer, I want to use zavora-xlsx from Node.js, so that I can generate xlsx files in JavaScript applications with Rust performance.

#### Acceptance Criteria

1. THE Node.js binding SHALL expose Workbook, Worksheet, Format, Chart, and Table classes
2. THE Node.js binding SHALL support writing cells, applying formats, adding charts, and saving to file or Buffer
3. THE Node.js binding SHALL be installable via `npm install zavora-xlsx`

### Requirement 89: C FFI

**User Story:** As a developer, I want a C-compatible API for zavora-xlsx, so that I can use the crate from any language with C FFI support.

#### Acceptance Criteria

1. THE C FFI SHALL expose functions for creating workbooks, worksheets, writing cells, applying formats, and saving
2. THE C FFI SHALL use opaque pointer handles for all struct types
3. THE C FFI SHALL return error codes and provide a function to retrieve the last error message

### Requirement 90: CLI Tool

**User Story:** As a developer, I want a command-line tool for xlsx inspection and conversion, so that I can work with xlsx files from the terminal.

#### Acceptance Criteria

1. THE CLI tool SHALL support inspecting xlsx file metadata (sheet names, row counts, column counts)
2. THE CLI tool SHALL support exporting sheets to CSV format
3. THE CLI tool SHALL support converting between xlsx, xlsm, and csv formats
4. WHEN an error occurs, THE CLI tool SHALL print a descriptive error message to stderr and exit with a non-zero status code
