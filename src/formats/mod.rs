//! File format variants: XLSB, XLS (BIFF8), ODS, CSV/TSV export, Strict OOXML.

pub mod csv_export;
pub mod strict_ooxml;

#[cfg(feature = "xlsb")]
pub mod xlsb_reader;

#[cfg(feature = "xls")]
pub mod xls_reader;

#[cfg(feature = "ods")]
pub mod ods;
