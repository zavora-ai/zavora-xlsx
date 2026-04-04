use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;

use crate::model::shared_strings::SharedStringTable;
use crate::xml::xml_reader::get_attr;

pub fn parse_sst(data: &[u8]) -> crate::Result<SharedStringTable> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(1024);
    let mut sst = SharedStringTable::new();

    // Find <sst> and pre-allocate
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if e.local_name().as_ref() == b"sst" => break,
            Event::Eof => return Ok(sst),
            _ => {}
        }
    }

    // Read <si> elements
    let mut in_si = false;
    let mut in_rph = false;
    let mut current = String::new();
    let mut text_buf = Vec::with_capacity(256);

    loop {
        text_buf.clear();
        match reader.read_event_into(&mut text_buf)? {
            Event::Start(e) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"si" => { in_si = true; current.clear(); }
                    b"rPh" => { in_rph = true; }
                    _ => {}
                }
            }
            Event::End(e) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"si" => {
                        sst.push(&current);
                        in_si = false;
                    }
                    b"rPh" => { in_rph = false; }
                    _ => {}
                }
            }
            Event::Text(e) if in_si && !in_rph => {
                if let Ok(text) = e.unescape() {
                    current.push_str(&text);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(sst)
}
