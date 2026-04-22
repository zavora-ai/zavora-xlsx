# zavora-xlsx-node

Node.js native addon for [zavora-xlsx](https://github.com/zavora-ai/zavora-xlsx), built with [napi-rs](https://napi.rs).

Read, write, and edit Excel `.xlsx` files from Node.js with native Rust performance.

## Build

```bash
cd zavora-xlsx-node
npm install
napi build
```

## API overview

### Workbook

```javascript
const { Workbook } = require('zavora-xlsx');

// Create
const wb = new Workbook();

// Open from file or buffer
const wb2 = Workbook.open('report.xlsx');
const wb3 = Workbook.openFromBuffer(fs.readFileSync('report.xlsx'));

// Sheet management
const ws = wb.worksheet(0);
const ws2 = wb.addWorksheet();
const ws3 = wb.addWorksheetWithName('Sales');
wb.sheetNames();   // ['Sheet1', 'Sales']
wb.sheetCount();   // 2

// Document properties
wb.setProperties({ title: 'Report', author: 'Team', company: 'Acme' });
const props = wb.properties();

// Save
wb.save('output.xlsx');
const buffer = wb.saveToBuffer();
```

### Worksheet — writing cells

```javascript
const { Format } = require('zavora-xlsx');

const bold = new Format().bold().fontSize(12).fontColor('#FFFFFF').backgroundColor('#2B579A');
const currency = new Format().numFormat('$#,##0.00');

ws.writeString(0, 0, 'Product', bold);
ws.writeNumber(1, 0, 1234.56, currency);
ws.writeBoolean(2, 0, true);
ws.writeFormula(3, 0, 'SUM(A2:A3)');
ws.writeBlank(4, 0, bold);
```

### Worksheet — reading cells

```javascript
const val = ws.readCell(0, 0);
// Returns: string | number | boolean | null | { formula, cachedValue }

const range = ws.usedRange();
// Returns: { firstRow, firstCol, lastRow, lastCol } | null
```

### Worksheet — layout

```javascript
ws.setColumnWidth(0, 20);
ws.setRowHeight(0, 30);
ws.setFreezePanes(1, 0);
ws.mergeRange(0, 0, 0, 3, 'Title');
ws.autofit();
ws.setZoom(125);
ws.name();
ws.setName('Data');
```

### Format

All methods are chainable:

```javascript
const fmt = new Format()
    .bold()
    .italic()
    .strikethrough()
    .fontSize(14)
    .fontName('Arial')
    .fontColor('#FF0000')
    .backgroundColor('#FFFF00')
    .numFormat('#,##0.00')
    .underline('single')       // 'single' | 'double'
    .border('thin')            // 'none' | 'thin' | 'medium' | 'thick' | 'double' | 'dashed' | 'dotted'
    .align('center')           // 'left' | 'center' | 'right' | 'fill' | 'justify' | 'top' | 'middle' | 'bottom'
    .textWrap()
    .shrinkToFit()
    .indent(2)
    .rotation(45);
```

### Chart

```javascript
const { Chart } = require('zavora-xlsx');

// Types: 'bar', 'column', 'line', 'pie', 'scatter', 'area', 'doughnut', 'radar'
const chart = new Chart('column');
chart.addSeries('Sheet1!$B$2:$B$5', 'Sheet1!$A$2:$A$5', 'Q1 Sales');
chart.setTitle('Quarterly Sales');
chart.setXAxisName('Product');
chart.setYAxisName('Revenue');
chart.setSize(800, 400);
chart.setLegendPosition('bottom');  // 'top' | 'bottom' | 'left' | 'right' | 'none'

ws.insertChart(6, 0, chart);
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

ws.addTable(0, 0, 10, 2, table);
```

### CSV export

```javascript
const csv = ws.toCsvString();
const tsv = ws.toCsvString({ delimiter: '\t' });
const custom = ws.toCsvString({
    delimiter: ';',
    quote: '"',
    lineEnding: '\n',
    dateFormat: 'dd/mm/yyyy',
});
```

## Cell value types

| Excel cell | JavaScript return type |
|------------|----------------------|
| Empty | `null` |
| String | `string` |
| Number | `number` |
| Boolean | `boolean` |
| DateTime | `number` (Excel serial) |
| Error | `string` |
| Rich text | `string` (plain text) |
| Formula | `{ formula: string, cachedValue: any }` |

## License

Apache-2.0
