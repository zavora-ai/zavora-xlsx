//! Serde integration for zavora-xlsx.
//!
//! This module provides serialization and deserialization of Rust structs
//! to and from Excel worksheet rows. Struct field names become column headers,
//! and each struct instance maps to a row.
//!
//! Requires the `serde-support` feature flag.
//!
//! # Example
//! ```ignore
//! use serde::{Serialize, Deserialize};
//! use zavora_xlsx::Workbook;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Record {
//!     name: String,
//!     value: f64,
//!     active: bool,
//! }
//!
//! let mut wb = Workbook::new();
//! let data = vec![
//!     Record { name: "Alice".into(), value: 100.0, active: true },
//!     Record { name: "Bob".into(), value: 200.0, active: false },
//! ];
//! wb.write_rows(0, &data).unwrap();
//! ```

pub mod de;
pub mod ser;

pub use de::read_rows;
pub use ser::write_rows;
