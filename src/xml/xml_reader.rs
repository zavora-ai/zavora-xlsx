use quick_xml::events::Event;
use quick_xml::events::attributes::Attributes;
use quick_xml::name::QName;
use quick_xml::reader::Reader as XmlReaderInner;

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
