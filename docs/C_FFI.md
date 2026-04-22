# zavora-xlsx C FFI

> **Status: 📋 Planned** — Header file defined, implementation behind `cffi` feature flag.

C-compatible API for cross-language use of zavora-xlsx.

## Overview

The C FFI provides an opaque handle-based API for creating and manipulating Excel workbooks from C, C++, and any language with C FFI support.

## Header File

The C header is defined at `docs/planned/zavora_xlsx.h`.

## Planned API

```c
#include "zavora_xlsx.h"

/* Create a new workbook */
ZavoraWorkbook* wb = zavora_workbook_new();

/* Write cells */
zavora_worksheet_write_string(wb, 0, 0, 0, "Hello");
zavora_worksheet_write_number(wb, 0, 0, 1, 42.5);
zavora_worksheet_write_bool(wb, 0, 1, 0, 1);

/* Save to file */
int rc = zavora_workbook_save(wb, "output.xlsx");
if (rc != 0) {
    const char* err = zavora_last_error();
    fprintf(stderr, "Error: %s\n", err);
}

/* Save to buffer */
uint8_t* buf = NULL;
size_t len = 0;
zavora_workbook_save_to_buffer(wb, &buf, &len);
/* ... use buffer ... */
zavora_buffer_free(buf, len);

/* Cleanup */
zavora_workbook_free(wb);
```

## Functions

| Function | Description |
|----------|-------------|
| `zavora_workbook_new()` | Create a new empty workbook. Returns `NULL` on failure. |
| `zavora_worksheet_write_string(wb, sheet, row, col, text)` | Write a string to a cell. Returns 0 on success. |
| `zavora_worksheet_write_number(wb, sheet, row, col, value)` | Write a number to a cell. Returns 0 on success. |
| `zavora_worksheet_write_bool(wb, sheet, row, col, value)` | Write a boolean to a cell. Returns 0 on success. |
| `zavora_workbook_save(wb, path)` | Save workbook to file. Returns 0 on success. |
| `zavora_workbook_save_to_buffer(wb, out_ptr, out_len)` | Save to in-memory buffer. Free with `zavora_buffer_free()`. |
| `zavora_buffer_free(ptr, len)` | Free a buffer from `save_to_buffer`. |
| `zavora_workbook_free(wb)` | Free a workbook handle. |
| `zavora_last_error()` | Get last error message. Returns `NULL` if no error. |

## Building

Enable the `cffi` feature flag:

```bash
cargo build --features cffi
```

This produces a C-compatible shared library that can be linked with `-lzavora_xlsx`.

## Error Handling

All functions that can fail return an `int` status code:
- `0` = success
- `-1` = failure (call `zavora_last_error()` for details)

The error string is valid until the next FFI call.

## Thread Safety

The workbook handle is **not** thread-safe. Each thread should create its own workbook, or access must be synchronized externally.
