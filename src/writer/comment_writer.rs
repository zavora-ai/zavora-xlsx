use crate::utility::col_to_letter;
use crate::worksheet::Comment;
use crate::xml::xml_writer::XmlWriter;

pub fn write_comments_xml(comments: &[Comment]) -> Vec<u8> {
    let mut authors: Vec<&str> = Vec::new();
    for c in comments {
        if !authors.contains(&c.author.as_str()) {
            authors.push(&c.author);
        }
    }

    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "comments",
        &[(
            "xmlns",
            "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
        )],
    );
    w.start_tag("authors", &[]);
    for a in &authors {
        w.text_element("author", &[], a);
    }
    w.end_tag("authors");
    w.start_tag("commentList", &[]);
    for c in comments {
        let cell_ref = format!("{}{}", col_to_letter(c.col), c.row + 1);
        let author_id = authors
            .iter()
            .position(|a| *a == c.author.as_str())
            .unwrap_or(0)
            .to_string();
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
    buf.extend_from_slice(
        b"<o:shapelayout v:ext=\"edit\"><o:idmap v:ext=\"edit\" data=\"1\"/></o:shapelayout>",
    );
    buf.extend_from_slice(b"<v:shapetype id=\"_x0000_t202\" coordsize=\"21600,21600\" o:spt=\"202\" path=\"m,l,21600r21600,l21600,xe\">");
    buf.extend_from_slice(b"<v:stroke joinstyle=\"miter\"/><v:path gradientshapeok=\"t\" o:connecttype=\"rect\"/></v:shapetype>");
    for (i, c) in comments.iter().enumerate() {
        let id = 1024 + i;
        let col = c.col as u32;
        let row = c.row;
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

/// Serialize threaded comments XML part (`xl/threadedComments/threadedComment{N}.xml`).
pub fn write_threaded_comments_xml(comments: &[crate::worksheet::ThreadedComment]) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "ThreadedComments",
        &[(
            "xmlns",
            "http://schemas.microsoft.com/spreadsheetml/2018/threadedcomments",
        )],
    );
    for (i, tc) in comments.iter().enumerate() {
        let cell_ref = format!("{}{}", col_to_letter(tc.col), tc.row + 1);
        let id = format!("{{TC{:04X}-0000-0000-0000-000000000000}}", i);
        w.start_tag(
            "threadedComment",
            &[
                ("ref", &cell_ref),
                ("personId", &format!("{{P{:04X}}}", 0)),
                ("id", &id),
                ("dT", &tc.timestamp),
            ],
        );
        w.start_tag("text", &[]);
        w.text(&tc.text);
        w.end_tag("text");
        w.end_tag("threadedComment");

        // Write replies
        for (ri, reply) in tc.replies.iter().enumerate() {
            let reply_id = format!("{{TC{:04X}-{:04X}-0000-0000-000000000000}}", i, ri + 1);
            w.start_tag(
                "threadedComment",
                &[
                    ("ref", &cell_ref),
                    ("personId", &format!("{{P{:04X}}}", 1)),
                    ("id", &reply_id),
                    ("parentId", &id),
                    ("dT", &reply.timestamp),
                ],
            );
            w.start_tag("text", &[]);
            w.text(&reply.text);
            w.end_tag("text");
            w.end_tag("threadedComment");
        }
    }
    w.end_tag("ThreadedComments");
    w.into_bytes()
}

/// Serialize person list XML part (`xl/persons/person.xml`).
pub fn write_persons_xml(comments: &[crate::worksheet::ThreadedComment]) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "personList",
        &[(
            "xmlns",
            "http://schemas.microsoft.com/spreadsheetml/2018/threadedcomments",
        )],
    );

    // Collect unique authors
    let mut authors: Vec<String> = Vec::new();
    for tc in comments {
        if !authors.contains(&tc.author) {
            authors.push(tc.author.clone());
        }
        for reply in &tc.replies {
            if !authors.contains(&reply.author) {
                authors.push(reply.author.clone());
            }
        }
    }

    for (i, author) in authors.iter().enumerate() {
        let id = format!("{{P{:04X}}}", i);
        w.empty_tag(
            "person",
            &[
                ("displayName", author),
                ("id", &id),
                ("userId", author),
                ("providerId", "None"),
            ],
        );
    }
    w.end_tag("personList");
    w.into_bytes()
}

/// Serialize form controls as VML drawing XML.
pub fn write_form_controls_vml(
    controls: &[(
        crate::utility::RowNum,
        crate::utility::ColNum,
        crate::worksheet::FormControl,
    )],
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"<xml xmlns:v=\"urn:schemas-microsoft-com:vml\" xmlns:o=\"urn:schemas-microsoft-com:office:office\" xmlns:x=\"urn:schemas-microsoft-com:office:excel\">");
    buf.extend_from_slice(
        b"<o:shapelayout v:ext=\"edit\"><o:idmap v:ext=\"edit\" data=\"1\"/></o:shapelayout>",
    );

    for (i, (row, col, control)) in controls.iter().enumerate() {
        let id = 2048 + i;
        let col = *col as u32;
        let row = *row;
        match control {
            crate::worksheet::FormControl::Checkbox {
                text,
                checked,
                cell_link,
            } => {
                let checked_val = if *checked { "Checked" } else { "Unchecked" };
                let fmla_link = cell_link.as_deref().unwrap_or("");
                buf.extend_from_slice(format!(
                    "<v:shape id=\"_x0000_s{id}\" type=\"#_x0000_t201\" style=\"position:absolute;width:72pt;height:18pt;z-index:{i}\">\
                     <v:textbox><div>{text}</div></v:textbox>\
                     <x:ClientData ObjectType=\"Checkbox\">\
                     <x:Anchor>{col}, 0, {row}, 0, {}, 0, {}, 0</x:Anchor>\
                     <x:Checked>{checked_val}</x:Checked>",
                    col + 2, row + 1
                ).as_bytes());
                if !fmla_link.is_empty() {
                    buf.extend_from_slice(
                        format!("<x:FmlaLink>{fmla_link}</x:FmlaLink>").as_bytes(),
                    );
                }
                buf.extend_from_slice(b"</x:ClientData></v:shape>");
            }
            crate::worksheet::FormControl::Dropdown {
                items,
                selected_index,
                cell_link,
            } => {
                let sel = selected_index.unwrap_or(0);
                let fmla_link = cell_link.as_deref().unwrap_or("");
                let _ = items; // Items are typically linked via a range
                buf.extend_from_slice(format!(
                    "<v:shape id=\"_x0000_s{id}\" type=\"#_x0000_t201\" style=\"position:absolute;width:108pt;height:18pt;z-index:{i}\">\
                     <x:ClientData ObjectType=\"Drop\">\
                     <x:Anchor>{col}, 0, {row}, 0, {}, 0, {}, 0</x:Anchor>\
                     <x:Sel>{sel}</x:Sel>",
                    col + 3, row + 1
                ).as_bytes());
                if !fmla_link.is_empty() {
                    buf.extend_from_slice(
                        format!("<x:FmlaLink>{fmla_link}</x:FmlaLink>").as_bytes(),
                    );
                }
                buf.extend_from_slice(b"</x:ClientData></v:shape>");
            }
            crate::worksheet::FormControl::Button { text, macro_name } => {
                let macro_attr = macro_name.as_deref().unwrap_or("");
                buf.extend_from_slice(format!(
                    "<v:shape id=\"_x0000_s{id}\" type=\"#_x0000_t201\" style=\"position:absolute;width:72pt;height:24pt;z-index:{i}\">\
                     <v:textbox><div>{text}</div></v:textbox>\
                     <x:ClientData ObjectType=\"Button\">\
                     <x:Anchor>{col}, 0, {row}, 0, {}, 0, {}, 0</x:Anchor>\
                     <x:FmlaMacro>{macro_attr}</x:FmlaMacro>\
                     </x:ClientData></v:shape>",
                    col + 2, row + 1
                ).as_bytes());
            }
            crate::worksheet::FormControl::Spinner {
                min_value,
                max_value,
                current_value,
                increment,
                cell_link,
            } => {
                let fmla_link = cell_link.as_deref().unwrap_or("");
                buf.extend_from_slice(format!(
                    "<v:shape id=\"_x0000_s{id}\" type=\"#_x0000_t201\" style=\"position:absolute;width:18pt;height:36pt;z-index:{i}\">\
                     <x:ClientData ObjectType=\"Spin\">\
                     <x:Anchor>{col}, 0, {row}, 0, {}, 0, {}, 0</x:Anchor>\
                     <x:Min>{min_value}</x:Min><x:Max>{max_value}</x:Max>\
                     <x:Val>{current_value}</x:Val><x:Inc>{increment}</x:Inc>",
                    col + 1, row + 2
                ).as_bytes());
                if !fmla_link.is_empty() {
                    buf.extend_from_slice(
                        format!("<x:FmlaLink>{fmla_link}</x:FmlaLink>").as_bytes(),
                    );
                }
                buf.extend_from_slice(b"</x:ClientData></v:shape>");
            }
        }
    }
    buf.extend_from_slice(b"</xml>");
    buf
}
