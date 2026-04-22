use std::sync::{Arc, Mutex};
use napi::{Env, JsUnknown};
use napi_derive::napi;
use crate::error::IntoNapi;
use crate::format::Format;
use crate::chart::Chart;
use crate::table::Table;

#[napi]
pub struct Worksheet {
    pub(crate) workbook: Arc<Mutex<zavora_xlsx::Workbook>>,
    pub(crate) index: usize,
}

#[napi(object)]
pub struct UsedRangeResult {
    pub first_row: u32,
    pub first_col: u32,
    pub last_row: u32,
    pub last_col: u32,
}

#[napi(object)]
pub struct CsvOptionsJs {
    pub delimiter: Option<String>,
    pub quote: Option<String>,
    pub line_ending: Option<String>,
    pub date_format: Option<String>,
}

#[napi]
impl Worksheet {
    // ── Cell write methods (Task 8.2) ──

    #[napi]
    pub fn write_string(
        &self,
        row: u32,
        col: u16,
        value: String,
        format: Option<&Format>,
    ) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        match format {
            Some(fmt) => {
                ws.write_with_format(row, col, value.as_str(), &*fmt.inner.borrow())
                    .into_napi()?;
            }
            None => {
                ws.write(row, col, value.as_str()).into_napi()?;
            }
        }
        Ok(())
    }

    #[napi]
    pub fn write_number(
        &self,
        row: u32,
        col: u16,
        value: f64,
        format: Option<&Format>,
    ) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        match format {
            Some(fmt) => {
                ws.write_with_format(row, col, value, &*fmt.inner.borrow())
                    .into_napi()?;
            }
            None => {
                ws.write(row, col, value).into_napi()?;
            }
        }
        Ok(())
    }

    #[napi]
    pub fn write_boolean(
        &self,
        row: u32,
        col: u16,
        value: bool,
        format: Option<&Format>,
    ) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        match format {
            Some(fmt) => {
                ws.write_with_format(row, col, value, &*fmt.inner.borrow())
                    .into_napi()?;
            }
            None => {
                ws.write(row, col, value).into_napi()?;
            }
        }
        Ok(())
    }

    #[napi]
    pub fn write_formula(
        &self,
        row: u32,
        col: u16,
        formula: String,
        format: Option<&Format>,
    ) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        ws.write_formula(row, col, &formula).into_napi()?;
        if let Some(fmt) = format {
            ws.set_cell_format(row, col, &*fmt.inner.borrow()).into_napi()?;
        }
        Ok(())
    }

    #[napi]
    pub fn write_blank(&self, row: u32, col: u16, format: &Format) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        ws.write_blank(row, col, &*format.inner.borrow()).into_napi()?;
        Ok(())
    }

    // ── Cell read methods (Task 8.3) ──

    #[napi]
    pub fn read_cell(&self, env: Env, row: u32, col: u16) -> napi::Result<JsUnknown> {
        let wb = self.lock()?;
        let ws = wb.worksheet_ref(self.index).into_napi()?;
        let cell_value = ws.read_cell(row, col);
        cell_value_to_js(&env, &cell_value)
    }

    #[napi]
    pub fn used_range(&self) -> napi::Result<Option<UsedRangeResult>> {
        let wb = self.lock()?;
        let ws = wb.worksheet_ref(self.index).into_napi()?;
        Ok(ws.used_range().map(|(r1, c1, r2, c2)| UsedRangeResult {
            first_row: r1,
            first_col: c1 as u32,
            last_row: r2,
            last_col: c2 as u32,
        }))
    }

    // ── Worksheet name methods (Task 8.4) ──

    #[napi]
    pub fn name(&self) -> napi::Result<String> {
        let wb = self.lock()?;
        let ws = wb.worksheet_ref(self.index).into_napi()?;
        Ok(ws.name().to_string())
    }

    #[napi]
    pub fn set_name(&self, name: String) -> napi::Result<()> {
        let mut wb = self.lock()?;
        wb.rename_worksheet(self.index, &name).into_napi()
    }

    // ── Layout methods (Task 10.1) ──

    #[napi]
    pub fn set_column_width(&self, col: u16, width: f64) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        ws.set_column_width(col, width).into_napi()?;
        Ok(())
    }

    #[napi]
    pub fn set_row_height(&self, row: u32, height: f64) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        ws.set_row_height(row, height).into_napi()?;
        Ok(())
    }

    #[napi]
    pub fn set_freeze_panes(&self, row: u32, col: u16) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        ws.set_freeze_panes(row, col).into_napi()?;
        Ok(())
    }

    #[napi]
    pub fn merge_range(
        &self,
        r1: u32,
        c1: u16,
        r2: u32,
        c2: u16,
        text: String,
        format: Option<&Format>,
    ) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        let fmt = match format {
            Some(f) => f.inner.borrow().clone(),
            None => zavora_xlsx::Format::new(),
        };
        ws.merge_range(r1, c1, r2, c2, &text, &fmt).into_napi()?;
        Ok(())
    }

    #[napi]
    pub fn autofit(&self) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        ws.autofit().into_napi()?;
        Ok(())
    }

    #[napi]
    pub fn set_zoom(&self, percent: u16) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        ws.set_zoom(percent);
        Ok(())
    }

    // ── Chart and table insertion (Task 10.2) ──

    #[napi]
    pub fn insert_chart(&self, row: u32, col: u16, chart: &Chart) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        ws.insert_chart(row, col, &*chart.inner.borrow()).into_napi()?;
        Ok(())
    }

    #[napi]
    pub fn add_table(
        &self,
        first_row: u32,
        first_col: u16,
        last_row: u32,
        last_col: u16,
        table: &Table,
    ) -> napi::Result<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_napi()?;
        ws.add_table(first_row, first_col, last_row, last_col, &*table.inner.borrow())
            .into_napi()?;
        Ok(())
    }

    // ── CSV export (Task 11.1) ──

    #[napi]
    pub fn to_csv_string(&self, options: Option<CsvOptionsJs>) -> napi::Result<String> {
        let wb = self.lock()?;
        let ws = wb.worksheet_ref(self.index).into_napi()?;
        let mut opts = zavora_xlsx::CsvOptions::new();
        if let Some(ref js_opts) = options {
            if let Some(ref d) = js_opts.delimiter {
                if d.is_empty() {
                    return Err(napi::Error::new(
                        napi::Status::InvalidArg,
                        "delimiter must not be empty",
                    ));
                }
                opts.delimiter = d.as_bytes()[0];
            }
            if let Some(ref q) = js_opts.quote {
                if q.is_empty() {
                    return Err(napi::Error::new(
                        napi::Status::InvalidArg,
                        "quote must not be empty",
                    ));
                }
                opts.quote = q.as_bytes()[0];
            }
            if let Some(ref le) = js_opts.line_ending {
                opts.line_ending = le.clone();
            }
            if let Some(ref df) = js_opts.date_format {
                opts.date_format = df.clone();
            }
        }
        Ok(ws.to_csv_string(&opts))
    }
}

// ── Private helpers ──

impl Worksheet {
    fn lock(&self) -> napi::Result<std::sync::MutexGuard<'_, zavora_xlsx::Workbook>> {
        self.workbook.lock().map_err(|_| {
            napi::Error::new(napi::Status::GenericFailure, "lock poisoned")
        })
    }
}

fn cell_value_to_js(env: &Env, value: &zavora_xlsx::CellValue) -> napi::Result<JsUnknown> {
    match value {
        zavora_xlsx::CellValue::Empty => Ok(env.get_null()?.into_unknown()),
        zavora_xlsx::CellValue::String(s) => Ok(env.create_string(s)?.into_unknown()),
        zavora_xlsx::CellValue::Number(n) => Ok(env.create_double(*n)?.into_unknown()),
        zavora_xlsx::CellValue::Bool(b) => Ok(env.get_boolean(*b)?.into_unknown()),
        zavora_xlsx::CellValue::DateTime(dt) => Ok(env.create_double(dt.serial())?.into_unknown()),
        zavora_xlsx::CellValue::Error(e) => Ok(env.create_string(e)?.into_unknown()),
        zavora_xlsx::CellValue::RichText(rt) => {
            Ok(env.create_string(&rt.plain_text())?.into_unknown())
        }
        zavora_xlsx::CellValue::Formula {
            formula,
            cached_value,
        } => {
            let mut obj = env.create_object()?;
            obj.set_named_property("formula", env.create_string(formula)?)?;
            let cached_js = cell_value_to_js(env, cached_value)?;
            obj.set_named_property("cachedValue", cached_js)?;
            Ok(obj.into_unknown())
        }
    }
}
