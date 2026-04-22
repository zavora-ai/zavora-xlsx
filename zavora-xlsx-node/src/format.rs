use std::cell::RefCell;
use napi_derive::napi;

#[napi]
pub struct Format {
    pub(crate) inner: RefCell<zavora_xlsx::Format>,
}

#[napi]
impl Format {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(zavora_xlsx::Format::new()),
        }
    }

    #[napi]
    pub fn bold(&self) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.bold();
        }
        self
    }

    #[napi]
    pub fn italic(&self) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.italic();
        }
        self
    }

    #[napi]
    pub fn strikethrough(&self) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.strikethrough();
        }
        self
    }

    #[napi]
    pub fn text_wrap(&self) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.text_wrap();
        }
        self
    }

    #[napi]
    pub fn shrink_to_fit(&self) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.shrink_to_fit();
        }
        self
    }

    #[napi]
    pub fn font_size(&self, size: f64) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.font_size(size);
        }
        self
    }

    #[napi]
    pub fn font_name(&self, name: String) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.font_name(&name);
        }
        self
    }

    #[napi]
    pub fn font_color(&self, hex: String) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.font_color(hex.as_str());
        }
        self
    }

    #[napi]
    pub fn background_color(&self, hex: String) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.background_color(hex.as_str());
        }
        self
    }

    #[napi]
    pub fn num_format(&self, format: String) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.num_format(&format);
        }
        self
    }

    #[napi]
    pub fn indent(&self, level: u8) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.indent(level);
        }
        self
    }

    #[napi]
    pub fn rotation(&self, angle: i16) -> &Self {
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.rotation(angle);
        }
        self
    }

    #[napi]
    pub fn underline(&self, style: String) -> &Self {
        let u = match style.as_str() {
            "double" => zavora_xlsx::Underline::Double,
            _ => zavora_xlsx::Underline::Single,
        };
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.underline(u);
        }
        self
    }

    #[napi]
    pub fn border(&self, style: String) -> &Self {
        let bs = match style.as_str() {
            "none" => zavora_xlsx::BorderStyle::None,
            "thin" => zavora_xlsx::BorderStyle::Thin,
            "medium" => zavora_xlsx::BorderStyle::Medium,
            "thick" => zavora_xlsx::BorderStyle::Thick,
            "double" => zavora_xlsx::BorderStyle::Double,
            "dashed" => zavora_xlsx::BorderStyle::Dashed,
            "dotted" => zavora_xlsx::BorderStyle::Dotted,
            _ => zavora_xlsx::BorderStyle::Thin,
        };
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.border(bs);
        }
        self
    }

    #[napi]
    pub fn align(&self, alignment: String) -> &Self {
        let a = match alignment.as_str() {
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
        {
            let old = self.inner.borrow().clone();
            *self.inner.borrow_mut() = old.align(a);
        }
        self
    }
}
