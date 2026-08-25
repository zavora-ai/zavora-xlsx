use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesRef, BytesText, Event};
use quick_xml::name::QName;
use quick_xml::reader::Reader as XmlReaderInner;

/// Compatibility helper for decoding and unescaping text events.
///
/// `quick-xml` 0.41 resolves entities while reading and replaced the former
/// `BytesText::unescape` operation with encoding-aware decoding. Keeping the
/// compatibility name here avoids duplicating that migration across readers.
pub(crate) trait BytesTextExt {
    fn unescape(&self) -> Result<String, String>;
}

impl BytesTextExt for BytesText<'_> {
    fn unescape(&self) -> Result<String, String> {
        self.decode()
            .map(std::borrow::Cow::into_owned)
            .map_err(|error| error.to_string())
    }
}

/// Resolve a character or general entity-reference event into text.
pub(crate) fn decode_xml_ref(reference: &BytesRef<'_>) -> Result<String, String> {
    if let Some(character) = reference
        .resolve_char_ref()
        .map_err(|error| error.to_string())?
    {
        return Ok(character.to_string());
    }

    let name = reference.decode().map_err(|error| error.to_string())?;
    Ok(match name.as_ref() {
        "lt" => "<".to_string(),
        "gt" => ">".to_string(),
        "amp" => "&".to_string(),
        "apos" => "'".to_string(),
        "quot" => "\"".to_string(),
        other => format!("&{other};"),
    })
}

/// Streaming XML reader with buffer reuse.
#[allow(dead_code)]
pub struct XmlReader<'a> {
    inner: XmlReaderInner<&'a [u8]>,
    buf: Vec<u8>,
}

#[allow(dead_code)]
impl<'a> XmlReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let mut inner = XmlReaderInner::from_reader(data);
        let config = inner.config_mut();
        config.check_end_names = false;
        config.trim_text(false);
        config.check_comments = false;
        config.expand_empty_elements = true;
        Self {
            inner,
            buf: Vec::with_capacity(1024),
        }
    }

    /// Read next event, reusing internal buffer.
    pub fn next_event(&mut self) -> crate::Result<Event<'_>> {
        self.buf.clear();
        Ok(self.inner.read_event_into(&mut self.buf)?)
    }

    /// Get the underlying reader for position info.
    pub fn reader(&self) -> &XmlReaderInner<&'a [u8]> {
        &self.inner
    }
}

/// Extract a single attribute value by QName from an attribute iterator.
pub fn get_attr<'a>(attrs: Attributes<'a>, name: &[u8]) -> Option<&'a [u8]> {
    for attr in attrs.into_iter().flatten() {
        if attr.key == QName(name)
            && let std::borrow::Cow::Borrowed(v) = attr.value
        {
            return Some(v);
        }
    }
    None
}

/// Extract attribute value as a string slice.
pub fn get_attr_str<'a>(attrs: Attributes<'a>, name: &[u8]) -> Option<&'a str> {
    get_attr(attrs, name).and_then(|v| std::str::from_utf8(v).ok())
}
