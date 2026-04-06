use std::collections::HashMap;

/// Internal font data for dedup.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct FontData {
    pub bold: bool,
    pub italic: bool,
    pub underline: u8, // 0=none, 1=single, 2=double
    pub strikethrough: bool,
    pub size_x100: u32, // size * 100 for hash
    pub name: String,
    pub color_rgb: Option<[u8; 3]>,
}

/// Internal fill data for dedup.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FillData {
    pub pattern: u8, // 0=none, 1=solid, ...
    pub fg_rgb: Option<[u8; 3]>,
    pub bg_rgb: Option<[u8; 3]>,
}

impl Default for FillData {
    fn default() -> Self { Self { pattern: 0, fg_rgb: None, bg_rgb: None } }
}

/// Internal border data for dedup.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct BorderData {
    pub top: u8,
    pub bottom: u8,
    pub left: u8,
    pub right: u8,
    pub color_rgb: Option<[u8; 3]>,
    pub top_color: Option<[u8; 3]>,
    pub bottom_color: Option<[u8; 3]>,
    pub left_color: Option<[u8; 3]>,
    pub right_color: Option<[u8; 3]>,
    pub diagonal: u8,
    pub diagonal_type: u8, // 0=none, 1=up, 2=down, 3=both
}

/// Internal alignment data.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct AlignmentData {
    pub horizontal: u8, // 0=general,1=left,2=center,3=right,4=fill,5=justify
    pub vertical: u8,   // 0=top,1=center,2=bottom
    pub wrap_text: bool,
    pub shrink: bool,
    pub indent: u8,
    pub rotation: i16,
}

/// An xf record — references into the indexed collections.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct XfRecord {
    pub font_id: usize,
    pub fill_id: usize,
    pub border_id: usize,
    pub num_fmt_id: u16,
    pub alignment: Option<AlignmentData>,
    pub locked: Option<bool>,
    pub formula_hidden: bool,
    pub quote_prefix: bool,
}

/// Centralized style registry with deduplication.
#[derive(Debug)]
pub struct StyleRegistry {
    pub fonts: Vec<FontData>,
    pub fills: Vec<FillData>,
    pub borders: Vec<BorderData>,
    pub num_formats: Vec<(u16, String)>, // (id, format_code)
    pub xf_records: Vec<XfRecord>,
    pub dxf_formats: Vec<DxfData>,

    font_map: HashMap<FontData, usize>,
    fill_map: HashMap<FillData, usize>,
    border_map: HashMap<BorderData, usize>,
    xf_map: HashMap<XfRecord, u32>,
    next_custom_num_fmt_id: u16,
}

/// Differential formatting data for conditional formatting.
#[derive(Debug, Clone)]
pub struct DxfData {
    pub font: Option<FontData>,
    pub fill: Option<FillData>,
    pub border: Option<BorderData>,
    pub num_format: Option<String>,
}

impl StyleRegistry {
    /// Create with Excel's required default entries.
    pub fn new() -> Self {
        let default_font = FontData {
            size_x100: 1100,
            name: "Calibri".into(),
            ..Default::default()
        };
        let none_fill = FillData::default();
        let gray_fill = FillData { pattern: 17, fg_rgb: None, bg_rgb: None }; // gray125
        let default_border = BorderData::default();

        let mut reg = Self {
            fonts: vec![default_font.clone()],
            fills: vec![none_fill.clone(), gray_fill.clone()],
            borders: vec![default_border.clone()],
            num_formats: Vec::new(),
            xf_records: Vec::new(),
            dxf_formats: Vec::new(),
            font_map: HashMap::new(),
            fill_map: HashMap::new(),
            border_map: HashMap::new(),
            xf_map: HashMap::new(),
            next_custom_num_fmt_id: 164,
        };
        reg.font_map.insert(default_font, 0);
        reg.fill_map.insert(none_fill, 0);
        reg.fill_map.insert(gray_fill, 1);
        reg.border_map.insert(default_border, 0);

        // Default xf record (style 0)
        let default_xf = XfRecord {
            font_id: 0, fill_id: 0, border_id: 0, num_fmt_id: 0, alignment: None,
            locked: None, formula_hidden: false, quote_prefix: false,
        };
        reg.xf_records.push(default_xf.clone());
        reg.xf_map.insert(default_xf, 0);
        reg
    }

    fn intern_font(&mut self, f: FontData) -> usize {
        if let Some(&id) = self.font_map.get(&f) { return id; }
        let id = self.fonts.len();
        self.font_map.insert(f.clone(), id);
        self.fonts.push(f);
        id
    }

    fn intern_fill(&mut self, f: FillData) -> usize {
        if let Some(&id) = self.fill_map.get(&f) { return id; }
        let id = self.fills.len();
        self.fill_map.insert(f.clone(), id);
        self.fills.push(f);
        id
    }

    fn intern_border(&mut self, b: BorderData) -> usize {
        if let Some(&id) = self.border_map.get(&b) { return id; }
        let id = self.borders.len();
        self.border_map.insert(b.clone(), id);
        self.borders.push(b);
        id
    }

    fn intern_num_format(&mut self, code: &str) -> u16 {
        // Check builtins
        if code.is_empty() || code == "General" { return 0; }
        // Check existing custom
        for &(id, ref c) in &self.num_formats {
            if c == code { return id; }
        }
        let id = self.next_custom_num_fmt_id;
        self.next_custom_num_fmt_id += 1;
        self.num_formats.push((id, code.to_string()));
        id
    }

    /// Register a Format and return its xf index.
    pub fn register_format(&mut self, fmt: &crate::format::Format) -> u32 {
        let font = FontData {
            bold: fmt.bold,
            italic: fmt.italic,
            underline: fmt.underline as u8,
            strikethrough: fmt.strikethrough,
            size_x100: (fmt.font_size * 100.0) as u32,
            name: fmt.font_name.clone(),
            color_rgb: fmt.font_color,
        };
        let fill = if let Some(bg) = fmt.bg_color {
            let p = if matches!(fmt.pattern, crate::format::Pattern::None) { 1u8 } else { fmt.pattern as u8 };
            FillData { pattern: p, fg_rgb: fmt.fg_color.or(Some(bg)), bg_rgb: Some(bg) }
        } else if !matches!(fmt.pattern, crate::format::Pattern::None) {
            FillData { pattern: fmt.pattern as u8, fg_rgb: fmt.fg_color, bg_rgb: None }
        } else {
            FillData::default()
        };
        let border = BorderData {
            top: fmt.border_top as u8,
            bottom: fmt.border_bottom as u8,
            left: fmt.border_left as u8,
            right: fmt.border_right as u8,
            color_rgb: fmt.border_color,
            top_color: fmt.border_top_color,
            bottom_color: fmt.border_bottom_color,
            left_color: fmt.border_left_color,
            right_color: fmt.border_right_color,
            diagonal: fmt.diagonal_border as u8,
            diagonal_type: fmt.diagonal_type as u8,
        };
        let alignment = if fmt.has_alignment() {
            Some(AlignmentData {
                horizontal: fmt.h_align,
                vertical: fmt.v_align,
                wrap_text: fmt.wrap_text,
                shrink: fmt.shrink,
                indent: fmt.indent,
                rotation: fmt.rotation,
            })
        } else {
            None
        };

        let font_id = self.intern_font(font);
        let fill_id = self.intern_fill(fill);
        let border_id = self.intern_border(border);
        let num_fmt_id = self.intern_num_format(&fmt.num_format);

        let xf = XfRecord { font_id, fill_id, border_id, num_fmt_id, alignment, locked: fmt.locked, formula_hidden: fmt.formula_hidden, quote_prefix: fmt.quote_prefix };
        if let Some(&idx) = self.xf_map.get(&xf) {
            return idx;
        }
        let idx = self.xf_records.len() as u32;
        self.xf_map.insert(xf.clone(), idx);
        self.xf_records.push(xf);
        idx
    }

    /// Register a differential format for conditional formatting. Returns dxf index.
    pub fn register_dxf(&mut self, fmt: &crate::format::Format) -> u32 {
        let font = if fmt.bold || fmt.italic || fmt.strikethrough || fmt.font_color.is_some() || fmt.underline as u8 > 0 {
            Some(FontData {
                bold: fmt.bold, italic: fmt.italic, underline: fmt.underline as u8,
                strikethrough: fmt.strikethrough, size_x100: 0, name: String::new(),
                color_rgb: fmt.font_color,
            })
        } else { None };
        let fill = fmt.bg_color.map(|bg| FillData { pattern: 1, fg_rgb: Some(bg), bg_rgb: None });
        let border = if fmt.border_top as u8 > 0 || fmt.border_bottom as u8 > 0 || fmt.border_left as u8 > 0 || fmt.border_right as u8 > 0 {
            Some(BorderData { top: fmt.border_top as u8, bottom: fmt.border_bottom as u8, left: fmt.border_left as u8, right: fmt.border_right as u8, color_rgb: fmt.border_color, top_color: None, bottom_color: None, left_color: None, right_color: None, diagonal: 0, diagonal_type: 0 })
        } else { None };
        let num_format = if fmt.num_format.is_empty() { None } else { Some(fmt.num_format.clone()) };
        let idx = self.dxf_formats.len() as u32;
        self.dxf_formats.push(DxfData { font, fill, border, num_format });
        idx
    }

    /// Detect if a number format ID represents a date/time.
    pub fn is_date_format(num_fmt_id: u16, custom_formats: &[(u16, String)]) -> bool {
        // Builtin date format IDs
        match num_fmt_id {
            14..=22 | 45..=47 | 27..=36 | 50..=58 | 71..=81 => return true,
            _ => {}
        }
        // Check custom format string
        if let Some((_, code)) = custom_formats.iter().find(|(id, _)| *id == num_fmt_id) {
            return Self::format_string_is_date(code);
        }
        false
    }

    /// Heuristic: does a format string contain date/time tokens?
    pub fn format_string_is_date(code: &str) -> bool {
        let mut in_quote = false;
        let mut in_bracket = false;
        for c in code.chars() {
            match c {
                '"' => in_quote = !in_quote,
                '[' if !in_quote => in_bracket = true,
                ']' if !in_quote => in_bracket = false,
                '\\' if !in_quote => { /* skip next */ }
                ';' if !in_quote => return false, // only check first section
                'd' | 'm' | 'y' | 'h' | 's' if !in_quote && !in_bracket => return true,
                _ => {}
            }
        }
        false
    }
}

impl Default for StyleRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::Format;

    #[test]
    fn default_format_is_zero() {
        let mut reg = StyleRegistry::new();
        let idx = reg.register_format(&Format::new());
        assert_eq!(idx, 0);
    }

    #[test]
    fn dedup_identical_formats() {
        let mut reg = StyleRegistry::new();
        let f = Format::new().bold();
        let idx1 = reg.register_format(&f);
        let idx2 = reg.register_format(&f);
        assert_eq!(idx1, idx2);
        assert!(idx1 > 0); // not the default
    }

    #[test]
    fn different_formats_different_indices() {
        let mut reg = StyleRegistry::new();
        let a = reg.register_format(&Format::new().bold());
        let b = reg.register_format(&Format::new().italic());
        assert_ne!(a, b);
    }

    #[test]
    fn date_format_detection() {
        assert!(StyleRegistry::format_string_is_date("yyyy-mm-dd"));
        assert!(StyleRegistry::format_string_is_date("dd/mm/yyyy hh:mm"));
        assert!(!StyleRegistry::format_string_is_date("#,##0.00"));
        assert!(!StyleRegistry::format_string_is_date("\"Date: \"General"));
    }
}
