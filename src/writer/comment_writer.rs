use crate::utility::col_to_letter;
use crate::worksheet::Comment;
use crate::xml::xml_writer::XmlWriter;

pub fn write_comments_xml(comments: &[Comment]) -> Vec<u8> {
    let mut authors: Vec<&str> = Vec::new();
    for c in comments {
        if !authors.contains(&c.author.as_str()) { authors.push(&c.author); }
    }

    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("comments", &[("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main")]);
    w.start_tag("authors", &[]);
    for a in &authors { w.text_element("author", &[], a); }
    w.end_tag("authors");
    w.start_tag("commentList", &[]);
    for c in comments {
        let cell_ref = format!("{}{}", col_to_letter(c.col), c.row + 1);
        let author_id = authors.iter().position(|a| *a == c.author.as_str()).unwrap_or(0).to_string();
        w.start_tag("comment", &[("ref", &cell_ref), ("authorId", &author_id)]);
        w.start_tag("text", &[]);
        w.start_tag("r", &[]);
        w.text_element("t", &[], &c.text);
        w.end_tag("r");
        w.end_tag("text");
        w.end_tag("comment");
    }
    w.end_tag("commentList");
    w.end_tag("comments");
    w.into_bytes()
}

/// VML drawing XML for comment positioning (required by Excel).
pub fn write_vml_drawing(comments: &[Comment]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"<xml xmlns:v=\"urn:schemas-microsoft-com:vml\" xmlns:o=\"urn:schemas-microsoft-com:office:office\" xmlns:x=\"urn:schemas-microsoft-com:office:excel\">");
    buf.extend_from_slice(b"<o:shapelayout v:ext=\"edit\"><o:idmap v:ext=\"edit\" data=\"1\"/></o:shapelayout>");
    buf.extend_from_slice(b"<v:shapetype id=\"_x0000_t202\" coordsize=\"21600,21600\" o:spt=\"202\" path=\"m,l,21600r21600,l21600,xe\">");
    buf.extend_from_slice(b"<v:stroke joinstyle=\"miter\"/><v:path gradientshapeok=\"t\" o:connecttype=\"rect\"/></v:shapetype>");
    for (i, c) in comments.iter().enumerate() {
        let id = 1024 + i;
        let col = c.col as u32;
        let row = c.row as u32;
        buf.extend_from_slice(format!(
            "<v:shape id=\"_x0000_s{id}\" type=\"#_x0000_t202\" style=\"position:absolute;width:108pt;height:59.25pt;z-index:{i}\" fillcolor=\"#ffffe1\" o:insetmode=\"auto\">\
             <v:fill color2=\"#ffffe1\"/><v:shadow on=\"t\" color=\"black\" obscured=\"t\"/><v:path o:connecttype=\"none\"/>\
             <v:textbox/>\
             <x:ClientData ObjectType=\"Note\"><x:MoveWithCells/><x:SizeWithCells/>\
             <x:Anchor>{col}, 15, {row}, 10, {}, 31, {}, 4</x:Anchor>\
             <x:Row>{row}</x:Row><x:Column>{col}</x:Column>\
             </x:ClientData></v:shape>",
            col + 2, row + 4
        ).as_bytes());
    }
    buf.extend_from_slice(b"</xml>");
    buf
}
