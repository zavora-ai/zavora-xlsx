// Write operations: write, write_formula, write_rich_text, write_blank, clear_cell, internal writers

use super::Worksheet;
use crate::cell::{CellType, IntoExcelData, RichText};
use crate::datetime::ExcelDateTime;
use crate::format::Format;
use crate::utility::{ColNum, RowNum};

impl Worksheet {
    pub fn write(
        &mut self,
        row: RowNum,
        col: ColNum,
        data: impl IntoExcelData,
    ) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        data.write_cell(self, row, col)?;
        Ok(self)
    }

    pub fn write_with_format(
        &mut self,
        row: RowNum,
        col: ColNum,
        data: impl IntoExcelData,
        fmt: &Format,
    ) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        data.write_cell_with_format(self, row, col, fmt)?;
        Ok(self)
    }

    pub fn write_row(
        &mut self,
        row: RowNum,
        start_col: ColNum,
        data: impl IntoIterator<Item = impl IntoExcelData>,
    ) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        for (i, val) in data.into_iter().enumerate() {
            val.write_cell(self, row, start_col + i as ColNum)?;
        }
        Ok(self)
    }

    pub fn write_column(
        &mut self,
        start_row: RowNum,
        col: ColNum,
        data: impl IntoIterator<Item = impl IntoExcelData>,
    ) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        for (i, val) in data.into_iter().enumerate() {
            val.write_cell(self, start_row + i as RowNum, col)?;
        }
        Ok(self)
    }

    pub fn write_formula(
        &mut self,
        row: RowNum,
        col: ColNum,
        formula: &str,
    ) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        let text = formula.strip_prefix('=').unwrap_or(formula);
        self.cells.entry(row).or_default().insert(
            col,
            (
                CellType::Formula {
                    text: text.to_string(),
                    cached_number: None,
                },
                0,
            ),
        );
        Ok(self)
    }

    pub fn write_formula_with_result(
        &mut self,
        row: RowNum,
        col: ColNum,
        formula: &str,
        result: f64,
    ) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        let text = formula.strip_prefix('=').unwrap_or(formula);
        self.cells.entry(row).or_default().insert(
            col,
            (
                CellType::Formula {
                    text: text.to_string(),
                    cached_number: Some(result),
                },
                0,
            ),
        );
        Ok(self)
    }

    pub fn write_array_formula(
        &mut self,
        r1: RowNum,
        c1: ColNum,
        r2: RowNum,
        c2: ColNum,
        formula: &str,
    ) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        let text = formula.strip_prefix('=').unwrap_or(formula);
        let range = format!(
            "{}{}:{}{}",
            crate::utility::col_to_letter(c1),
            r1 + 1,
            crate::utility::col_to_letter(c2),
            r2 + 1
        );
        self.cells.entry(r1).or_default().insert(
            c1,
            (
                CellType::ArrayFormula {
                    text: text.to_string(),
                    range,
                },
                0,
            ),
        );
        Ok(self)
    }

    pub fn write_dynamic_formula(
        &mut self,
        row: RowNum,
        col: ColNum,
        formula: &str,
    ) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        let text = formula.strip_prefix('=').unwrap_or(formula);
        let range = format!("{}{}", crate::utility::col_to_letter(col), row + 1);
        self.cells.entry(row).or_default().insert(
            col,
            (
                CellType::DynamicFormula {
                    text: text.to_string(),
                    range,
                },
                0,
            ),
        );
        Ok(self)
    }

    pub fn write_rich_text(
        &mut self,
        row: RowNum,
        col: ColNum,
        rich_text: &RichText,
    ) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        self.cells
            .entry(row)
            .or_default()
            .insert(col, (CellType::RichText(rich_text.clone()), 0));
        Ok(self)
    }

    pub fn write_blank(
        &mut self,
        row: RowNum,
        col: ColNum,
        format: &Format,
    ) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        self.cells
            .entry(row)
            .or_default()
            .insert(col, (CellType::Empty, 0));
        self.pending_formats.insert((row, col), format.clone());
        Ok(self)
    }

    pub fn clear_cell(&mut self, row: RowNum, col: ColNum) -> &mut Self {
        self.ensure_deserialized();
        self.dirty = true;
        if let Some(cols) = self.cells.get_mut(&row) {
            cols.remove(&col);
        }
        self.pending_formats.remove(&(row, col));
        self
    }

    // Internal write methods called by IntoExcelData impls
    pub(crate) fn write_number_internal(
        &mut self,
        row: RowNum,
        col: ColNum,
        n: f64,
        fmt: Option<&Format>,
    ) -> crate::Result<()> {
        self.cells
            .entry(row)
            .or_default()
            .insert(col, (CellType::Number(n), 0));
        if let Some(f) = fmt {
            self.pending_formats.insert((row, col), f.clone());
        }
        Ok(())
    }

    pub(crate) fn write_string_internal(
        &mut self,
        row: RowNum,
        col: ColNum,
        s: &str,
        fmt: Option<&Format>,
    ) -> crate::Result<()> {
        let cell = CellType::InlineString(s.to_string());
        self.cells.entry(row).or_default().insert(col, (cell, 0));
        if let Some(f) = fmt {
            self.pending_formats.insert((row, col), f.clone());
        }
        Ok(())
    }

    pub(crate) fn write_bool_internal(
        &mut self,
        row: RowNum,
        col: ColNum,
        b: bool,
        fmt: Option<&Format>,
    ) -> crate::Result<()> {
        self.cells
            .entry(row)
            .or_default()
            .insert(col, (CellType::Bool(b), 0));
        if let Some(f) = fmt {
            self.pending_formats.insert((row, col), f.clone());
        }
        Ok(())
    }

    pub(crate) fn write_datetime_internal(
        &mut self,
        row: RowNum,
        col: ColNum,
        dt: ExcelDateTime,
        fmt: Option<&Format>,
    ) -> crate::Result<()> {
        self.cells
            .entry(row)
            .or_default()
            .insert(col, (CellType::DateTime(dt.serial()), 0));
        if let Some(f) = fmt {
            self.pending_formats.insert((row, col), f.clone());
        }
        Ok(())
    }
}
