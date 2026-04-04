use crate::xml::xml_writer::XmlWriter;

/// Document properties (docProps/core.xml + app.xml).
#[derive(Debug, Clone, Default)]
pub struct DocProperties {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub category: Option<String>,
    pub company: Option<String>,
}

impl DocProperties {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: &str) -> Self { self.title = Some(t.into()); self }
    pub fn author(mut self, a: &str) -> Self { self.author = Some(a.into()); self }
    pub fn subject(mut self, s: &str) -> Self { self.subject = Some(s.into()); self }
    pub fn description(mut self, d: &str) -> Self { self.description = Some(d.into()); self }
    pub fn keywords(mut self, k: &str) -> Self { self.keywords = Some(k.into()); self }
    pub fn category(mut self, c: &str) -> Self { self.category = Some(c.into()); self }
    pub fn company(mut self, c: &str) -> Self { self.company = Some(c.into()); self }
}

pub fn write_core_xml(props: &DocProperties) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("cp:coreProperties", &[
        ("xmlns:cp", "http://schemas.openxmlformats.org/package/2006/metadata/core-properties"),
        ("xmlns:dc", "http://purl.org/dc/elements/1.1/"),
        ("xmlns:dcterms", "http://purl.org/dc/terms/"),
    ]);
    if let Some(ref t) = props.title { w.text_element("dc:title", &[], t); }
    if let Some(ref s) = props.subject { w.text_element("dc:subject", &[], s); }
    if let Some(ref a) = props.author { w.text_element("dc:creator", &[], a); }
    if let Some(ref d) = props.description { w.text_element("dc:description", &[], d); }
    if let Some(ref k) = props.keywords { w.text_element("cp:keywords", &[], k); }
    if let Some(ref c) = props.category { w.text_element("cp:category", &[], c); }
    w.end_tag("cp:coreProperties");
    w.into_bytes()
}

pub fn write_app_xml(props: &DocProperties) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("Properties", &[
        ("xmlns", "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"),
    ]);
    w.text_element("Application", &[], "zavora-xlsx");
    if let Some(ref c) = props.company { w.text_element("Company", &[], c); }
    w.end_tag("Properties");
    w.into_bytes()
}

pub fn parse_core_xml(data: &[u8]) -> DocProperties {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(256);
    let mut props = DocProperties::new();
    let mut current_tag = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                current_tag = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
            }
            Ok(Event::Text(e)) => {
                if let Ok(text) = e.unescape() {
                    let text = text.to_string();
                    match current_tag.as_str() {
                        "title" => props.title = Some(text),
                        "subject" => props.subject = Some(text),
                        "creator" => props.author = Some(text),
                        "description" => props.description = Some(text),
                        "keywords" => props.keywords = Some(text),
                        "category" => props.category = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(_)) => { current_tag.clear(); }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }
    props
}
