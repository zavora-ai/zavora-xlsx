/// Cell format builder with font, background, borders, number format, and alignment.
/// Formats are deduplicated internally — identical formats share the same xf index (4 bytes per cell).
#[derive(Debug, Clone)]
pub struct Format {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) underline: Underline,
    pub(crate) strikethrough: bool,
    pub(crate) font_size: f64,
    pub(crate) font_name: String,
    pub(crate) font_color: Option<[u8; 3]>,
    pub(crate) bg_color: Option<[u8; 3]>,
    pub(crate) border_top: BorderStyle,
    pub(crate) border_bottom: BorderStyle,
    pub(crate) border_left: BorderStyle,
    pub(crate) border_right: BorderStyle,
    pub(crate) border_color: Option<[u8; 3]>,
    pub(crate) border_top_color: Option<[u8; 3]>,
    pub(crate) border_bottom_color: Option<[u8; 3]>,
    pub(crate) border_left_color: Option<[u8; 3]>,
    pub(crate) border_right_color: Option<[u8; 3]>,
    pub(crate) h_align: u8,
    pub(crate) v_align: u8,
    pub(crate) wrap_text: bool,
    pub(crate) shrink: bool,
    pub(crate) indent: u8,
    pub(crate) rotation: i16,
    pub(crate) num_format: String,
    pub(crate) locked: Option<bool>,
    pub(crate) formula_hidden: bool,
    // Sprint 9
    pub(crate) diagonal_border: BorderStyle,
    pub(crate) diagonal_type: DiagonalType,
    pub(crate) fg_color: Option<[u8; 3]>,
    pub(crate) pattern: Pattern,
    pub(crate) quote_prefix: bool,
    pub(crate) gradient: Option<GradientFill>,
    pub(crate) theme_color: Option<ThemeColor>,
    pub(crate) cell_style: Option<String>,
    // Text effects
    pub(crate) shadow: bool,
    pub(crate) outline: bool,
    pub(crate) emboss: bool,
    pub(crate) engrave: bool,
}

impl Format {
    pub fn new() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: Underline::None,
            strikethrough: false,
            font_size: 11.0,
            font_name: "Calibri".into(),
            font_color: None,
            bg_color: None,
            border_top: BorderStyle::None,
            border_bottom: BorderStyle::None,
            border_left: BorderStyle::None,
            border_right: BorderStyle::None,
            border_color: None,
            border_top_color: None,
            border_bottom_color: None,
            border_left_color: None,
            border_right_color: None,
            h_align: 0,
            v_align: 0,
            wrap_text: false,
            shrink: false,
            indent: 0,
            rotation: 0,
            num_format: String::new(),
            locked: None,
            formula_hidden: false,
            diagonal_border: BorderStyle::None,
            diagonal_type: DiagonalType::None,
            fg_color: None,
            pattern: Pattern::None,
            quote_prefix: false,
            gradient: None,
            theme_color: None,
            cell_style: None,
            shadow: false,
            outline: false,
            emboss: false,
            engrave: false,
        }
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    pub fn underline(mut self, u: Underline) -> Self {
        self.underline = u;
        self
    }
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }
    pub fn font_size(mut self, s: f64) -> Self {
        self.font_size = s;
        self
    }
    pub fn font_name(mut self, n: &str) -> Self {
        self.font_name = n.into();
        self
    }
    pub fn font_color(mut self, c: impl IntoColor) -> Self {
        self.font_color = Some(c.into_color().to_rgb());
        self
    }
    pub fn background_color(mut self, c: impl IntoColor) -> Self {
        self.bg_color = Some(c.into_color().to_rgb());
        self
    }
    pub fn num_format(mut self, f: &str) -> Self {
        self.num_format = f.into();
        self
    }

    pub fn border(mut self, s: BorderStyle) -> Self {
        self.border_top = s;
        self.border_bottom = s;
        self.border_left = s;
        self.border_right = s;
        self
    }
    pub fn border_color(mut self, c: impl IntoColor) -> Self {
        self.border_color = Some(c.into_color().to_rgb());
        self
    }
    pub fn border_top_color(mut self, c: impl IntoColor) -> Self {
        self.border_top_color = Some(c.into_color().to_rgb());
        self
    }
    pub fn border_bottom_color(mut self, c: impl IntoColor) -> Self {
        self.border_bottom_color = Some(c.into_color().to_rgb());
        self
    }
    pub fn border_left_color(mut self, c: impl IntoColor) -> Self {
        self.border_left_color = Some(c.into_color().to_rgb());
        self
    }
    pub fn border_right_color(mut self, c: impl IntoColor) -> Self {
        self.border_right_color = Some(c.into_color().to_rgb());
        self
    }
    pub fn border_top(mut self, s: BorderStyle) -> Self {
        self.border_top = s;
        self
    }
    pub fn border_bottom(mut self, s: BorderStyle) -> Self {
        self.border_bottom = s;
        self
    }
    pub fn border_left(mut self, s: BorderStyle) -> Self {
        self.border_left = s;
        self
    }
    pub fn border_right(mut self, s: BorderStyle) -> Self {
        self.border_right = s;
        self
    }

    pub fn align(mut self, a: Align) -> Self {
        match a {
            Align::Left => self.h_align = 1,
            Align::Center => self.h_align = 2,
            Align::Right => self.h_align = 3,
            Align::Fill => self.h_align = 4,
            Align::Justify => self.h_align = 5,
            Align::Top => self.v_align = 0,
            Align::VerticalCenter => self.v_align = 1,
            Align::Bottom => self.v_align = 2,
        }
        self
    }
    pub fn text_wrap(mut self) -> Self {
        self.wrap_text = true;
        self
    }
    pub fn shrink_to_fit(mut self) -> Self {
        self.shrink = true;
        self
    }
    pub fn indent(mut self, level: u8) -> Self {
        self.indent = level;
        self
    }
    pub fn rotation(mut self, angle: i16) -> Self {
        self.rotation = angle;
        self
    }
    pub fn unlocked(mut self) -> Self {
        self.locked = Some(false);
        self
    }
    pub fn locked(mut self) -> Self {
        self.locked = Some(true);
        self
    }
    pub fn formula_hidden(mut self) -> Self {
        self.formula_hidden = true;
        self
    }

    /// Set diagonal border (up, down, or both).
    pub fn diagonal_border(mut self, style: BorderStyle, diag_type: DiagonalType) -> Self {
        self.diagonal_border = style;
        self.diagonal_type = diag_type;
        self
    }
    /// Set foreground color for pattern fills.
    pub fn foreground_color(mut self, c: impl IntoColor) -> Self {
        self.fg_color = Some(c.into_color().to_rgb());
        self
    }
    /// Set pattern fill type.
    pub fn pattern_fill(mut self, p: Pattern) -> Self {
        self.pattern = p;
        self
    }
    /// Force text display (leading apostrophe).
    pub fn quote_prefix(mut self) -> Self {
        self.quote_prefix = true;
        self
    }

    /// Set a gradient fill with the given angle (degrees) and color stops.
    pub fn gradient_fill(mut self, angle: f64, stops: Vec<GradientStop>) -> Self {
        self.gradient = Some(GradientFill { angle, stops });
        self
    }

    /// Set the font color using a theme color reference with optional tint.
    /// The tint value ranges from -1.0 (fully dark) to 1.0 (fully light).
    pub fn theme_color(mut self, index: ThemeColorIndex, tint: f64) -> Self {
        self.theme_color = Some(ThemeColor { index, tint });
        self
    }

    /// Apply a named cell style (e.g. "Normal", "Heading 1", "Currency", "Percent").
    pub fn cell_style(mut self, name: &str) -> Self {
        self.cell_style = Some(name.to_string());
        self
    }

    /// Apply shadow text effect.
    pub fn shadow(mut self) -> Self {
        self.shadow = true;
        self
    }
    /// Apply outline text effect.
    pub fn outline(mut self) -> Self {
        self.outline = true;
        self
    }
    /// Apply emboss text effect.
    pub fn emboss(mut self) -> Self {
        self.emboss = true;
        self
    }
    /// Apply engrave (imprint) text effect.
    pub fn engrave(mut self) -> Self {
        self.engrave = true;
        self
    }

    pub(crate) fn has_alignment(&self) -> bool {
        self.h_align != 0
            || self.v_align != 0
            || self.wrap_text
            || self.shrink
            || self.indent != 0
            || self.rotation != 0
    }

    // ── Read accessors ──

    /// Returns `true` if the format has bold font.
    pub fn is_bold(&self) -> bool {
        self.bold
    }
    /// Returns `true` if the format has italic font.
    pub fn is_italic(&self) -> bool {
        self.italic
    }
    /// Returns the underline style.
    pub fn get_underline(&self) -> Underline {
        self.underline
    }
    /// Returns `true` if the format has strikethrough.
    pub fn is_strikethrough(&self) -> bool {
        self.strikethrough
    }
    /// Returns the font size.
    pub fn get_font_size(&self) -> f64 {
        self.font_size
    }
    /// Returns the font name.
    pub fn get_font_name(&self) -> &str {
        &self.font_name
    }
    /// Returns the font color as RGB, if set.
    pub fn get_font_color(&self) -> Option<[u8; 3]> {
        self.font_color
    }
    /// Returns the background color as RGB, if set.
    pub fn get_bg_color(&self) -> Option<[u8; 3]> {
        self.bg_color
    }
    /// Returns the foreground color as RGB, if set.
    pub fn get_fg_color(&self) -> Option<[u8; 3]> {
        self.fg_color
    }
    /// Returns the fill pattern.
    pub fn get_pattern(&self) -> Pattern {
        self.pattern
    }
    /// Returns the left border style.
    pub fn get_border_left(&self) -> BorderStyle {
        self.border_left
    }
    /// Returns the right border style.
    pub fn get_border_right(&self) -> BorderStyle {
        self.border_right
    }
    /// Returns the top border style.
    pub fn get_border_top(&self) -> BorderStyle {
        self.border_top
    }
    /// Returns the bottom border style.
    pub fn get_border_bottom(&self) -> BorderStyle {
        self.border_bottom
    }
    /// Returns the horizontal alignment value.
    pub fn get_h_align(&self) -> u8 {
        self.h_align
    }
    /// Returns the vertical alignment value.
    pub fn get_v_align(&self) -> u8 {
        self.v_align
    }
    /// Returns `true` if text wrapping is enabled.
    pub fn is_wrap_text(&self) -> bool {
        self.wrap_text
    }
    /// Returns `true` if shrink-to-fit is enabled.
    pub fn is_shrink(&self) -> bool {
        self.shrink
    }
    /// Returns the indent level.
    pub fn get_indent(&self) -> u8 {
        self.indent
    }
    /// Returns the text rotation angle.
    pub fn get_rotation(&self) -> i16 {
        self.rotation
    }
    /// Returns the number format string.
    pub fn get_num_format(&self) -> &str {
        &self.num_format
    }
    /// Returns the theme color, if set.
    pub fn get_theme_color(&self) -> Option<&ThemeColor> {
        self.theme_color.as_ref()
    }
    /// Returns the named cell style, if set.
    pub fn get_cell_style(&self) -> Option<&str> {
        self.cell_style.as_deref()
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Underline {
    None = 0,
    Single = 1,
    Double = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[derive(Default)]
pub enum BorderStyle {
    #[default]
    None = 0,
    Thin = 1,
    Medium = 2,
    Thick = 3,
    Dashed = 4,
    Dotted = 5,
    Double = 6,
}

#[derive(Debug, Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right,
    Fill,
    Justify,
    Top,
    VerticalCenter,
    Bottom,
}

/// A gradient stop defining a color at a specific position.
#[derive(Debug, Clone, PartialEq)]
pub struct GradientStop {
    /// Position of the stop (0.0 to 1.0).
    pub position: f64,
    /// RGB color at this stop.
    pub color: [u8; 3],
}

/// A gradient fill with an angle and color stops.
#[derive(Debug, Clone, PartialEq)]
pub struct GradientFill {
    /// Angle of the gradient in degrees.
    pub angle: f64,
    /// Color stops defining the gradient.
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone, Copy)]
pub enum Pattern {
    None = 0,
    Solid = 1,
    MediumGray = 2,
    DarkGray = 3,
    LightGray = 4,
    DarkHorizontal = 5,
    DarkVertical = 6,
    DarkDown = 7,
    DarkUp = 8,
    DarkGrid = 9,
    DarkTrellis = 10,
    LightHorizontal = 11,
    LightVertical = 12,
    LightDown = 13,
    LightUp = 14,
    LightGrid = 15,
    LightTrellis = 16,
    Gray125 = 17,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DiagonalType {
    #[default]
    None,
    Up,
    Down,
    Both,
}

/// A theme color reference with optional tint adjustment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeColor {
    pub index: ThemeColorIndex,
    pub tint: f64, // -1.0 to 1.0
}

/// Theme color indices corresponding to the Office theme color scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeColorIndex {
    Dark1 = 0,
    Light1 = 1,
    Dark2 = 2,
    Light2 = 3,
    Accent1 = 4,
    Accent2 = 5,
    Accent3 = 6,
    Accent4 = 7,
    Accent5 = 8,
    Accent6 = 9,
}

impl ThemeColorIndex {
    /// Convert a numeric theme index to a ThemeColorIndex enum variant.
    pub fn from_index(idx: u8) -> Option<Self> {
        match idx {
            0 => Some(Self::Dark1),
            1 => Some(Self::Light1),
            2 => Some(Self::Dark2),
            3 => Some(Self::Light2),
            4 => Some(Self::Accent1),
            5 => Some(Self::Accent2),
            6 => Some(Self::Accent3),
            7 => Some(Self::Accent4),
            8 => Some(Self::Accent5),
            9 => Some(Self::Accent6),
            _ => None,
        }
    }

    /// Get the numeric theme index value.
    pub fn to_index(self) -> u8 {
        self as u8
    }

    /// Get the default RGB color for this theme index (Office default theme).
    pub fn default_rgb(self) -> [u8; 3] {
        match self {
            Self::Dark1 => [0x00, 0x00, 0x00],
            Self::Light1 => [0xFF, 0xFF, 0xFF],
            Self::Dark2 => [0x44, 0x54, 0x6A],
            Self::Light2 => [0xE7, 0xE6, 0xE6],
            Self::Accent1 => [0x44, 0x72, 0xC4],
            Self::Accent2 => [0xED, 0x7D, 0x31],
            Self::Accent3 => [0xA5, 0xA5, 0xA5],
            Self::Accent4 => [0xFF, 0xC0, 0x00],
            Self::Accent5 => [0x5B, 0x9B, 0xD5],
            Self::Accent6 => [0x70, 0xAD, 0x47],
        }
    }
}

/// Apply a tint to an RGB color.
/// Positive tint lightens toward white, negative tint darkens toward black.
pub fn apply_tint(rgb: [u8; 3], tint: f64) -> [u8; 3] {
    if tint == 0.0 {
        return rgb;
    }
    let apply = |c: u8| -> u8 {
        let c_f = c as f64 / 255.0;
        let result = if tint > 0.0 {
            c_f + (1.0 - c_f) * tint
        } else {
            c_f * (1.0 + tint)
        };
        (result.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    [apply(rgb[0]), apply(rgb[1]), apply(rgb[2])]
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Rgb(u8, u8, u8),
    Named(NamedColor),
}

impl Color {
    pub fn to_rgb(self) -> [u8; 3] {
        match self {
            Color::Rgb(r, g, b) => [r, g, b],
            Color::Named(n) => n.to_rgb(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NamedColor {
    Black,
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    Orange,
    Purple,
    Gray,
}

impl NamedColor {
    pub fn to_rgb(self) -> [u8; 3] {
        match self {
            NamedColor::Black => [0, 0, 0],
            NamedColor::White => [255, 255, 255],
            NamedColor::Red => [255, 0, 0],
            NamedColor::Green => [0, 128, 0],
            NamedColor::Blue => [0, 0, 255],
            NamedColor::Yellow => [255, 255, 0],
            NamedColor::Cyan => [0, 255, 255],
            NamedColor::Magenta => [255, 0, 255],
            NamedColor::Orange => [255, 165, 0],
            NamedColor::Purple => [128, 0, 128],
            NamedColor::Gray => [128, 128, 128],
        }
    }
}

pub trait IntoColor {
    fn into_color(self) -> Color;
}

impl IntoColor for Color {
    fn into_color(self) -> Color {
        self
    }
}
impl IntoColor for NamedColor {
    fn into_color(self) -> Color {
        Color::Named(self)
    }
}
impl IntoColor for (u8, u8, u8) {
    fn into_color(self) -> Color {
        Color::Rgb(self.0, self.1, self.2)
    }
}

impl IntoColor for &str {
    fn into_color(self) -> Color {
        let s = self.trim_start_matches('#');
        if s.len() == 6
            && let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&s[0..2], 16),
                u8::from_str_radix(&s[2..4], 16),
                u8::from_str_radix(&s[4..6], 16),
            )
        {
            return Color::Rgb(r, g, b);
        }
        Color::Rgb(0, 0, 0) // fallback
    }
}
