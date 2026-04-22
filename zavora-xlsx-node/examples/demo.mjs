// Node.js Bindings Demo
// Note: This is a reference example showing the API.
// To run it, first build the native addon with `napi build` in the zavora-xlsx-node directory.

// const { Workbook, Format, Chart, Table } = require('../index.js');
//
// // Create a new workbook
// const wb = new Workbook();
// const ws = wb.worksheet(0);
//
// // Create formats
// const header = new Format().bold().fontSize(12).fontColor('#FFFFFF').backgroundColor('#2B579A');
// const currency = new Format().numFormat('#,##0.00');
//
// // Write headers
// ws.writeString(0, 0, 'Product', header);
// ws.writeString(0, 1, 'Q1 Sales', header);
// ws.writeString(0, 2, 'Q2 Sales', header);
// ws.writeString(0, 3, 'Total', header);
//
// // Write data
// const data = [
//     ['Widget A', 15000, 18000],
//     ['Widget B', 22000, 19500],
//     ['Widget C', 8500, 12000],
//     ['Widget D', 31000, 28000],
// ];
//
// data.forEach((row, i) => {
//     const r = i + 1;
//     ws.writeString(r, 0, row[0]);
//     ws.writeNumber(r, 1, row[1], currency);
//     ws.writeNumber(r, 2, row[2], currency);
//     ws.writeFormula(r, 3, `B${r+1}+C${r+1}`);
// });
//
// // Add a chart
// const chart = new Chart('column');
// chart.addSeries('Sheet1!$B$2:$B$5', 'Sheet1!$A$2:$A$5', 'Q1');
// chart.addSeries('Sheet1!$C$2:$C$5', 'Sheet1!$A$2:$A$5', 'Q2');
// chart.setTitle('Quarterly Sales');
// ws.insertChart(6, 0, chart);
//
// // Set properties
// wb.setProperties({ title: 'Sales Report', author: 'Node.js Demo' });
//
// // Save
// wb.save('output/node_demo.xlsx');
// console.log('Saved to output/node_demo.xlsx');
//
// // Or save to buffer
// const buffer = wb.saveToBuffer();
// console.log(`Buffer size: ${buffer.length} bytes`);

console.log('zavora-xlsx Node.js bindings API reference');
console.log('Build the native addon first: cd zavora-xlsx-node && napi build');
console.log('Then uncomment the code above to run the demo.');
