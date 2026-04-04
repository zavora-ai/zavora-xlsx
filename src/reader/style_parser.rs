use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::xml::xml_reader::get_attr;

pub struct ParsedStyles {
    pub num_formats: Vec<(u16, String)>,
    pub xf_num_fmt_ids: Vec<u16>,
}

impl Default for ParsedStyles {
    fn default() -> Self {
        Self { num_formats: Vec::new(), xf_num_fmt_ids: vec![0] }
    }
}

pub fn parse_styles(data: &[u8]) -> crate::Result<ParsedStyles> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(512);

    let mut num_formats = Vec::new();
    let mut xf_num_fmt_ids = Vec::new();
    let mut in_cell_xfs = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) | Event::Empty(e) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"numFmt" => {
                        let id = get_attr(e.attributes(), b"numFmtId")
                            .and_then(|v| std::str::from_utf8(v).ok())
                            .and_then(|v| v.parse::<u16>().ok())
                            .unwrap_or(0);
                        let code = get_attr(e.attributes(), b"formatCode")
                            .and_then(|v| std::str::from_utf8(v).ok())
                            .unwrap_or("")
                            .to_string();
                        num_formats.push((id, code));
                    }
                    b"cellXfs" => { in_cell_xfs = true; }
                    b"xf" if in_cell_xfs => {
                        let id = get_attr(e.attributes(), b"numFmtId")
                            .and_then(|v| std::str::from_utf8(v).ok())
                            .and_then(|v| v.parse::<u16>().ok())
                            .unwrap_or(0);
                        xf_num_fmt_ids.push(id);
                    }
                    _ => {}
                }
            }
            Event::End(e) if e.local_name().as_ref() == b"cellXfs" => {
                in_cell_xfs = false;
            }
            Event::Eof => break,
            _ => {}
        }
    }

    if xf_num_fmt_ids.is_empty() {
        xf_num_fmt_ids.push(0);
    }

    Ok(ParsedStyles { num_formats, xf_num_fmt_ids })
}
