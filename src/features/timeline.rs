use crate::utility::{ColNum, RowNum};

/// The granularity level for a timeline filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineLevel {
    Years,
    Quarters,
    Months,
    Days,
}

impl TimelineLevel {
    /// Return the numeric level value used in the timeline XML.
    pub fn to_xml_value(self) -> u8 {
        match self {
            TimelineLevel::Years => 0,
            TimelineLevel::Quarters => 1,
            TimelineLevel::Months => 2,
            TimelineLevel::Days => 3,
        }
    }
}

/// A timeline visual filter control for date-based filtering on pivot tables.
#[derive(Debug, Clone)]
pub struct Timeline {
    /// Display name of the timeline.
    pub name: String,
    /// Caption shown in the timeline header.
    pub caption: String,
    /// Source date field name in the pivot table.
    pub source_name: String,
    /// Row position (0-based).
    pub(crate) row: RowNum,
    /// Column position (0-based).
    pub(crate) col: ColNum,
    /// The granularity level for the timeline.
    pub level: TimelineLevel,
}

impl Timeline {
    /// Create a new timeline with the given name and source date field.
    pub fn new(name: &str, source_name: &str) -> Self {
        Self {
            name: name.to_string(),
            caption: name.to_string(),
            source_name: source_name.to_string(),
            row: 0,
            col: 0,
            level: TimelineLevel::Months,
        }
    }

    /// Set the caption displayed in the timeline header.
    pub fn set_caption(mut self, caption: &str) -> Self {
        self.caption = caption.to_string();
        self
    }

    /// Set the timeline level (granularity).
    pub fn set_level(mut self, level: TimelineLevel) -> Self {
        self.level = level;
        self
    }
}
