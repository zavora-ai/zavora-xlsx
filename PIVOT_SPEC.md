# Pivot Table Spec: 100% Functionality

## Goal
Full pivot table write support in zavora-xlsx — the first Rust crate to achieve this. No other Rust Excel crate (rust_xlsxwriter, umya-spreadsheet, calamine) can create pivot tables from scratch.

---

## Architecture

A pivot table requires **5 XML parts** + **4 relationship entries** + **3 content type overrides**:

```
xl/pivotTables/pivotTable1.xml          ← The pivot table definition
xl/pivotTables/_rels/pivotTable1.xml.rels  ← Links to cache definition
xl/pivotCache/pivotCacheDefinition1.xml ← Cache metadata (fields, source range)
xl/pivotCache/pivotCacheRecords1.xml    ← Cached data values
xl/pivotCache/_rels/pivotCacheDefinition1.xml.rels ← Links to cache records
```

**Relationships:**
- `xl/_rels/workbook.xml.rels` → pivotCacheDefinition (workbook-level)
- `xl/worksheets/_rels/sheetN.xml.rels` → pivotTable (sheet-level)
- `xl/pivotTables/_rels/pivotTable1.xml.rels` → pivotCacheDefinition
- `xl/pivotCache/_rels/pivotCacheDefinition1.xml.rels` → pivotCacheRecords

**Content Types:**
- `application/vnd.openxmlformats-officedocument.spreadsheetml.pivotTable+xml`
- `application/vnd.openxmlformats-officedocument.spreadsheetml.pivotCacheDefinition+xml`
- `application/vnd.openxmlformats-officedocument.spreadsheetml.pivotCacheRecords+xml`

---

## API Design

```rust
use zavora_xlsx::*;

let mut wb = Workbook::new();

// Source data on Sheet1
let ws = wb.worksheet(0)?;
ws.set_name("Sales")?;
ws.write_row(0, 0, ["Region", "Product", "Quarter", "Revenue", "Units"])?;
ws.write_row(1, 0, ["East", "Widget", "Q1", 10000.0, 150.0])?;
// ... more data rows

// Create pivot table on Sheet2
let ws2 = wb.add_worksheet_with_name("Analysis")?;

let pivot = PivotTable::new("SalesPivot", "Sales!$A$1:$E$100")
    // Row fields — categories shown as rows
    .add_row_field("Region")
    .add_row_field("Product")
    // Column field — categories shown as columns
    .add_column_field("Quarter")
    // Value fields — aggregated data
    .add_value_field("Revenue", PivotAggregation::Sum)
    .add_value_field("Units", PivotAggregation::Sum)
    .add_value_field("Revenue", PivotAggregation::Average)
    // Filter field — page filter
    .add_filter_field("Region")
    // Calculated field
    .add_calculated_field("Avg Price", "Revenue/Units")
    // Styling
    .set_style(PivotStyle::Light16)
    .show_row_headers(true)
    .show_column_headers(true)
    .show_row_stripes(true)
    .show_grand_totals(true, true)  // row totals, column totals
    // Sorting
    .sort_field("Revenue", PivotSort::Descending)
    // Number format on value field
    .set_value_format("Revenue", "$#,##0")
    // Subtotals
    .show_subtotals("Region", true)
    // Compact vs tabular layout
    .set_layout(PivotLayout::Tabular);

ws2.add_pivot_table(0, 0, &pivot)?;

wb.save("analysis.xlsx")?;
```

---

## Data Structures

### PivotTable (user-facing)
```rust
pub struct PivotTable {
    name: String,
    source_range: String,           // "Sheet1!$A$1:$E$100"
    source_sheet: String,           // extracted from source_range
    row_fields: Vec<String>,        // field names for rows
    column_fields: Vec<String>,     // field names for columns
    value_fields: Vec<PivotValueField>,
    filter_fields: Vec<String>,     // page filter fields
    calculated_fields: Vec<(String, String)>, // (name, formula)
    style: PivotStyle,
    show_row_headers: bool,
    show_col_headers: bool,
    show_row_stripes: bool,
    show_col_stripes: bool,
    show_row_grand_total: bool,
    show_col_grand_total: bool,
    layout: PivotLayout,
    field_sorts: Vec<(String, PivotSort)>,
    field_formats: Vec<(String, String)>,    // (field_name, num_format)
    field_subtotals: Vec<(String, bool)>,
}

pub struct PivotValueField {
    source_field: String,           // column name in source data
    name: String,                   // display name (e.g., "Sum of Revenue")
    aggregation: PivotAggregation,
}

pub enum PivotAggregation {
    Sum, Count, Average, Max, Min, Product, CountNums,
    StdDev, StdDevP, Var, VarP,
}

pub enum PivotStyle {
    None,
    Light1, Light2, /* ... */ Light28,
    Medium1, Medium2, /* ... */ Medium28,
    Dark1, Dark2, /* ... */ Dark28,
}

pub enum PivotLayout { Compact, Outline, Tabular }
pub enum PivotSort { Ascending, Descending }
```

---

## XML Generation

### 1. pivotCacheDefinition1.xml
Describes the source data structure.

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<pivotCacheDefinition xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
    xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
    r:id="rId1" refreshedBy="zavora-xlsx" refreshedDate="45990"
    createdVersion="8" refreshedVersion="8" minRefreshableVersion="3"
    recordCount="100">
  <cacheSource type="worksheet">
    <worksheetSource ref="A1:E101" sheet="Sales"/>
  </cacheSource>
  <cacheFields count="5">
    <cacheField name="Region" numFmtId="0">
      <sharedItems count="4">
        <s v="East"/>
        <s v="West"/>
        <s v="North"/>
        <s v="South"/>
      </sharedItems>
    </cacheField>
    <cacheField name="Product" numFmtId="0">
      <sharedItems count="3">
        <s v="Widget"/>
        <s v="Gadget"/>
        <s v="Doohickey"/>
      </sharedItems>
    </cacheField>
    <cacheField name="Quarter" numFmtId="0">
      <sharedItems count="4">
        <s v="Q1"/><s v="Q2"/><s v="Q3"/><s v="Q4"/>
      </sharedItems>
    </cacheField>
    <cacheField name="Revenue" numFmtId="0">
      <sharedItems containsSemiMixedTypes="0" containsString="0"
          containsNumber="1" minValue="1000" maxValue="50000"/>
    </cacheField>
    <cacheField name="Units" numFmtId="0">
      <sharedItems containsSemiMixedTypes="0" containsString="0"
          containsNumber="1" containsInteger="1" minValue="10" maxValue="500"/>
    </cacheField>
  </cacheFields>
</pivotCacheDefinition>
```

**Key logic:**
- Scan source data to build `sharedItems` for string fields (unique values)
- For numeric fields: compute min/max, set `containsNumber="1"`, `containsString="0"`
- For date fields: set `containsDate="1"`, min/max dates
- For mixed fields: set `containsMixedTypes="1"`

### 2. pivotCacheRecords1.xml
Raw data values, one `<r>` per source row.

```xml
<pivotCacheRecords xmlns="..." count="100">
  <r>
    <x v="0"/>     <!-- "East" (index into sharedItems) -->
    <x v="0"/>     <!-- "Widget" -->
    <x v="0"/>     <!-- "Q1" -->
    <n v="10000"/>  <!-- Revenue (inline number) -->
    <n v="150"/>    <!-- Units -->
  </r>
  <!-- ... more rows -->
</pivotCacheRecords>
```

**Key logic:**
- String fields: `<x v="N"/>` where N is index into sharedItems
- Number fields: `<n v="123.45"/>`
- Date fields: `<d v="2024-01-15T00:00:00"/>`
- Boolean fields: `<b v="1"/>`
- Blank fields: `<m/>`

### 3. pivotTable1.xml
The pivot table definition — field assignments and layout.

```xml
<pivotTableDefinition xmlns="..." name="SalesPivot" cacheId="1"
    dataCaption="Values" updatedVersion="8" createdVersion="8"
    indent="0" outline="1" outlineData="1">
  <location ref="A1:F10" firstHeaderRow="1" firstDataRow="2" firstDataCol="2"/>
  <pivotFields count="5">
    <!-- Region: row field -->
    <pivotField axis="axisRow" showAll="0">
      <items count="5">
        <item x="0"/><item x="1"/><item x="2"/><item x="3"/>
        <item t="default"/>  <!-- grand total -->
      </items>
    </pivotField>
    <!-- Product: row field -->
    <pivotField axis="axisRow" showAll="0">
      <items count="4">
        <item x="0"/><item x="1"/><item x="2"/>
        <item t="default"/>
      </items>
    </pivotField>
    <!-- Quarter: column field -->
    <pivotField axis="axisCol" showAll="0">
      <items count="5">
        <item x="0"/><item x="1"/><item x="2"/><item x="3"/>
        <item t="default"/>
      </items>
    </pivotField>
    <!-- Revenue: data field -->
    <pivotField dataField="1" showAll="0"/>
    <!-- Units: data field -->
    <pivotField dataField="1" showAll="0"/>
  </pivotFields>
  <rowFields count="2">
    <field x="0"/>  <!-- Region -->
    <field x="1"/>  <!-- Product -->
  </rowFields>
  <rowItems count="...">
    <!-- Generated: one <i> per visible row -->
  </rowItems>
  <colFields count="1">
    <field x="2"/>  <!-- Quarter -->
  </colFields>
  <colItems count="...">
    <!-- Generated: one <i> per visible column -->
  </colItems>
  <dataFields count="2">
    <dataField name="Sum of Revenue" fld="3" baseField="0" baseItem="0"/>
    <dataField name="Sum of Units" fld="4" baseField="0" baseItem="0"/>
  </dataFields>
  <pivotTableStyleInfo name="PivotStyleLight16"
      showRowHeaders="1" showColHeaders="1"
      showRowStripes="0" showColStripes="0" showLastColumn="1"/>
</pivotTableDefinition>
```

---

## Implementation Plan

### Phase 1: Core (MVP) — ~300 LOC
Write a pivot table that Excel opens and refreshes correctly.

1. **PivotTable struct** + builder API
2. **Cache builder**: scan source data from worksheet cells to build:
   - `cacheFields` with `sharedItems` (unique string values)
   - `cacheRecords` with indexed string refs + inline numbers
3. **Pivot table XML writer**: generate pivotTableDefinition with:
   - `pivotFields` (axis assignment, items)
   - `rowFields`, `colFields`, `dataFields`
   - `location` (computed from field count)
   - `pivotTableStyleInfo`
4. **Wiring**: content types, relationships (workbook + sheet + pivotTable + cache)
5. **Integration**: `ws.add_pivot_table(row, col, &pivot)` on Worksheet

### Phase 2: Aggregations & Formatting — ~100 LOC
6. All 11 aggregation types (Sum, Count, Average, Max, Min, Product, CountNums, StdDev, StdDevP, Var, VarP)
7. Number format per value field
8. Calculated fields (`<calculatedItems>`)
9. Field subtotals control

### Phase 3: Layout & Filtering — ~100 LOC
10. Compact / Outline / Tabular layout
11. Page filter fields (`<pageFields>`)
12. Field sorting
13. Grand total control (row/column)
14. Multiple value fields in columns vs rows

### Phase 4: Advanced — ~100 LOC
15. Date grouping (by month, quarter, year)
16. Top N filter
17. Value filter (show items where sum > X)
18. Slicer connection (reference only — slicer XML is separate)
19. Refresh on open flag

---

## Relationship Wiring

```
workbook.xml.rels:
  rId_pc1 → pivotCache/pivotCacheDefinition1.xml (pivotCacheDefinition type)

sheetN.xml.rels:
  rId_pt1 → ../pivotTables/pivotTable1.xml (pivotTable type)

pivotTable1.xml.rels:
  rId1 → ../pivotCache/pivotCacheDefinition1.xml (pivotCacheDefinition type)

pivotCacheDefinition1.xml.rels:
  rId1 → pivotCacheRecords1.xml (pivotCacheRecords type)

[Content_Types].xml:
  /xl/pivotTables/pivotTable1.xml → pivotTable content type
  /xl/pivotCache/pivotCacheDefinition1.xml → pivotCacheDefinition content type
  /xl/pivotCache/pivotCacheRecords1.xml → pivotCacheRecords content type
```

---

## Data Scanning Algorithm

To build the cache, we need to scan the source data:

```
1. Parse source_range to get (sheet, r1, c1, r2, c2)
2. Read header row → field names
3. For each column:
   a. Collect all values
   b. If all numeric → containsNumber=1, compute min/max
   c. If all strings → build sharedItems (unique sorted values)
   d. If mixed → containsMixedTypes=1
   e. If has blanks → containsBlank=1
   f. If dates → containsDate=1, min/max dates
4. Build cacheRecords:
   - String values → <x v="index"/> (index into sharedItems)
   - Number values → <n v="123.45"/>
   - Blank → <m/>
```

---

## Acceptance Criteria

1. Create a pivot table from source data on another sheet
2. Excel opens with zero repair errors
3. Pivot table is functional — can be refreshed, filtered, sorted in Excel
4. Support row fields, column fields, value fields (Sum, Count, Average)
5. Support page filter fields
6. Support multiple value fields
7. Support all 11 aggregation types
8. Support calculated fields
9. Support 3 layout modes (Compact, Outline, Tabular)
10. Support pivot table styles (Light/Medium/Dark)
11. Support grand totals control
12. Support field subtotals control
13. Support number formatting on value fields
14. Demo file opens in Excel with working pivot table

---

## Estimated LOC

| Component | Lines |
|-----------|-------|
| PivotTable struct + builder | ~80 |
| Cache scanner (data → sharedItems) | ~120 |
| pivotCacheDefinition writer | ~60 |
| pivotCacheRecords writer | ~40 |
| pivotTableDefinition writer | ~150 |
| Relationship wiring | ~50 |
| Content types | ~10 |
| Worksheet integration | ~20 |
| **Total** | **~530** |

This would make zavora-xlsx the **only Rust crate** that can create pivot tables from scratch.
