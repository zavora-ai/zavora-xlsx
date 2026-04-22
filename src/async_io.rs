//! Async I/O support for zavora-xlsx.
//!
//! Provides `open_async()` and `save_async()` methods that delegate file I/O
//! to tokio's blocking thread pool via `tokio::task::spawn_blocking`.
//!
//! Requires the `async-tokio` feature flag.
//!
//! # Example
//! ```ignore
//! use zavora_xlsx::Workbook;
//!
//! #[tokio::main]
//! async fn main() -> zavora_xlsx::Result<()> {
//!     let mut wb = Workbook::open_async("input.xlsx").await?;
//!     let ws = wb.worksheet(0)?;
//!     ws.write(0, 0, "Async!")?;
//!     wb.save_async("output.xlsx").await?;
//!     Ok(())
//! }
//! ```
//!
//! The async methods use `spawn_blocking` because the underlying ZIP and XML
//! parsing is CPU-bound and synchronous. This keeps the tokio runtime responsive
//! while the heavy work runs on a dedicated thread.

// The actual async methods are implemented directly on Workbook in workbook/mod.rs
// behind the `async-tokio` feature flag. This module exists for documentation
// and to satisfy the `pub mod async_io` declaration in lib.rs.
