# AGENTS.md

## Project overview

zavora-xlsx is a high-performance Rust crate for reading, writing, and editing Excel `.xlsx` files. It is a Cargo workspace with five crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `zavora-xlsx` | Library | Core xlsx engine — cells, formatting, charts, tables, conditional formatting, images, pivot tables, streaming, formula engine |
| `zavora-xlsx-derive` | Proc-macro | `#[derive(ExcelRow)]` for typed row mapping without serde |
| `zavora-xlsx-cli` | Binary | CLI tool: `inspect`, `export`, `convert` subcommands |
| `zavora-xlsx-node` | cdylib | Node.js bindings via napi-rs |
| `zavora-xlsx-python` | cdylib | Python bindings via PyO3 + maturin |

The core library has 4 runtime dependencies (`quick-xml`, `zip`, `atoi_simd`, `fast-float2`) and targets zero Excel repair errors.

## Build commands

```bash
# Build the entire workspace
cargo build --workspace

# Build only the core library
cargo build -p zavora-xlsx

# Build the CLI tool
cargo build -p zavora-xlsx-cli

# Check binding crates (cdylib crates can't link without their runtime)
cargo check -p zavora-xlsx-node
cargo check -p zavora-xlsx-python
```

## Test commands

```bash
# Run all tests across the workspace
cargo test --workspace

# Core library tests only
cargo test -p zavora-xlsx

# CLI integration tests
cargo test -p zavora-xlsx-cli

# Derive macro tests (includes trybuild compile-fail tests)
cargo test -p zavora-xlsx-derive

# Run a specific test
cargo test test_name

# Run an example
cargo run --example derive_macro_demo
```

All tests must pass before committing. The derive macro crate uses `trybuild` for compile-fail tests — if you change error messages, run `TRYBUILD=overwrite cargo test -p zavora-xlsx-derive` to regenerate `.stderr` files.

## Lint and format

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

Both must pass clean before any commit or PR. Clippy is run with `-D warnings` (warnings are errors).

For the binding crates that can't link, use `cargo clippy -p zavora-xlsx -p zavora-xlsx-cli -p zavora-xlsx-derive -- -D warnings` and `cargo check -p zavora-xlsx-node -p zavora-xlsx-python` separately.

## Code style

- Rust 2024 edition across all crates
- No `unsafe` in the core library (the Python bindings have `#![allow(unsafe_op_in_unsafe_fn)]` for PyO3)
- Builder pattern for `Format` — methods consume `self` and return `Self`
- Worksheet methods return `Result<&mut Self>` for chaining
- All coordinates are 0-based: `row: u32` (RowNum), `col: u16` (ColNum)
- Use `impl IntoExcelData` for generic cell writing, not type-specific methods
- Errors use the central `zavora_xlsx::Error` enum — never `panic!` in library code
- Keep `pub(crate)` for internal fields; expose read accessors for public API
- XML generation uses the internal `XmlWriter` — never write raw XML strings
- ZIP handling uses the internal `ZipReader`/`ZipOutput` wrappers

## Architecture

### Core library (`src/`)

```
src/
├── lib.rs              # Public re-exports
├── cell.rs             # CellValue enum, IntoExcelData trait
├── format.rs           # Format builder (font, borders, alignment, colors)
├── error.rs            # Error enum (Io, Zip, Xml, SheetNotFound, etc.)
├── datetime.rs         # ExcelDateTime (serial date conversion)
├── derive_traits.rs    # ExcelRowWriter, ExcelRowReader traits
├── properties.rs       # DocProperties, CustomProperty
├── streaming.rs        # StreamingWorkbook (constant-memory write)
├── utility.rs          # RowNum, ColNum, cell ref parsing, col_to_letter
├── formula.rs          # Formula string utilities
├── workbook/           # Workbook struct, open/save, sheet management
│   ├── mod.rs          # Workbook impl (open, save, worksheet access)
│   ├── save.rs         # save_to_buffer, pivot cache resolution
│   └── xml.rs          # Content types, workbook XML generation
├── worksheet/          # Worksheet struct, cell operations
│   ├── mod.rs          # Worksheet struct, read_cell, used_range
│   ├── write.rs        # write, write_formula, write_rich_text, write_blank
│   ├── layout.rs       # Column widths, row heights, freeze, autofit, print, zoom
│   ├── features.rs     # insert_chart, add_table, add_conditional_format, etc.
│   ├── ops.rs          # insert_rows, remove_rows, insert_columns, remove_columns
│   └── types.rs        # Comment, Hyperlink, PrintSettings, SheetProtection, etc.
├── features/           # Excel features (each is a self-contained module)
│   ├── chart.rs        # Chart, ChartSeries, ChartType (18 types)
│   ├── chartex.rs      # ChartEx: Waterfall, Funnel, Sunburst, Histogram, BoxWhisker, Map
│   ├── treemap.rs      # TreemapChart
│   ├── table.rs        # Table, TableColumn, TableStyle, CustomTableStyle
│   ├── conditional.rs  # 11 conditional format rule types
│   ├── validation.rs   # DataValidation, ValidationRule
│   ├── image.rs        # Image (PNG/JPEG with auto dimension detection)
│   ├── sparkline.rs    # Sparkline (Line, Column, WinLoss)
│   ├── pivot.rs        # PivotTable with calculated fields, grouping
│   ├── shape.rs        # Drawing shapes (Rectangle, Ellipse, Arrow, etc.)
│   ├── slicer.rs       # Slicer visual filter controls
│   └── timeline.rs     # Timeline date-based filter controls
├── formula_engine/     # Formula parser, evaluator, dependency graph
├── reader/             # xlsx reader (sheet, chart, table, comment, etc.)
├── writer/             # xlsx writer (sheet, chart, table, style, etc.)
├── formats/            # CSV export, Strict OOXML, XLS/XLSB stubs
├── model/              # SharedStringTable, StyleRegistry (internal)
├── xml/                # XmlWriter, XmlReader (internal)
├── zip/                # ZipReader, ZipOutput (internal)
├── serde_support/      # serde Serialize/Deserialize for rows (behind feature flag)
├── crypto/             # Encryption stubs (behind feature flag)
├── async_io.rs         # Async I/O (behind feature flag)
└── cffi.rs             # C FFI (behind feature flag)
```

### Workspace crates

- `zavora-xlsx-derive/src/lib.rs` — Single file: attribute parser, field analyzer, writer/reader codegen
- `zavora-xlsx-cli/src/main.rs` — Single file: clap args, inspect/export/convert handlers
- `zavora-xlsx-node/src/` — 6 modules: lib, error, workbook, worksheet, format, chart, table
- `zavora-xlsx-python/src/` — 6 modules: lib, error, workbook, worksheet, format, chart, table

## Key patterns

### Workbook/Worksheet ownership

The core `Workbook` owns a `Vec<Worksheet>`. Access is via `wb.worksheet(idx)` (mutable) or `wb.worksheet_ref(idx)` (immutable). The binding crates (Node, Python) use `Arc<Mutex<Workbook>>` to share ownership between Workbook and Worksheet handles.

### Format builder

`Format` methods consume `self` and return `Self`:

```rust
let fmt = Format::new().bold().font_size(14.0).font_color("#FF0000");
```

In the Node bindings, `RefCell<Format>` provides interior mutability. In the Python bindings, `PyRefMut<Self>` enables chaining.

### Error handling

All fallible operations return `zavora_xlsx::Result<T>` (alias for `Result<T, zavora_xlsx::Error>`). The binding crates convert errors:
- Node: `Error → napi::Error` via `IntoNapi` trait
- Python: `Error → PyErr` via `IntoPyResult` trait, with variant-specific mapping (Io→OSError, SheetNotFound→IndexError, InvalidData→ValueError)

### Cell value types

`CellValue` enum: `Empty`, `String`, `Number(f64)`, `Bool`, `DateTime(ExcelDateTime)`, `Error(String)`, `Formula { formula, cached_value }`, `RichText`.

Writing uses the `IntoExcelData` trait — implemented for `&str`, `String`, `f64`, `bool`, all integer types, and `ExcelDateTime`.

## Feature flags

| Flag | Adds | Dependency |
|------|------|-----------|
| `serde-support` | `write_rows` / `read_rows` for serde structs | `serde` |
| `async-tokio` | Async file I/O | `tokio` |
| `mmap` | Memory-mapped reading | `memmap2` |
| `wasm` | WASM target support | — |
| `cffi` | C FFI bindings | — |
| `xlsb` | XLSB reading (experimental) | — |
| `xls` | XLS reading (experimental) | — |
| `ods` | ODS support (experimental) | — |
| `crypto` | Encrypted workbook support (experimental) | — |

## File conventions

- Examples go in `examples/` (core crate) or `<crate>/examples/`
- Tests go in `tests/` (core crate integration tests) or inline `#[cfg(test)]` modules
- Documentation lives in `docs/guide/` (user guides) and `docs/ecosystem/` (crate-specific docs)
- Output files from examples go in `output/` (gitignored)
- Spec files live in `.kiro/specs/` (gitignored from the published crate)

## Things to avoid

- Do not add heavy dependencies — the core library has only 4 runtime deps
- Do not use `unwrap()` or `panic!()` in library code — return `Result` or `Option`
- Do not write raw XML strings — use `XmlWriter` for all XML generation
- Do not modify `src/model/` or `src/zip/` without understanding the save pipeline
- Do not break the `Format` builder pattern — methods must consume and return `Self`
- Do not change cell coordinate types — `RowNum = u32`, `ColNum = u16` are used everywhere
- Do not add `unsafe` to the core library
- Do not modify `docs/planned/` — those are historical planning documents

## Commit conventions

- Format: `type: description` (e.g., `feat: add waterfall chart support`, `fix: correct merge range offset`)
- Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`
- Keep PR titles under 70 characters
- Run `cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace` before committing

## Publishing

The core crate and derive macro are published to crates.io. The CLI, Node, and Python crates are workspace members but published separately.

```bash
# Dry run
cargo publish --dry-run -p zavora-xlsx
cargo publish --dry-run -p zavora-xlsx-derive

# Audit
cargo audit

# Docs
cargo doc --no-deps -p zavora-xlsx
```

## Documentation

- `docs/index.md` — Documentation home
- `docs/guide/getting-started.md` — Installation and first workbook
- `docs/guide/formatting.md` — Complete formatting reference
- `docs/guide/charts.md` — All chart types and configuration
- `docs/guide/tables-and-data.md` — Tables, validation, conditional formatting
- `docs/guide/reading.md` — Reading files, streaming reader, edit mode
- `docs/guide/advanced.md` — Streaming write, pivot tables, images, protection
- `docs/guide/serde.md` — Serde integration
- `docs/ecosystem/cli.md` — CLI tool reference
- `docs/ecosystem/derive-macro.md` — Derive macro reference
- `docs/ecosystem/node-bindings.md` — Node.js API reference
- `docs/ecosystem/python-bindings.md` — Python API reference
