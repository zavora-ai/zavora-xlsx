/// Drawing shapes for Task 72.
use crate::utility::{ColNum, RowNum};

/// Preset shape types supported by the drawing engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeType {
    Rectangle,
    RoundedRectangle,
    Ellipse,
    Triangle,
    Diamond,
    Arrow,
    Callout,
    TextBox,
}

impl ShapeType {
    /// Return the OOXML preset geometry name.
    pub fn preset_name(&self) -> &str {
        match self {
            ShapeType::Rectangle => "rect",
            ShapeType::RoundedRectangle => "roundRect",
            ShapeType::Ellipse => "ellipse",
            ShapeType::Triangle => "triangle",
            ShapeType::Diamond => "diamond",
            ShapeType::Arrow => "rightArrow",
            ShapeType::Callout => "wedgeRoundRectCallout",
            ShapeType::TextBox => "rect",
        }
    }
}

/// A drawing shape placed on a worksheet.
#[derive(Debug, Clone)]
pub struct Shape {
    pub shape_type: ShapeType,
    pub row: RowNum,
    pub col: ColNum,
    pub width: u32,
    pub height: u32,
    pub text: Option<String>,
    pub fill_color: Option<[u8; 3]>,
    pub outline_color: Option<[u8; 3]>,
    pub outline_width: Option<f64>,
    pub font_size: Option<f64>,
    pub font_bold: bool,
}

impl Shape {
    /// Create a new shape with the given type and dimensions (in pixels).
    pub fn new(shape_type: ShapeType, width: u32, height: u32) -> Self {
        Self {
            shape_type,
            row: 0,
            col: 0,
            width,
            height,
            text: None,
            fill_color: None,
            outline_color: None,
            outline_width: None,
            font_size: None,
            font_bold: false,
        }
    }

    /// Set the text body of the shape.
    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    /// Set the fill color as RGB.
    pub fn fill_color(mut self, rgb: [u8; 3]) -> Self {
        self.fill_color = Some(rgb);
        self
    }

    /// Set the outline color as RGB.
    pub fn outline_color(mut self, rgb: [u8; 3]) -> Self {
        self.outline_color = Some(rgb);
        self
    }

    /// Set the outline width in points.
    pub fn outline_width(mut self, width: f64) -> Self {
        self.outline_width = Some(width);
        self
    }

    /// Set the font size for the text body.
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Set the font to bold.
    pub fn bold(mut self) -> Self {
        self.font_bold = true;
        self
    }
}
