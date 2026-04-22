# zavora-xlsx Documentation

High-performance Rust crate for reading, writing, and editing Excel `.xlsx` files.

## Getting Started

New to zavora-xlsx? Start here:

- [Getting Started](guide/getting-started.md) — installation, first workbook, cell types, saving and opening files

## Core Guides

Step-by-step guides covering every feature of the library:

| Guide | What you'll learn |
|-------|-------------------|
| [Formatting](guide/formatting.md) | Fonts, colors, borders, alignment, number formats, patterns, gradients, themes |
| [Charts](guide/charts.md) | Standard, 3D, and ChartEx chart types with series configuration |
| [Tables & Data](guide/tables-and-data.md) | Tables, data validation, conditional formatting, autofilter |
| [Reading Files](guide/reading.md) | Opening files, reading cells, streaming reader, edit mode |
| [Advanced Features](guide/advanced.md) | Streaming write, pivot tables, images, sparklines, protection |
| [Serde Integration](guide/serde.md) | Serialize/deserialize structs to worksheet rows |

## Ecosystem

Companion crates and language bindings:

| Crate | Description | Status |
|-------|-------------|--------|
| [CLI Tool](ecosystem/cli.md) | `zavora-xlsx inspect`, `export`, `convert` | ✅ Implemented |
| [Derive Macro](ecosystem/derive-macro.md) | `#[derive(ExcelRow)]` for typed row mapping | ✅ Implemented |
| [Node.js Bindings](ecosystem/node-bindings.md) | napi-rs bindings for Node.js | ✅ Implemented |
| [Python Bindings](ecosystem/python-bindings.md) | PyO3 bindings for Python | ✅ Implemented |
| [C FFI](ecosystem/c-ffi.md) | C-compatible API | 📋 Planned |

## Roadmap

See the [feature roadmap](roadmap.md) for what's implemented and what's planned.

## Quick Links

- [GitHub Repository](https://github.com/zavora-ai/zavora-xlsx)
- [API Reference (docs.rs)](https://docs.rs/zavora-xlsx)
- [Crate (crates.io)](https://crates.io/crates/zavora-xlsx)
