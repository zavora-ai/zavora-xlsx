use pyo3::prelude::*;

#[pyclass]
pub struct Format {
    pub(crate) inner: zavora_xlsx::Format,
}

#[pymethods]
impl Format {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: zavora_xlsx::Format::new(),
        }
    }

    pub fn bold(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().bold();
        slf
    }

    pub fn italic(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().italic();
        slf
    }

    pub fn strikethrough(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().strikethrough();
        slf
    }

    pub fn text_wrap(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().text_wrap();
        slf
    }

    pub fn shrink_to_fit(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().shrink_to_fit();
        slf
    }

    pub fn font_size(mut slf: PyRefMut<'_, Self>, size: f64) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().font_size(size);
        slf
    }

    pub fn font_name<'a>(mut slf: PyRefMut<'a, Self>, name: &'a str) -> PyRefMut<'a, Self> {
        slf.inner = slf.inner.clone().font_name(name);
        slf
    }

    pub fn font_color<'a>(mut slf: PyRefMut<'a, Self>, hex: &'a str) -> PyRefMut<'a, Self> {
        slf.inner = slf.inner.clone().font_color(hex);
        slf
    }

    pub fn background_color<'a>(mut slf: PyRefMut<'a, Self>, hex: &'a str) -> PyRefMut<'a, Self> {
        slf.inner = slf.inner.clone().background_color(hex);
        slf
    }

    pub fn num_format<'a>(mut slf: PyRefMut<'a, Self>, format: &'a str) -> PyRefMut<'a, Self> {
        slf.inner = slf.inner.clone().num_format(format);
        slf
    }

    pub fn indent(mut slf: PyRefMut<'_, Self>, level: u8) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().indent(level);
        slf
    }

    pub fn rotation(mut slf: PyRefMut<'_, Self>, angle: i16) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().rotation(angle);
        slf
    }

    pub fn underline<'a>(mut slf: PyRefMut<'a, Self>, style: &'a str) -> PyRefMut<'a, Self> {
        let u = match style {
            "double" => zavora_xlsx::Underline::Double,
            _ => zavora_xlsx::Underline::Single,
        };
        slf.inner = slf.inner.clone().underline(u);
        slf
    }

    pub fn border<'a>(mut slf: PyRefMut<'a, Self>, style: &'a str) -> PyRefMut<'a, Self> {
        let bs = match style {
            "none" => zavora_xlsx::BorderStyle::None,
            "thin" => zavora_xlsx::BorderStyle::Thin,
            "medium" => zavora_xlsx::BorderStyle::Medium,
            "thick" => zavora_xlsx::BorderStyle::Thick,
            "double" => zavora_xlsx::BorderStyle::Double,
            "dashed" => zavora_xlsx::BorderStyle::Dashed,
            "dotted" => zavora_xlsx::BorderStyle::Dotted,
            _ => zavora_xlsx::BorderStyle::Thin,
        };
        slf.inner = slf.inner.clone().border(bs);
        slf
    }

    pub fn align<'a>(mut slf: PyRefMut<'a, Self>, alignment: &'a str) -> PyRefMut<'a, Self> {
        let a = match alignment {
            "left" => zavora_xlsx::Align::Left,
            "center" => zavora_xlsx::Align::Center,
            "right" => zavora_xlsx::Align::Right,
            "fill" => zavora_xlsx::Align::Fill,
            "justify" => zavora_xlsx::Align::Justify,
            "top" => zavora_xlsx::Align::Top,
            "middle" => zavora_xlsx::Align::VerticalCenter,
            "bottom" => zavora_xlsx::Align::Bottom,
            _ => zavora_xlsx::Align::Left,
        };
        slf.inner = slf.inner.clone().align(a);
        slf
    }
}
