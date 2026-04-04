use std::fmt::Write;

/// Buffer-based XML writer using String.
pub struct XmlWriter {
    buf: String,
}

impl XmlWriter {
    pub fn new() -> Self {
        Self { buf: String::with_capacity(4096) }
    }

    pub fn declaration(&mut self) {
        self.buf.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
    }

    pub fn start_tag(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.buf.push('<');
        self.buf.push_str(name);
        for (k, v) in attrs {
            self.buf.push(' ');
            self.buf.push_str(k);
            self.buf.push_str("=\"");
            escape_attr_into(&mut self.buf, v);
            self.buf.push('"');
        }
        self.buf.push('>');
    }

    pub fn empty_tag(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.buf.push('<');
        self.buf.push_str(name);
        for (k, v) in attrs {
            self.buf.push(' ');
            self.buf.push_str(k);
            self.buf.push_str("=\"");
            escape_attr_into(&mut self.buf, v);
            self.buf.push('"');
        }
        self.buf.push_str("/>");
    }

    pub fn end_tag(&mut self, name: &str) {
        let _ = write!(self.buf, "</{name}>");
    }

    pub fn text(&mut self, content: &str) {
        escape_text_into(&mut self.buf, content);
    }

    /// Write <name attrs...>text</name> in one call.
    pub fn text_element(&mut self, name: &str, attrs: &[(&str, &str)], content: &str) {
        self.start_tag(name, attrs);
        self.text(content);
        self.end_tag(name);
    }

    pub fn raw(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf.into_bytes()
    }
}

fn escape_attr_into(buf: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => buf.push_str("&amp;"),
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            '"' => buf.push_str("&quot;"),
            _ => buf.push(c),
        }
    }
}

fn escape_text_into(buf: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => buf.push_str("&amp;"),
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            _ => buf.push(c),
        }
    }
}
