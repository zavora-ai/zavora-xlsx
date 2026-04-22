# zavora-xlsx-node

> **Status: Planned** — This crate will be a separate workspace member.

Node.js bindings for zavora-xlsx using napi-rs.

## Planned Features

- `Workbook`, `Worksheet`, `Format`, `Chart`, `Table` JS classes
- Cell writing methods
- Save to file and save to Buffer
- `npm install` packaging via napi-build

## Example (planned)

```javascript
const { Workbook, Format } = require('zavora-xlsx');

const wb = new Workbook();
const ws = wb.worksheet(0);

const bold = new Format().bold();
ws.writeString(0, 0, 'Name', bold);
ws.writeString(0, 1, 'Score', bold);
ws.writeString(1, 0, 'Alice');
ws.writeNumber(1, 1, 95.5);

wb.save('output.xlsx');

// Or save to Buffer
const buffer = wb.saveToBuffer();
```

## Architecture

This crate wraps the core `zavora-xlsx` API using napi-rs's `#[napi]`
attribute macros. It will be built with napi-build for distribution as
a native Node.js addon.

## Dependencies

```toml
[dependencies]
napi = { version = "2", features = ["napi4"] }
napi-derive = "2"
zavora-xlsx = { path = ".." }

[build-dependencies]
napi-build = "2"
```
