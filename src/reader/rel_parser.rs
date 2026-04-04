use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::xml::xml_reader::get_attr_str;

pub struct Rel {
    pub id: String,
    #[allow(dead_code)]
    pub rel_type: String,
    pub target: String,
}

pub fn parse_rels(data: &[u8]) -> crate::Result<Vec<Rel>> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(256);
    let mut rels = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) | Event::Empty(e)
                if e.local_name().as_ref() == b"Relationship" =>
            {
                let id = get_attr_str(e.attributes(), b"Id").unwrap_or("").to_string();
                let rel_type = get_attr_str(e.attributes(), b"Type").unwrap_or("").to_string();
                let target = get_attr_str(e.attributes(), b"Target").unwrap_or("").to_string();
                rels.push(Rel { id, rel_type, target });
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(rels)
}
