# zavora-xlsx-node

> **Status: ✅ Implemented** — Available as the `zavora-xlsx-node` workspace member.

Node.js bindings for zavora-xlsx using napi-rs.

## Building

```bash
cd zavora-xlsx-node
napi build
```

## API Reference

### Workbook

```javascript
const { Workbook } = require('zavora-xlsx');

// Create a new workbook
const wb = new Workbook();

// Open an existing file
const wb2 = Workbook.open('report.xlsx');

// Open from a Buffer
const buffer = fs.readFileSync('report.xlsx');
const wb3 = Workbook.openFromBuffer(buffer);

// Access worksheets
const ws = wb.worksheet(0);           // by index
const ws2 = wb.addWorksheet();         // add new sheet
const ws3 = wb.addWorksheetWithName('Sales');

// Sheet info
wb.sheetNames();   // ['Sheet1', 'Sales']
wb.sheetCount();   // 2

// Document properties
wb.setProperties({
    title: 'Sales Report',
    author: 'Node.js Demo',
    subject: 'Q4 Results',
    description: 'Quarterly sales data',
    keywords: 'sales, quarterly',
    category: 'Reports',
    company: 'Acme Corp',
});
const props = wb.properties();

// Save
wb.save('output.xlsx');
const buf = wb.saveToBuffer();
```

### Worksheet

```javascript
const ws = wb.worksheet(0);

// Write cells
ws.writeString(0, 0, 'Hello');
ws.writeString(0, 0, 'Hello', format);  // with format
ws.writeNumber(1, 0, 42.5);
ws.writeNumber(1, 0, 42.5, format);
ws.writeBoolean(2, 0, true);
ws.writeFormula(3, 0, 'SUM(A1:A3)');
ws.writeBlank(4, 0, format);

// Read cells
const value = ws.readCell(0, 0);  // returns string, number, boolean, null, or object
const range = ws.usedRange();     // { firstRow, firstCol, lastRow, lastCol } or null

// Sheet name
ws.name();
ws.setName('Data');

// Layout
ws.setColumnWidth(0, 20.0);
ws.setRowHeight(0, 30.0);
ws.setFreezePanes(1, 0);
ws.mergeRange(0, 0, 0, 3, 'Title', format);
ws.autofit();
ws.setZoom(125);

// Charts and tables
ws.insertChart(10, 0, chart);
ws.addTable(0, 0, 5, 3, table);

// CSV export
const csv = ws.toCsvString();
const tsv = ws.toCsvString({ delimiter: '\t' });
const custom = ws.toCsvString({
    delimiter: ',',
    quote: '"',
    lineEnding: '\r\n',
    dateFormat: 'yyyy-mm-dd',
});
```

### Format

```javascript
const { Format } = require('zavora-xlsx');

const fmt = new Format()
    .bold()
    .italic()
    .strikethrough()
    .textWrap()
    .shrinkToFit()
    .fontSize(14)
    .fontName('Arial')
    .fontColor('#FF0000')
    .backgroundColor('#FFFF00')
    .numFormat('#,##0.00')
    .indent(2)
    .rotation(45)
    .underline('single')    // 'single' or 'double'
    .border('thin')         // 'none', 'thin', 'medium', 'thick', 'double', 'dashed', 'dotted'
    .align('center');       // 'left', 'center', 'right', 'fill', 'justify', 'top', 'middle', 'bottom'
```

### Chart

```javascript
const { Chart } = require('zavora-xlsx');

// Types: 'bar', 'column', 'line', 'pie', 'scatter', 'area', 'doughnut', 'radar'
const chart = new Chart('column');

chart.addSeries('Sheet1!$B$2:$B$5', 'Sheet1!$A$2:$A$5', 'Q1 Sales');
chart.addSeries('Sheet1!$C$2:$C$5', 'Sheet1!$A$2:$A$5', 'Q2 Sales');
chart.setTitle('Quarterly Sales');
chart.setXAxisName('Product');
chart.setYAxisName('Revenue');
chart.setSize(800, 400);
chart.setLegendPosition('bottom');  // 'bottom', 'top', 'left', 'right', 'none'

ws.insertChart(10, 0, chart);
```

### Table

```javascript
const { Table } = require('zavora-xlsx');

const table = new Table();
table.setName('SalesData');
table.setColumns([
    { name: 'Product' },
    { name: 'Revenue', totalFunction: 'sum' },
    { name: 'Count', totalLabel: 'Total' },
]);
table.setStyle('TableStyleMedium9');
table.setTotalRow(true);
table.setAutofilter(true);

ws.addTable(0, 0, 5, 2, table);
```

## Cell Value Types

When reading cells with `readCell()`, the return type depends on the cell content:

| Excel Cell | JavaScript Type |
|------------|----------------|
| Empty | `null` |
| String | `string` |
| Number | `number` |
| Boolean | `boolean` |
| DateTime | `number` (Excel serial) |
| Error | `string` (error text) |
| Rich Text | `string` (plain text) |
| Formula | `{ formula: string, cachedValue: any }` |

## Example

See `zavora-xlsx-node/examples/demo.mjs` for a complete API reference example.

## Architecture

This crate wraps the core `zavora-xlsx` API using napi-rs's `#[napi]` attribute macros. It is built with napi-build for distribution as a native Node.js addon.

## Dependencies

```toml
[dependencies]
napi = { version = "2", features = ["napi4"] }
napi-derive = "2"
zavora-xlsx = { path = ".." }

[build-dependencies]
napi-build = "2"
```
