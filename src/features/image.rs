use crate::utility::{ColNum, RowNum};

#[derive(Debug, Clone)]
pub struct Image {
    pub(crate) data: Vec<u8>,
    pub(crate) image_type: ImageType,
    pub(crate) width_px: u32,
    pub(crate) height_px: u32,
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
    pub(crate) scale_width: f64,
    pub(crate) scale_height: f64,
    // Accessibility metadata (Task 82)
    pub(crate) alt_text: Option<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType {
    Png,
    Jpeg,
}

impl ImageType {
    pub fn extension(&self) -> &str {
        match self {
            ImageType::Png => "png",
            ImageType::Jpeg => "jpeg",
        }
    }
    pub fn content_type(&self) -> &str {
        match self {
            ImageType::Png => "image/png",
            ImageType::Jpeg => "image/jpeg",
        }
    }
}

impl Image {
    pub fn from_path(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let data = std::fs::read(path.as_ref())?;
        Self::from_buffer(&data)
    }

    pub fn from_buffer(data: &[u8]) -> crate::Result<Self> {
        let (image_type, width_px, height_px) = detect_image(data)?;
        Ok(Self {
            data: data.to_vec(),
            image_type,
            width_px,
            height_px,
            row: 0,
            col: 0,
            scale_width: 1.0,
            scale_height: 1.0,
            alt_text: None,
        })
    }

    pub fn set_width(&mut self, px: u32) -> &mut Self {
        if self.width_px > 0 {
            self.scale_width = px as f64 / self.width_px as f64;
        }
        self
    }
    pub fn set_height(&mut self, px: u32) -> &mut Self {
        if self.height_px > 0 {
            self.scale_height = px as f64 / self.height_px as f64;
        }
        self
    }
    pub fn set_scale_width(&mut self, scale: f64) -> &mut Self {
        self.scale_width = scale;
        self
    }
    pub fn set_scale_height(&mut self, scale: f64) -> &mut Self {
        self.scale_height = scale;
        self
    }

    /// Set accessibility alt text (title and description) for this image.
    ///
    /// The title and description are serialized as `title` and `descr`
    /// attributes on the `<xdr:cNvPr>` element in the drawing XML.
    pub fn set_alt_text(&mut self, title: &str, description: &str) -> &mut Self {
        self.alt_text = Some((title.to_string(), description.to_string()));
        self
    }

    /// Returns the alt text (title, description) if set.
    pub fn alt_text(&self) -> Option<(&str, &str)> {
        self.alt_text
            .as_ref()
            .map(|(t, d)| (t.as_str(), d.as_str()))
    }
}

fn detect_image(data: &[u8]) -> crate::Result<(ImageType, u32, u32)> {
    if data.len() >= 24 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
        // PNG: width at offset 16, height at offset 20 (big-endian u32)
        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        return Ok((ImageType::Png, w, h));
    }
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
        // JPEG: scan for SOF0 marker (0xFF 0xC0) to get dimensions
        let (w, h) = jpeg_dimensions(data)?;
        return Ok((ImageType::Jpeg, w, h));
    }
    Err(crate::Error::InvalidData(
        "Unsupported image format (only PNG and JPEG supported)".into(),
    ))
}

fn jpeg_dimensions(data: &[u8]) -> crate::Result<(u32, u32)> {
    let mut i = 2;
    while i + 1 < data.len() {
        if data[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = data[i + 1];
        if marker == 0xD9 {
            break;
        } // EOI
        if marker == 0xC0 || marker == 0xC2 {
            // SOF0 or SOF2: height at +5, width at +7
            if i + 9 < data.len() {
                let h = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                let w = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                return Ok((w, h));
            }
        }
        if i + 3 < data.len() {
            let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            i += 2 + len;
        } else {
            break;
        }
    }
    Err(crate::Error::InvalidData(
        "Could not determine JPEG dimensions".into(),
    ))
}
