// Types: SheetProtection, PrintSettings, SheetVisibility, Orientation, Hyperlink, Comment, helpers

use crate::utility::{ColNum, RowNum};

/// Sheet protection settings.
#[derive(Debug, Clone)]
pub struct SheetProtection {
    pub(crate) password_hash: Option<String>,
    pub sheet: bool,
    pub objects: bool,
    pub scenarios: bool,
    pub format_cells: bool,
    pub format_columns: bool,
    pub format_rows: bool,
    pub insert_columns: bool,
    pub insert_rows: bool,
    pub insert_hyperlinks: bool,
    pub delete_columns: bool,
    pub delete_rows: bool,
    pub select_locked_cells: bool,
    pub sort: bool,
    pub auto_filter: bool,
    pub pivot_tables: bool,
    pub select_unlocked_cells: bool,
}

impl Default for SheetProtection {
    fn default() -> Self {
        Self {
            password_hash: None,
            sheet: true,
            objects: true,
            scenarios: true,
            format_cells: true,
            format_columns: true,
            format_rows: true,
            insert_columns: true,
            insert_rows: true,
            insert_hyperlinks: true,
            delete_columns: true,
            delete_rows: true,
            select_locked_cells: false,
            sort: true,
            auto_filter: true,
            pivot_tables: true,
            select_unlocked_cells: false,
        }
    }
}

impl SheetProtection {
    /// Returns the legacy password hash, if set.
    pub fn password_hash(&self) -> Option<&str> {
        self.password_hash.as_deref()
    }

    /// Set the `sheet` protection flag (default: true).
    pub fn set_sheet(&mut self, v: bool) -> &mut Self {
        self.sheet = v;
        self
    }

    /// Set the `objects` protection flag (default: true).
    pub fn set_objects(&mut self, v: bool) -> &mut Self {
        self.objects = v;
        self
    }

    /// Set the `scenarios` protection flag (default: true).
    pub fn set_scenarios(&mut self, v: bool) -> &mut Self {
        self.scenarios = v;
        self
    }

    /// Allow or disallow formatting cells (default: true = protected).
    pub fn set_format_cells(&mut self, v: bool) -> &mut Self {
        self.format_cells = v;
        self
    }

    /// Allow or disallow formatting columns (default: true = protected).
    pub fn set_format_columns(&mut self, v: bool) -> &mut Self {
        self.format_columns = v;
        self
    }

    /// Allow or disallow formatting rows (default: true = protected).
    pub fn set_format_rows(&mut self, v: bool) -> &mut Self {
        self.format_rows = v;
        self
    }

    /// Allow or disallow inserting columns (default: true = protected).
    pub fn set_insert_columns(&mut self, v: bool) -> &mut Self {
        self.insert_columns = v;
        self
    }

    /// Allow or disallow inserting rows (default: true = protected).
    pub fn set_insert_rows(&mut self, v: bool) -> &mut Self {
        self.insert_rows = v;
        self
    }

    /// Allow or disallow inserting hyperlinks (default: true = protected).
    pub fn set_insert_hyperlinks(&mut self, v: bool) -> &mut Self {
        self.insert_hyperlinks = v;
        self
    }

    /// Allow or disallow deleting columns (default: true = protected).
    pub fn set_delete_columns(&mut self, v: bool) -> &mut Self {
        self.delete_columns = v;
        self
    }

    /// Allow or disallow deleting rows (default: true = protected).
    pub fn set_delete_rows(&mut self, v: bool) -> &mut Self {
        self.delete_rows = v;
        self
    }

    /// Set whether selecting locked cells is prevented (default: false).
    pub fn set_select_locked_cells(&mut self, v: bool) -> &mut Self {
        self.select_locked_cells = v;
        self
    }

    /// Allow or disallow sorting (default: true = protected).
    pub fn set_sort(&mut self, v: bool) -> &mut Self {
        self.sort = v;
        self
    }

    /// Allow or disallow autofilter (default: true = protected).
    pub fn set_auto_filter(&mut self, v: bool) -> &mut Self {
        self.auto_filter = v;
        self
    }

    /// Allow or disallow pivot tables (default: true = protected).
    pub fn set_pivot_tables(&mut self, v: bool) -> &mut Self {
        self.pivot_tables = v;
        self
    }

    /// Set whether selecting unlocked cells is prevented (default: false).
    pub fn set_select_unlocked_cells(&mut self, v: bool) -> &mut Self {
        self.select_unlocked_cells = v;
        self
    }
}

/// Print settings for a worksheet.
#[derive(Debug, Clone, Default)]
pub struct PrintSettings {
    pub paper_size: Option<u8>,
    pub orientation: Option<Orientation>,
    pub fit_to_page: bool,
    pub fit_to_width: Option<u16>,
    pub fit_to_height: Option<u16>,
    pub scale: Option<u16>,
    pub margin_top: Option<f64>,
    pub margin_bottom: Option<f64>,
    pub margin_left: Option<f64>,
    pub margin_right: Option<f64>,
    pub margin_header: Option<f64>,
    pub margin_footer: Option<f64>,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub row_breaks: Vec<RowNum>,
    pub col_breaks: Vec<ColNum>,
    pub print_area: Option<(RowNum, ColNum, RowNum, ColNum)>,
    pub repeat_rows: Option<(RowNum, RowNum)>,
    pub repeat_cols: Option<(ColNum, ColNum)>,
    pub print_gridlines: bool,
    pub print_headings: bool,
    pub center_horizontally: bool,
    pub center_vertically: bool,
    pub black_and_white: bool,
    pub first_page_number: Option<u16>,
    pub(crate) protected_ranges: Vec<(String, String, Option<String>)>,
}

impl PrintSettings {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn paper_size(mut self, size: u8) -> Self {
        self.paper_size = Some(size);
        self
    }
    pub fn orientation(mut self, o: Orientation) -> Self {
        self.orientation = Some(o);
        self
    }
    pub fn fit_to_page(mut self, width: u16, height: u16) -> Self {
        self.fit_to_page = true;
        self.fit_to_width = Some(width);
        self.fit_to_height = Some(height);
        self
    }
    pub fn margins(mut self, top: f64, bottom: f64, left: f64, right: f64) -> Self {
        self.margin_top = Some(top);
        self.margin_bottom = Some(bottom);
        self.margin_left = Some(left);
        self.margin_right = Some(right);
        self
    }
    pub fn header(mut self, h: &str) -> Self {
        self.header = Some(h.to_string());
        self
    }
    pub fn footer(mut self, f: &str) -> Self {
        self.footer = Some(f.to_string());
        self
    }
    pub fn print_gridlines(mut self, v: bool) -> Self {
        self.print_gridlines = v;
        self
    }
    pub fn print_headings(mut self, v: bool) -> Self {
        self.print_headings = v;
        self
    }
    pub fn center_horizontally(mut self, v: bool) -> Self {
        self.center_horizontally = v;
        self
    }
    pub fn center_vertically(mut self, v: bool) -> Self {
        self.center_vertically = v;
        self
    }
    pub fn black_and_white(mut self, v: bool) -> Self {
        self.black_and_white = v;
        self
    }
    pub fn first_page_number(mut self, n: u16) -> Self {
        self.first_page_number = Some(n);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetVisibility {
    #[default]
    Visible,
    Hidden,
    VeryHidden,
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// A hyperlink on a cell.
#[derive(Debug, Clone)]
pub struct Hyperlink {
    pub row: RowNum,
    pub col: ColNum,
    pub url: String,
    pub location: Option<String>,
    pub tooltip: Option<String>,
}

/// A comment/note on a cell.
#[derive(Debug, Clone)]
pub struct Comment {
    pub row: RowNum,
    pub col: ColNum,
    pub text: String,
    pub author: String,
}

/// A threaded comment (modern comment) on a cell.
#[derive(Debug, Clone)]
pub struct ThreadedComment {
    pub row: RowNum,
    pub col: ColNum,
    pub author: String,
    pub text: String,
    pub timestamp: String,
    pub replies: Vec<ThreadedCommentReply>,
}

/// A reply within a threaded comment.
#[derive(Debug, Clone)]
pub struct ThreadedCommentReply {
    pub author: String,
    pub text: String,
    pub timestamp: String,
}

impl ThreadedComment {
    /// Create a new threaded comment.
    pub fn new(author: &str, text: &str) -> Self {
        Self {
            row: 0,
            col: 0,
            author: author.to_string(),
            text: text.to_string(),
            timestamp: "2024-01-01T00:00:00.000".to_string(),
            replies: Vec::new(),
        }
    }

    /// Set the timestamp (ISO 8601 format).
    pub fn timestamp(mut self, ts: &str) -> Self {
        self.timestamp = ts.to_string();
        self
    }

    /// Add a reply to this threaded comment.
    pub fn add_reply(&mut self, author: &str, text: &str) -> &mut Self {
        self.replies.push(ThreadedCommentReply {
            author: author.to_string(),
            text: text.to_string(),
            timestamp: "2024-01-01T00:00:00.000".to_string(),
        });
        self
    }

    /// Add a reply with a specific timestamp.
    pub fn add_reply_with_timestamp(
        &mut self,
        author: &str,
        text: &str,
        timestamp: &str,
    ) -> &mut Self {
        self.replies.push(ThreadedCommentReply {
            author: author.to_string(),
            text: text.to_string(),
            timestamp: timestamp.to_string(),
        });
        self
    }
}

/// Form control types for worksheet interactivity (Task 70).
#[derive(Debug, Clone)]
pub enum FormControl {
    Checkbox {
        text: String,
        checked: bool,
        cell_link: Option<String>,
    },
    Dropdown {
        items: Vec<String>,
        selected_index: Option<usize>,
        cell_link: Option<String>,
    },
    Button {
        text: String,
        macro_name: Option<String>,
    },
    Spinner {
        min_value: i32,
        max_value: i32,
        current_value: i32,
        increment: i32,
        cell_link: Option<String>,
    },
}

impl FormControl {
    /// Create a checkbox form control.
    pub fn checkbox(text: &str) -> Self {
        FormControl::Checkbox {
            text: text.to_string(),
            checked: false,
            cell_link: None,
        }
    }

    /// Create a checkbox with a cell link.
    pub fn checkbox_with_link(text: &str, cell_link: &str) -> Self {
        FormControl::Checkbox {
            text: text.to_string(),
            checked: false,
            cell_link: Some(cell_link.to_string()),
        }
    }

    /// Create a dropdown form control.
    pub fn dropdown(items: Vec<String>) -> Self {
        FormControl::Dropdown {
            items,
            selected_index: None,
            cell_link: None,
        }
    }

    /// Create a button form control.
    pub fn button(text: &str) -> Self {
        FormControl::Button {
            text: text.to_string(),
            macro_name: None,
        }
    }

    /// Create a spinner form control.
    pub fn spinner(min: i32, max: i32, current: i32) -> Self {
        FormControl::Spinner {
            min_value: min,
            max_value: max,
            current_value: current,
            increment: 1,
            cell_link: None,
        }
    }

    /// Create a spinner with a cell link.
    pub fn spinner_with_link(min: i32, max: i32, current: i32, cell_link: &str) -> Self {
        FormControl::Spinner {
            min_value: min,
            max_value: max,
            current_value: current,
            increment: 1,
            cell_link: Some(cell_link.to_string()),
        }
    }
}

/// A phonetic text run (furigana) for East Asian text pronunciation.
#[derive(Debug, Clone)]
pub struct PhoneticRun {
    /// Start character index in the base text.
    pub start_index: u32,
    /// End character index in the base text.
    pub end_index: u32,
    /// The phonetic (pronunciation) text.
    pub text: String,
}

impl PhoneticRun {
    pub fn new(start_index: u32, end_index: u32, text: &str) -> Self {
        Self {
            start_index,
            end_index,
            text: text.to_string(),
        }
    }
}

/// Sort direction for worksheet sort state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// A sort condition within a sort state.
#[derive(Debug, Clone)]
pub struct SortCondition {
    pub col: ColNum,
    pub direction: SortDirection,
}

/// Sort state for a worksheet.
#[derive(Debug, Clone)]
pub struct SortState {
    pub range: (RowNum, ColNum, RowNum, ColNum),
    pub conditions: Vec<SortCondition>,
}

/// Filter rule for advanced autofilter.
#[derive(Debug, Clone)]
pub enum FilterRule {
    /// Top N or Bottom N filter.
    Top10 { top: bool, percent: bool, val: f64 },
    /// Date group filter.
    DateFilter {
        year: u16,
        month: Option<u8>,
        day: Option<u8>,
    },
    /// Custom filter with one or two conditions.
    CustomFilter {
        and: bool,
        conditions: Vec<(String, String)>, // (operator, value) pairs
    },
}

/// Advanced filter column entry.
#[derive(Debug, Clone)]
pub struct AdvancedFilterColumn {
    pub col: ColNum,
    pub rule: FilterRule,
}

/// Excel legacy password hash (XOR-based, 16-bit).
pub(crate) fn hash_password(password: &str) -> String {
    let bytes = password.as_bytes();
    let mut hash: u16 = 0;
    for (i, &b) in bytes.iter().rev().enumerate() {
        let mut val = b as u16;
        val ^= (i + 1) as u16;
        val = ((val >> 14) & 1) | ((val << 1) & 0x7FFF);
        hash ^= val;
    }
    hash ^= bytes.len() as u16;
    hash ^= 0xCE4B;
    format!("{hash:04X}")
}

pub(crate) fn hash_password_public(password: &str) -> String {
    hash_password(password)
}

/// Validate an Excel sheet name.
pub(crate) fn validate_sheet_name(name: &str) -> crate::Result<()> {
    if name.is_empty() {
        return Err(crate::Error::InvalidData(
            "Sheet name cannot be empty".into(),
        ));
    }
    if name.len() > 31 {
        return Err(crate::Error::InvalidData(format!(
            "Sheet name '{}' exceeds 31 characters",
            name
        )));
    }
    for c in name.chars() {
        if matches!(c, '/' | '\\' | '*' | '?' | '[' | ']' | ':') {
            return Err(crate::Error::InvalidData(format!(
                "Sheet name '{}' contains invalid character '{}'",
                name, c
            )));
        }
    }
    Ok(())
}
