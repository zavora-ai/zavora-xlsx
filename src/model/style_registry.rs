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
    pub theme_color: Option<(u8, i64)>, // (theme_index, tint_x10000) for dedup
    pub shadow: bool,
    pub outline: bool,
    pub emboss: bool,
    pub engrave: bool,
}

/// Internal fill data for dedup.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct FillData {
    pub pattern: u8, // 0=none, 1=solid, ...
    pub fg_rgb: Option<[u8; 3]>,
    pub bg_rgb: Option<[u8; 3]>,
    pub gradient: Option<GradientFillData>,
}

/// Internal gradient fill data for dedup.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GradientFillData {
    pub angle_x100: i64, // angle * 100 for hash
    pub stops: Vec<GradientStopData>,
}

/// Internal gradient stop data for dedup.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GradientStopData {
    pub position_x1000: u32, // position * 1000 for hash
    pub color: [u8; 3],
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
    pub xf_style_ids: Vec<usize>, // parallel to xf_records: xfId referencing cellStyleXfs
    pub dxf_formats: Vec<DxfData>,
    pub cell_style_xfs: Vec<XfRecord>,
    pub cell_styles: Vec<CellStyleEntry>,

    font_map: HashMap<FontData, usize>,
    fill_map: HashMap<FillData, usize>,
    border_map: HashMap<BorderData, usize>,
    xf_map: HashMap<XfRecord, u32>,
    cell_style_map: HashMap<String, usize>, // style name -> cellStyleXfs index
    next_custom_num_fmt_id: u16,
}

/// A named cell style entry referencing a cellStyleXfs record.
#[derive(Debug, Clone)]
pub struct CellStyleEntry {
    pub name: String,
    pub xf_id: usize,
    pub builtin_id: u32,
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
        let gray_fill = FillData {
            pattern: 17,
            fg_rgb: None,
            bg_rgb: None,
            gradient: None,
        }; // gray125
        let default_border = BorderData::default();

        let mut reg = Self {
            fonts: vec![default_font.clone()],
            fills: vec![none_fill.clone(), gray_fill.clone()],
            borders: vec![default_border.clone()],
            num_formats: Vec::new(),
            xf_records: Vec::new(),
            xf_style_ids: Vec::new(),
            dxf_formats: Vec::new(),
            cell_style_xfs: Vec::new(),
            cell_styles: Vec::new(),
            font_map: HashMap::new(),
            fill_map: HashMap::new(),
            border_map: HashMap::new(),
            xf_map: HashMap::new(),
            cell_style_map: HashMap::new(),
            next_custom_num_fmt_id: 164,
        };
        reg.font_map.insert(default_font, 0);
        reg.fill_map.insert(none_fill, 0);
        reg.fill_map.insert(gray_fill, 1);
        reg.border_map.insert(default_border, 0);

        // Default xf record (style 0)
        let default_xf = XfRecord {
            font_id: 0,
            fill_id: 0,
            border_id: 0,
            num_fmt_id: 0,
            alignment: None,
            locked: None,
            formula_hidden: false,
            quote_prefix: false,
        };
        reg.xf_records.push(default_xf.clone());
        reg.xf_style_ids.push(0); // references "Normal" cellStyleXfs entry
        reg.xf_map.insert(default_xf.clone(), 0);

        // Default cellStyleXfs entry for "Normal"
        reg.cell_style_xfs.push(default_xf);
        reg.cell_styles.push(CellStyleEntry {
            name: "Normal".to_string(),
            xf_id: 0,
            builtin_id: 0,
        });
        reg.cell_style_map.insert("Normal".to_string(), 0);
        reg
    }

    fn intern_font(&mut self, f: FontData) -> usize {
        if let Some(&id) = self.font_map.get(&f) {
            return id;
        }
        let id = self.fonts.len();
        self.font_map.insert(f.clone(), id);
        self.fonts.push(f);
        id
    }

    fn intern_fill(&mut self, f: FillData) -> usize {
        if let Some(&id) = self.fill_map.get(&f) {
            return id;
        }
        let id = self.fills.len();
        self.fill_map.insert(f.clone(), id);
        self.fills.push(f);
        id
    }

    fn intern_border(&mut self, b: BorderData) -> usize {
        if let Some(&id) = self.border_map.get(&b) {
            return id;
        }
        let id = self.borders.len();
        self.border_map.insert(b.clone(), id);
        self.borders.push(b);
        id
    }

    fn intern_num_format(&mut self, code: &str) -> u16 {
        // Check builtins
        if code.is_empty() || code == "General" {
            return 0;
        }
        // Check existing custom
        for &(id, ref c) in &self.num_formats {
            if c == code {
                return id;
            }
        }
        let id = self.next_custom_num_fmt_id;
        self.next_custom_num_fmt_id += 1;
        self.num_formats.push((id, code.to_string()));
        id
    }

    /// Register a Format and return its xf index.
    /// Adopt the style table a file was read with, keeping every index where it was.
    ///
    /// Cells remember the `xf` index they were parsed with, but the table those indices point
    /// into was thrown away: saving rebuilt `styles.xml` from whatever this registry happened to
    /// hold, so a workbook that was opened, changed in one cell and saved came back with all of
    /// its formatting gone — bold headings plain, shaded totals white — while the save reported
    /// success. Editing a formatted file destroyed the formatting.
    ///
    /// Indices are preserved rather than remapped, because they are already written into every
    /// cell of every sheet. Anything registered afterwards appends, and deduplicates against
    /// what the file already had, so formatting something the way an existing cell is formatted
    /// reuses that entry rather than growing a second identical one.
    pub fn import_parsed(&mut self, parsed: &crate::reader::style_parser::ParsedStyles) {
        if parsed.xf_records.is_empty() {
            // Nothing to adopt. The defaults from `new` are the right answer for a file that
            // carried no style table of its own.
            return;
        }

        self.fonts = parsed
            .fonts
            .iter()
            .map(|font| FontData {
                bold: font.bold,
                italic: font.italic,
                underline: font.underline as u8,
                strikethrough: font.strikethrough,
                size_x100: (font.size * 100.0) as u32,
                name: font.name.clone(),
                color_rgb: font.color,
                theme_color: None,
                shadow: false,
                outline: false,
                emboss: false,
                engrave: false,
            })
            .collect();

        self.fills = parsed
            .fills
            .iter()
            .map(|fill| FillData {
                pattern: fill.pattern as u8,
                fg_rgb: fill.fg_color,
                bg_rgb: fill.bg_color,
                gradient: None,
            })
            .collect();

        self.borders = parsed
            .borders
            .iter()
            .map(|border| BorderData {
                top: border.top.style as u8,
                bottom: border.bottom.style as u8,
                left: border.left.style as u8,
                right: border.right.style as u8,
                color_rgb: border.top.color.or(border.left.color),
                top_color: border.top.color,
                bottom_color: border.bottom.color,
                left_color: border.left.color,
                right_color: border.right.color,
                ..Default::default()
            })
            .collect();

        // A file may legally have no fonts, fills or borders of its own while still having xf
        // records that point at index 0. The minimum table has to exist or those point nowhere.
        if self.fonts.is_empty() {
            self.fonts.push(FontData {
                size_x100: 1100,
                name: "Calibri".into(),
                ..Default::default()
            });
        }
        while self.fills.len() < 2 {
            self.fills.push(FillData::default());
        }
        if self.borders.is_empty() {
            self.borders.push(BorderData::default());
        }

        self.num_formats = parsed.num_formats.clone();
        self.xf_records = parsed
            .xf_records
            .iter()
            .map(|record| XfRecord {
                font_id: record.font_id,
                fill_id: record.fill_id,
                border_id: record.border_id,
                num_fmt_id: record.num_fmt_id,
                // Only kept when it says something. An alignment of all defaults would write a
                // redundant <alignment/> into every style the file has.
                alignment: if record.alignment.horizontal == 0
                    && record.alignment.vertical == 0
                    && !record.alignment.wrap_text
                    && !record.alignment.shrink_to_fit
                    && record.alignment.indent == 0
                    && record.alignment.rotation == 0
                {
                    None
                } else {
                    Some(AlignmentData {
                        horizontal: record.alignment.horizontal,
                        vertical: record.alignment.vertical,
                        wrap_text: record.alignment.wrap_text,
                        shrink: record.alignment.shrink_to_fit,
                        indent: record.alignment.indent,
                        rotation: record.alignment.rotation,
                    })
                },
                locked: None,
                formula_hidden: false,
                quote_prefix: false,
            })
            .collect();
        // The number format each xf uses lives beside the records when parsed, so it is folded
        // back in; without it every cell keeps its font and loses its "1,234.00".
        for (index, record) in self.xf_records.iter_mut().enumerate() {
            if let Some(&num_fmt_id) = parsed.xf_num_fmt_ids.get(index) {
                record.num_fmt_id = num_fmt_id;
            }
        }
        self.xf_style_ids = vec![0; self.xf_records.len()];

        // Rebuilt so that anything registered from here on deduplicates against the file's own
        // entries instead of adding a second copy of a format the file already has.
        self.font_map = self
            .fonts
            .iter()
            .enumerate()
            .map(|(index, font)| (font.clone(), index))
            .collect();
        self.fill_map = self
            .fills
            .iter()
            .enumerate()
            .map(|(index, fill)| (fill.clone(), index))
            .collect();
        self.border_map = self
            .borders
            .iter()
            .enumerate()
            .map(|(index, border)| (border.clone(), index))
            .collect();
        self.xf_map = self
            .xf_records
            .iter()
            .enumerate()
            .map(|(index, record)| (record.clone(), index as u32))
            .collect();

        // A custom number format id must not collide with one the file already uses.
        let highest = self
            .num_formats
            .iter()
            .map(|(id, _)| *id)
            .max()
            .unwrap_or(163);
        self.next_custom_num_fmt_id = highest.max(163) + 1;
    }

    pub fn register_format(&mut self, fmt: &crate::format::Format) -> u32 {
        let font = FontData {
            bold: fmt.bold,
            italic: fmt.italic,
            underline: fmt.underline as u8,
            strikethrough: fmt.strikethrough,
            size_x100: (fmt.font_size * 100.0) as u32,
            name: fmt.font_name.clone(),
            color_rgb: fmt.font_color,
            theme_color: fmt
                .theme_color
                .map(|tc| (tc.index.to_index(), (tc.tint * 10000.0) as i64)),
            shadow: fmt.shadow,
            outline: fmt.outline,
            emboss: fmt.emboss,
            engrave: fmt.engrave,
        };
        let fill = if let Some(ref grad) = fmt.gradient {
            let stops: Vec<GradientStopData> = grad
                .stops
                .iter()
                .map(|s| GradientStopData {
                    position_x1000: (s.position * 1000.0) as u32,
                    color: s.color,
                })
                .collect();
            FillData {
                pattern: 0,
                fg_rgb: None,
                bg_rgb: None,
                gradient: Some(GradientFillData {
                    angle_x100: (grad.angle * 100.0) as i64,
                    stops,
                }),
            }
        } else if let Some(bg) = fmt.bg_color {
            let p = if matches!(fmt.pattern, crate::format::Pattern::None) {
                1u8
            } else {
                fmt.pattern as u8
            };
            FillData {
                pattern: p,
                fg_rgb: fmt.fg_color.or(Some(bg)),
                bg_rgb: Some(bg),
                gradient: None,
            }
        } else if !matches!(fmt.pattern, crate::format::Pattern::None) {
            FillData {
                pattern: fmt.pattern as u8,
                fg_rgb: fmt.fg_color,
                bg_rgb: None,
                gradient: None,
            }
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

        // If a cell_style is specified, ensure it's registered in cellStyleXfs
        let xf_id = if let Some(ref style_name) = fmt.cell_style {
            self.ensure_cell_style(style_name, font_id, fill_id, border_id, num_fmt_id)
        } else {
            0 // default "Normal" style
        };

        let xf = XfRecord {
            font_id,
            fill_id,
            border_id,
            num_fmt_id,
            alignment,
            locked: fmt.locked,
            formula_hidden: fmt.formula_hidden,
            quote_prefix: fmt.quote_prefix,
        };
        if let Some(&idx) = self.xf_map.get(&xf) {
            return idx;
        }
        let idx = self.xf_records.len() as u32;
        self.xf_map.insert(xf.clone(), idx);
        self.xf_records.push(xf);
        self.xf_style_ids.push(xf_id);
        idx
    }

    /// Ensure a named cell style is registered in cellStyleXfs. Returns the xfId.
    fn ensure_cell_style(
        &mut self,
        name: &str,
        font_id: usize,
        fill_id: usize,
        border_id: usize,
        num_fmt_id: u16,
    ) -> usize {
        if let Some(&xf_id) = self.cell_style_map.get(name) {
            return xf_id;
        }
        let builtin_id = Self::builtin_id_for_name(name);
        let xf = XfRecord {
            font_id,
            fill_id,
            border_id,
            num_fmt_id,
            alignment: None,
            locked: None,
            formula_hidden: false,
            quote_prefix: false,
        };
        let xf_id = self.cell_style_xfs.len();
        self.cell_style_xfs.push(xf);
        self.cell_styles.push(CellStyleEntry {
            name: name.to_string(),
            xf_id,
            builtin_id,
        });
        self.cell_style_map.insert(name.to_string(), xf_id);
        xf_id
    }

    /// Get the builtinId for a named cell style.
    fn builtin_id_for_name(name: &str) -> u32 {
        match name {
            "Normal" => 0,
            "Comma" => 3,
            "Currency" => 4,
            "Percent" => 5,
            "Title" => 15,
            "Heading 1" => 16,
            "Heading 2" => 17,
            "Heading 3" => 18,
            "Heading 4" => 19,
            "Total" => 25,
            "Good" => 26,
            "Bad" => 27,
            "Neutral" => 28,
            _ => 0, // fallback to Normal for unknown styles
        }
    }

    /// Register a differential format for conditional formatting. Returns dxf index.
    pub fn register_dxf(&mut self, fmt: &crate::format::Format) -> u32 {
        let font = if fmt.bold
            || fmt.italic
            || fmt.strikethrough
            || fmt.font_color.is_some()
            || fmt.underline as u8 > 0
        {
            Some(FontData {
                bold: fmt.bold,
                italic: fmt.italic,
                underline: fmt.underline as u8,
                strikethrough: fmt.strikethrough,
                size_x100: 0,
                name: String::new(),
                color_rgb: fmt.font_color,
                theme_color: None,
                shadow: false,
                outline: false,
                emboss: false,
                engrave: false,
            })
        } else {
            None
        };
        let fill = fmt.bg_color.map(|bg| FillData {
            pattern: 1,
            fg_rgb: Some(bg),
            bg_rgb: None,
            gradient: None,
        });
        let border = if fmt.border_top as u8 > 0
            || fmt.border_bottom as u8 > 0
            || fmt.border_left as u8 > 0
            || fmt.border_right as u8 > 0
        {
            Some(BorderData {
                top: fmt.border_top as u8,
                bottom: fmt.border_bottom as u8,
                left: fmt.border_left as u8,
                right: fmt.border_right as u8,
                color_rgb: fmt.border_color,
                top_color: None,
                bottom_color: None,
                left_color: None,
                right_color: None,
                diagonal: 0,
                diagonal_type: 0,
            })
        } else {
            None
        };
        let num_format = if fmt.num_format.is_empty() {
            None
        } else {
            Some(fmt.num_format.clone())
        };
        let idx = self.dxf_formats.len() as u32;
        self.dxf_formats.push(DxfData {
            font,
            fill,
            border,
            num_format,
        });
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
    fn default() -> Self {
        Self::new()
    }
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
