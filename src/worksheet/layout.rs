// Layout: widths, heights, freeze, autofit, hidden, grouping, view settings, print

use super::Worksheet;
use super::types::{Orientation, PrintSettings, SheetVisibility};
use crate::cell::CellType;
use crate::format::{Format, IntoColor};
use crate::utility::{ColNum, RowNum};
use std::collections::BTreeMap;

impl Worksheet {
    pub fn set_column_width(&mut self, col: ColNum, width: f64) -> crate::Result<&mut Self> {
        self.col_widths.insert(col, width);
        self.dirty = true;
        Ok(self)
    }
    pub fn set_row_height(&mut self, row: RowNum, height: f64) -> crate::Result<&mut Self> {
        self.row_heights.insert(row, height);
        self.dirty = true;
        Ok(self)
    }
    pub fn set_freeze_panes(&mut self, row: RowNum, col: ColNum) -> crate::Result<&mut Self> {
        self.freeze_row = row;
        self.freeze_col = col;
        self.dirty = true;
        Ok(self)
    }

    pub fn autofit(&mut self) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        let mut max_widths: BTreeMap<ColNum, f64> = BTreeMap::new();
        for cols in self.cells.values() {
            for (&col, (cell, _)) in cols {
                let len = match cell {
                    CellType::Number(n) => format!("{n}").len(),
                    CellType::InlineString(s) => s.len(),
                    CellType::SharedString(_) => 8,
                    CellType::Bool(_) => 5,
                    CellType::Formula { text, .. }
                    | CellType::ArrayFormula { text, .. }
                    | CellType::DynamicFormula { text, .. } => text.len().min(20),
                    CellType::DateTime(_) => 10,
                    CellType::Error(e) => e.len(),
                    CellType::RichText(rt) => rt.plain_text().len(),
                    CellType::Empty => 0,
                };
                let width = (len as f64 * 1.1 + 2.0).clamp(8.0, 64.0);
                let entry = max_widths.entry(col).or_insert(8.0);
                if width > *entry {
                    *entry = width;
                }
            }
        }
        for (col, w) in max_widths {
            self.col_widths.insert(col, w);
        }
        self.dirty = true;
        Ok(self)
    }

    pub fn set_print_settings(&mut self, settings: &PrintSettings) -> &mut Self {
        self.print_settings = Some(settings.clone());
        self.dirty = true;
        self
    }

    pub fn set_page_breaks(&mut self, row_breaks: &[RowNum], col_breaks: &[ColNum]) -> &mut Self {
        let ps = self
            .print_settings
            .get_or_insert_with(PrintSettings::default);
        ps.row_breaks = row_breaks.to_vec();
        ps.col_breaks = col_breaks.to_vec();
        self.dirty = true;
        self
    }

    pub fn set_row_hidden(&mut self, row: RowNum, hidden: bool) -> &mut Self {
        if hidden {
            self.hidden_rows.insert(row);
        } else {
            self.hidden_rows.remove(&row);
        }
        self.dirty = true;
        self
    }

    pub fn set_column_hidden(&mut self, col: ColNum, hidden: bool) -> &mut Self {
        if hidden {
            self.hidden_cols.insert(col);
        } else {
            self.hidden_cols.remove(&col);
        }
        self.dirty = true;
        self
    }

    pub fn set_autofilter(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum) -> &mut Self {
        self.autofilter = Some((r1, c1, r2, c2));
        self.dirty = true;
        self
    }

    pub fn group_rows(&mut self, start: RowNum, end: RowNum, level: u8) -> &mut Self {
        for r in start..=end {
            self.row_outline_levels.insert(r, level);
        }
        self.dirty = true;
        self
    }

    pub fn group_columns(&mut self, start: ColNum, end: ColNum, level: u8) -> &mut Self {
        for c in start..=end {
            self.col_outline_levels.insert(c, level);
        }
        self.dirty = true;
        self
    }

    pub fn set_print_area(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum) -> &mut Self {
        let ps = self
            .print_settings
            .get_or_insert_with(PrintSettings::default);
        ps.print_area = Some((r1, c1, r2, c2));
        self.dirty = true;
        self
    }

    pub fn set_repeat_rows(&mut self, first: RowNum, last: RowNum) -> &mut Self {
        let ps = self
            .print_settings
            .get_or_insert_with(PrintSettings::default);
        ps.repeat_rows = Some((first, last));
        self.dirty = true;
        self
    }

    pub fn set_repeat_columns(&mut self, first: ColNum, last: ColNum) -> &mut Self {
        let ps = self
            .print_settings
            .get_or_insert_with(PrintSettings::default);
        ps.repeat_cols = Some((first, last));
        self.dirty = true;
        self
    }

    pub fn set_print_scale(&mut self, percent: u16) -> &mut Self {
        let ps = self
            .print_settings
            .get_or_insert_with(PrintSettings::default);
        ps.scale = Some(percent.clamp(10, 400));
        self.dirty = true;
        self
    }

    // View settings
    pub fn set_zoom(&mut self, percent: u16) -> &mut Self {
        self.zoom = Some(percent.clamp(10, 400));
        self
    }
    pub fn hide_gridlines(&mut self) -> &mut Self {
        self.show_gridlines = false;
        self
    }
    pub fn hide_headings(&mut self) -> &mut Self {
        self.show_headings = false;
        self
    }
    pub fn set_right_to_left(&mut self) -> &mut Self {
        self.right_to_left = true;
        self
    }
    pub fn set_tab_color(&mut self, c: impl IntoColor) -> &mut Self {
        self.tab_color = Some(c.into_color().to_rgb());
        self
    }
    pub fn set_default_row_height(&mut self, height: f64) -> &mut Self {
        self.default_row_height = Some(height);
        self.dirty = true;
        self
    }
    pub fn set_column_format(&mut self, col: ColNum, format: &Format) -> &mut Self {
        self.col_formats.insert(col, format.clone());
        self.dirty = true;
        self
    }
    pub fn set_row_format(&mut self, row: RowNum, format: &Format) -> &mut Self {
        self.row_formats.insert(row, format.clone());
        self.dirty = true;
        self
    }
    pub fn set_hidden(&mut self) -> &mut Self {
        self.visibility = SheetVisibility::Hidden;
        self
    }
    pub fn set_very_hidden(&mut self) -> &mut Self {
        self.visibility = SheetVisibility::VeryHidden;
        self
    }
    pub fn set_selection(&mut self, row: RowNum, col: ColNum) -> &mut Self {
        self.selection = Some((row, col));
        self
    }
    pub fn set_top_left_cell(&mut self, row: RowNum, col: ColNum) -> &mut Self {
        self.top_left_cell = Some((row, col));
        self
    }
    pub fn ignore_error(&mut self, error_type: &str, range: &str) -> &mut Self {
        self.ignored_errors
            .push((error_type.to_string(), range.to_string()));
        self.dirty = true;
        self
    }
    pub fn filter_column(&mut self, col: ColNum, values: &[&str]) -> &mut Self {
        self.autofilter_columns
            .push((col, values.iter().map(|s| s.to_string()).collect()));
        self.dirty = true;
        self
    }

    /// Set a sort state on the worksheet.
    ///
    /// The sort applies to the autofilter range (or the specified range if autofilter is set).
    /// `col` is the 0-based column index, `direction` is ascending or descending.
    pub fn set_sort(&mut self, col: ColNum, direction: super::types::SortDirection) -> &mut Self {
        use super::types::{SortCondition, SortState};
        if let Some(range) = self.autofilter {
            match &mut self.sort_state {
                Some(ss) => {
                    ss.conditions.push(SortCondition { col, direction });
                }
                None => {
                    self.sort_state = Some(SortState {
                        range,
                        conditions: vec![SortCondition { col, direction }],
                    });
                }
            }
        } else {
            // If no autofilter, create a sort state with a default range
            match &mut self.sort_state {
                Some(ss) => {
                    ss.conditions.push(SortCondition { col, direction });
                }
                None => {
                    self.sort_state = Some(SortState {
                        range: (0, 0, 0, col),
                        conditions: vec![SortCondition { col, direction }],
                    });
                }
            }
        }
        self.dirty = true;
        self
    }

    /// Add an advanced filter rule to a column.
    ///
    /// This supports Top10, DateFilter, and CustomFilter rules.
    pub fn filter_column_advanced(
        &mut self,
        col: ColNum,
        rule: super::types::FilterRule,
    ) -> &mut Self {
        self.advanced_filter_columns
            .push(super::types::AdvancedFilterColumn { col, rule });
        self.dirty = true;
        self
    }

    /// Set width for a range of columns.
    pub fn set_column_range_width(&mut self, first: ColNum, last: ColNum, width: f64) -> &mut Self {
        for c in first..=last {
            self.col_widths.insert(c, width);
        }
        self.dirty = true;
        self
    }

    /// Hide a range of columns.
    pub fn set_column_range_hidden(&mut self, first: ColNum, last: ColNum) -> &mut Self {
        for c in first..=last {
            self.hidden_cols.insert(c);
        }
        self.dirty = true;
        self
    }

    // ── Page setup convenience methods ──

    pub fn set_landscape(&mut self) -> &mut Self {
        self.print_settings
            .get_or_insert_with(PrintSettings::default)
            .orientation = Some(Orientation::Landscape);
        self.dirty = true;
        self
    }
    pub fn set_portrait(&mut self) -> &mut Self {
        self.print_settings
            .get_or_insert_with(PrintSettings::default)
            .orientation = Some(Orientation::Portrait);
        self.dirty = true;
        self
    }
    pub fn set_paper_size(&mut self, size: u8) -> &mut Self {
        self.print_settings
            .get_or_insert_with(PrintSettings::default)
            .paper_size = Some(size);
        self.dirty = true;
        self
    }
    pub fn set_margins(&mut self, top: f64, bottom: f64, left: f64, right: f64) -> &mut Self {
        let ps = self
            .print_settings
            .get_or_insert_with(PrintSettings::default);
        ps.margin_top = Some(top);
        ps.margin_bottom = Some(bottom);
        ps.margin_left = Some(left);
        ps.margin_right = Some(right);
        self.dirty = true;
        self
    }
    pub fn set_fit_to_page(&mut self, width: u16, height: u16) -> &mut Self {
        let ps = self
            .print_settings
            .get_or_insert_with(PrintSettings::default);
        ps.fit_to_page = true;
        ps.fit_to_width = Some(width);
        ps.fit_to_height = Some(height);
        self.dirty = true;
        self
    }
    pub fn set_header(&mut self, header: &str) -> &mut Self {
        self.print_settings
            .get_or_insert_with(PrintSettings::default)
            .header = Some(header.into());
        self.dirty = true;
        self
    }
    pub fn set_footer(&mut self, footer: &str) -> &mut Self {
        self.print_settings
            .get_or_insert_with(PrintSettings::default)
            .footer = Some(footer.into());
        self.dirty = true;
        self
    }
    pub fn set_header_center(&mut self, text: &str) -> &mut Self {
        self.set_header(&format!("&C{text}"))
    }
    pub fn set_header_left(&mut self, text: &str) -> &mut Self {
        self.set_header(&format!("&L{text}"))
    }
    pub fn set_header_right(&mut self, text: &str) -> &mut Self {
        self.set_header(&format!("&R{text}"))
    }
    pub fn set_footer_center(&mut self, text: &str) -> &mut Self {
        self.set_footer(&format!("&C{text}"))
    }
    pub fn set_footer_left(&mut self, text: &str) -> &mut Self {
        self.set_footer(&format!("&L{text}"))
    }
    pub fn set_footer_right(&mut self, text: &str) -> &mut Self {
        self.set_footer(&format!("&R{text}"))
    }

    // ── CSV export ──

    /// Write sheet data as CSV to any writer.
    pub fn to_csv<W: std::io::Write>(&self, writer: &mut W, delimiter: u8) -> crate::Result<()> {
        let range = match self.used_range() {
            Some(r) => r,
            None => return Ok(()),
        };
        for r in range.0..=range.2 {
            let mut first = true;
            for c in range.1..=range.3 {
                if !first {
                    writer.write_all(&[delimiter])?;
                }
                first = false;
                let val = self.read_cell(r, c);
                let s = match &val {
                    crate::cell::CellValue::Empty => String::new(),
                    crate::cell::CellValue::String(s) => {
                        if s.contains(delimiter as char) || s.contains('"') || s.contains('\n') {
                            format!("\"{}\"", s.replace('"', "\"\""))
                        } else {
                            s.clone()
                        }
                    }
                    crate::cell::CellValue::Number(n) => format!("{n}"),
                    crate::cell::CellValue::Bool(b) => {
                        if *b {
                            "TRUE".into()
                        } else {
                            "FALSE".into()
                        }
                    }
                    crate::cell::CellValue::DateTime(dt) => dt.to_iso_string(),
                    crate::cell::CellValue::Error(e) => e.clone(),
                    crate::cell::CellValue::Formula { cached_value, .. } => {
                        match cached_value.as_ref() {
                            crate::cell::CellValue::Number(n) => format!("{n}"),
                            crate::cell::CellValue::String(s) => s.clone(),
                            _ => String::new(),
                        }
                    }
                    crate::cell::CellValue::RichText(rt) => rt.plain_text(),
                };
                writer.write_all(s.as_bytes())?;
            }
            writer.write_all(b"\r\n")?;
        }
        Ok(())
    }
}
