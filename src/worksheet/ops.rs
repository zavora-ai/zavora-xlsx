// Row/column insert/remove operations with formula adjustment

use super::Worksheet;
use crate::cell::CellType;
use crate::formula::adjust_formula;
use crate::utility::{ColNum, RowNum};
use std::collections::BTreeMap;

impl Worksheet {
    pub fn insert_rows(&mut self, at_row: RowNum, count: u32) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        let mut new_cells = BTreeMap::new();
        for (&r, cols) in &self.cells {
            let new_r = if r >= at_row { r + count } else { r };
            new_cells.insert(new_r, cols.clone());
        }
        self.cells = new_cells;
        for m in &mut self.merge_ranges {
            if m.0 >= at_row {
                m.0 += count;
            }
            if m.2 >= at_row {
                m.2 += count;
            }
        }
        let mut new_heights = BTreeMap::new();
        for (&r, &h) in &self.row_heights {
            let new_r = if r >= at_row { r + count } else { r };
            new_heights.insert(new_r, h);
        }
        self.row_heights = new_heights;
        self.adjust_formulas_for_row_insert(at_row, count as i64);
        Ok(self)
    }

    pub fn remove_rows(&mut self, at_row: RowNum, count: u32) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        let end_row = at_row + count;
        let mut new_cells = BTreeMap::new();
        for (&r, cols) in &self.cells {
            if r >= at_row && r < end_row {
                continue;
            }
            let new_r = if r >= end_row { r - count } else { r };
            new_cells.insert(new_r, cols.clone());
        }
        self.cells = new_cells;
        self.merge_ranges
            .retain(|m| !(m.0 >= at_row && m.2 < end_row));
        for m in &mut self.merge_ranges {
            if m.0 >= end_row {
                m.0 -= count;
            }
            if m.2 >= end_row {
                m.2 -= count;
            }
        }
        let mut new_heights = BTreeMap::new();
        for (&r, &h) in &self.row_heights {
            if r >= at_row && r < end_row {
                continue;
            }
            let new_r = if r >= end_row { r - count } else { r };
            new_heights.insert(new_r, h);
        }
        self.row_heights = new_heights;
        self.adjust_formulas_for_row_insert(at_row, -(count as i64));
        Ok(self)
    }

    pub fn insert_columns(&mut self, at_col: ColNum, count: u16) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        for cols in self.cells.values_mut() {
            let mut new_cols = BTreeMap::new();
            for (&c, v) in cols.iter() {
                let new_c = if c >= at_col { c + count } else { c };
                new_cols.insert(new_c, v.clone());
            }
            *cols = new_cols;
        }
        for m in &mut self.merge_ranges {
            if m.1 >= at_col {
                m.1 += count;
            }
            if m.3 >= at_col {
                m.3 += count;
            }
        }
        let mut new_widths = BTreeMap::new();
        for (&c, &w) in &self.col_widths {
            let new_c = if c >= at_col { c + count } else { c };
            new_widths.insert(new_c, w);
        }
        self.col_widths = new_widths;
        self.adjust_formulas_for_col_insert(at_col, count as i64);
        Ok(self)
    }

    pub fn remove_columns(&mut self, at_col: ColNum, count: u16) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        let end_col = at_col + count;
        for cols in self.cells.values_mut() {
            let mut new_cols = BTreeMap::new();
            for (&c, v) in cols.iter() {
                if c >= at_col && c < end_col {
                    continue;
                }
                let new_c = if c >= end_col { c - count } else { c };
                new_cols.insert(new_c, v.clone());
            }
            *cols = new_cols;
        }
        self.merge_ranges
            .retain(|m| !(m.1 >= at_col && m.3 < end_col));
        for m in &mut self.merge_ranges {
            if m.1 >= end_col {
                m.1 -= count;
            }
            if m.3 >= end_col {
                m.3 -= count;
            }
        }
        let mut new_widths = BTreeMap::new();
        for (&c, &w) in &self.col_widths {
            if c >= at_col && c < end_col {
                continue;
            }
            let new_c = if c >= end_col { c - count } else { c };
            new_widths.insert(new_c, w);
        }
        self.col_widths = new_widths;
        self.adjust_formulas_for_col_insert(at_col, -(count as i64));
        Ok(self)
    }

    fn adjust_formulas_for_row_insert(&mut self, at_row: RowNum, delta: i64) {
        for cols in self.cells.values_mut() {
            for (cell, _) in cols.values_mut() {
                if let CellType::Formula { text, .. } = cell {
                    *text = adjust_formula(text, Some(at_row), delta, None, 0);
                }
            }
        }
    }

    fn adjust_formulas_for_col_insert(&mut self, at_col: ColNum, delta: i64) {
        for cols in self.cells.values_mut() {
            for (cell, _) in cols.values_mut() {
                if let CellType::Formula { text, .. } = cell {
                    *text = adjust_formula(text, None, 0, Some(at_col), delta);
                }
            }
        }
    }
}
