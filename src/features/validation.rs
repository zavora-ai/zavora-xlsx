use crate::utility::{ColNum, RowNum};

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationRule {
    List(Vec<String>),
    ListRange(String),
    WholeNumber { min: Option<i64>, max: Option<i64> },
    Decimal { min: Option<f64>, max: Option<f64> },
    DateRange { min: Option<String>, max: Option<String> },
    TextLength { min: Option<u32>, max: Option<u32> },
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorStyle { Stop, Warning, Information }

impl ErrorStyle {
    pub fn xml_str(&self) -> &str {
        match self { ErrorStyle::Stop => "stop", ErrorStyle::Warning => "warning", ErrorStyle::Information => "information" }
    }
}

#[derive(Debug, Clone)]
pub struct DataValidation {
    pub(crate) rule: ValidationRule,
    pub(crate) input_title: Option<String>,
    pub(crate) input_message: Option<String>,
    pub(crate) error_style: ErrorStyle,
    pub(crate) error_title: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) first_row: RowNum,
    pub(crate) first_col: ColNum,
    pub(crate) last_row: RowNum,
    pub(crate) last_col: ColNum,
}

impl DataValidation {
    pub fn new(rule: ValidationRule) -> Self {
        Self {
            rule, input_title: None, input_message: None,
            error_style: ErrorStyle::Stop, error_title: None, error_message: None,
            first_row: 0, first_col: 0, last_row: 0, last_col: 0,
        }
    }
    pub fn set_input_message(&mut self, title: &str, msg: &str) -> &mut Self {
        self.input_title = Some(title.into()); self.input_message = Some(msg.into()); self
    }
    pub fn set_error_message(&mut self, style: ErrorStyle, title: &str, msg: &str) -> &mut Self {
        self.error_style = style; self.error_title = Some(title.into()); self.error_message = Some(msg.into()); self
    }

    // ── Read accessors ──

    pub fn rule(&self) -> &ValidationRule { &self.rule }
    pub fn input_title(&self) -> Option<&str> { self.input_title.as_deref() }
    pub fn input_message(&self) -> Option<&str> { self.input_message.as_deref() }
    pub fn error_style(&self) -> ErrorStyle { self.error_style }
    pub fn error_title(&self) -> Option<&str> { self.error_title.as_deref() }
    pub fn error_message(&self) -> Option<&str> { self.error_message.as_deref() }
    pub fn first_row(&self) -> RowNum { self.first_row }
    pub fn first_col(&self) -> ColNum { self.first_col }
    pub fn last_row(&self) -> RowNum { self.last_row }
    pub fn last_col(&self) -> ColNum { self.last_col }
}
