use crate::utility::{ColNum, RowNum};

/// A slicer visual filter control linked to a table or pivot table.
#[derive(Debug, Clone)]
pub struct Slicer {
    /// Display name of the slicer.
    pub name: String,
    /// Caption shown in the slicer header.
    pub caption: String,
    /// Source column name in the table or pivot table.
    pub source_name: String,
    /// Row position (0-based).
    pub(crate) row: RowNum,
    /// Column position (0-based).
    pub(crate) col: ColNum,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Slicer style name (e.g. "SlicerStyleLight1").
    pub style: String,
    /// Optional pivot table cache name to link to (for pivot table slicers).
    pub(crate) pivot_cache_name: Option<String>,
    /// Table ID this slicer is linked to (resolved at write time).
    pub(crate) table_id: Option<u32>,
    /// Column index in the table (1-based, resolved at write time).
    pub(crate) column_index: Option<u32>,
}

impl Slicer {
    /// Create a new slicer with the given name and source column.
    pub fn new(name: &str, source_name: &str) -> Self {
        Self {
            name: name.to_string(),
            caption: source_name.to_string(),
            source_name: source_name.to_string(),
            row: 0,
            col: 0,
            width: 200,
            height: 300,
            style: "SlicerStyleLight1".to_string(),
            pivot_cache_name: None,
            table_id: None,
            column_index: None,
        }
    }

    /// Set the caption displayed in the slicer header.
    pub fn set_caption(mut self, caption: &str) -> Self {
        self.caption = caption.to_string();
        self
    }

    /// Set the width in pixels.
    pub fn set_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Set the height in pixels.
    pub fn set_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Set the slicer style.
    pub fn set_style(mut self, style: &str) -> Self {
        self.style = style.to_string();
        self
    }

    /// Link this slicer to a pivot table cache by name.
    pub fn link_to_pivot_cache(mut self, cache_name: &str) -> Self {
        self.pivot_cache_name = Some(cache_name.to_string());
        self
    }
}
