use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::utility::{ColNum, RowNum};
use crate::xml::xml_reader::get_attr;

/// A comment parsed from the comment XML part.
#[derive(Debug, Clone)]
pub struct ParsedComment {
    pub row: RowNum,
    pub col: ColNum,
    pub text: String,
    pub author: String,
}

/// Parse legacy comment XML (`xl/commentsN.xml`) into a list of comments.
///
/// The XML structure is:
/// ```xml
/// <comments xmlns="...">
///   <authors>
///     <author>Author Name</author>
///   </authors>
///   <commentList>
///     <comment ref="A1" authorId="0">
///       <text>
///         <r><t>Comment text</t></r>
///       </text>
///     </comment>
///   </commentList>
/// </comments>
/// ```
///
/// Returns an empty vec if the data is malformed or cannot be parsed.
pub fn parse_comments(data: &[u8]) -> Vec<ParsedComment> {
    parse_comments_inner(data).unwrap_or_default()
}

fn parse_comments_inner(data: &[u8]) -> crate::Result<Vec<ParsedComment>> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::new();

    let mut authors: Vec<String> = Vec::new();
    let mut comments = Vec::new();

    let mut in_authors = false;
    let mut in_author = false;
    let mut author_text = String::new();

    let mut in_comment = false;
    let mut in_text = false;
    let mut in_t = false;
    let mut cur_row = 0u32;
    let mut cur_col = 0u16;
    let mut cur_author_id = 0usize;
    let mut cur_text = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"authors" => {
                    in_authors = true;
                }
                b"author" if in_authors => {
                    in_author = true;
                    author_text.clear();
                }
                b"comment" => {
                    in_comment = true;
                    cur_text.clear();
                    // Parse the ref attribute to get cell coordinates
                    if let Some(cell_ref) =
                        get_attr(e.attributes(), b"ref").and_then(|v| std::str::from_utf8(v).ok())
                        && let Ok((r, c)) = crate::utility::parse_cell_ref(cell_ref)
                    {
                        cur_row = r;
                        cur_col = c;
                    }
                    // Parse the authorId attribute
                    cur_author_id = get_attr(e.attributes(), b"authorId")
                        .and_then(|v| atoi_simd::parse::<usize>(v).ok())
                        .unwrap_or(0);
                }
                b"text" if in_comment => {
                    in_text = true;
                }
                b"t" if in_text => {
                    in_t = true;
                }
                _ => {}
            },
            Event::Text(e) => {
                if in_author && let Ok(t) = e.unescape() {
                    author_text.push_str(&t);
                }
                if in_t && let Ok(t) = e.unescape() {
                    cur_text.push_str(&t);
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                b"authors" => {
                    in_authors = false;
                }
                b"author" => {
                    if in_author {
                        authors.push(author_text.clone());
                        in_author = false;
                    }
                }
                b"comment" => {
                    if in_comment {
                        let author = authors.get(cur_author_id).cloned().unwrap_or_default();
                        comments.push(ParsedComment {
                            row: cur_row,
                            col: cur_col,
                            text: cur_text.clone(),
                            author,
                        });
                        in_comment = false;
                    }
                }
                b"text" => {
                    in_text = false;
                }
                b"t" => {
                    in_t = false;
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(comments)
}

/// Find the comment file path for a sheet by looking at the sheet's relationships.
///
/// Comments are linked via a relationship of type
/// `http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments`
/// in the sheet's `.rels` file.
///
/// Returns `None` if no comment relationship is found.
pub fn find_comments_path_from_rels(sheet_rels_data: &[u8]) -> Option<String> {
    let rels = crate::reader::rel_parser::parse_rels(sheet_rels_data).ok()?;
    let comment_rel_type =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";
    for rel in &rels {
        if rel.rel_type == comment_rel_type {
            let target = &rel.target;
            let full_path = if let Some(stripped) = target.strip_prefix("../") {
                format!("xl/{}", stripped)
            } else if let Some(stripped) = target.strip_prefix("/xl/") {
                stripped.to_string()
            } else if target.starts_with("xl/") {
                target.to_string()
            } else {
                format!("xl/{target}")
            };
            return Some(full_path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comments_basic() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<comments xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <authors>
    <author>Alice</author>
    <author>Bob</author>
  </authors>
  <commentList>
    <comment ref="A1" authorId="0">
      <text><r><t>Hello from Alice</t></r></text>
    </comment>
    <comment ref="B2" authorId="1">
      <text><r><t>Hello from Bob</t></r></text>
    </comment>
  </commentList>
</comments>"#;

        let comments = parse_comments(xml);
        assert_eq!(comments.len(), 2);

        assert_eq!(comments[0].row, 0);
        assert_eq!(comments[0].col, 0);
        assert_eq!(comments[0].text, "Hello from Alice");
        assert_eq!(comments[0].author, "Alice");

        assert_eq!(comments[1].row, 1);
        assert_eq!(comments[1].col, 1);
        assert_eq!(comments[1].text, "Hello from Bob");
        assert_eq!(comments[1].author, "Bob");
    }

    #[test]
    fn test_parse_comments_plain_text() {
        // Comments can also have <text><t>...</t></text> without <r> wrapper
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<comments xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <authors><author>Test</author></authors>
  <commentList>
    <comment ref="C3" authorId="0">
      <text><t>Plain text comment</t></text>
    </comment>
  </commentList>
</comments>"#;

        let comments = parse_comments(xml);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].row, 2);
        assert_eq!(comments[0].col, 2);
        assert_eq!(comments[0].text, "Plain text comment");
        assert_eq!(comments[0].author, "Test");
    }

    #[test]
    fn test_parse_comments_malformed_returns_empty() {
        let bad_xml = b"this is not xml at all";
        let comments = parse_comments(bad_xml);
        assert!(comments.is_empty());
    }

    #[test]
    fn test_parse_comments_empty_file() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<comments xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <authors></authors>
  <commentList></commentList>
</comments>"#;

        let comments = parse_comments(xml);
        assert!(comments.is_empty());
    }

    #[test]
    fn test_find_comments_path_from_rels() {
        let rels_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="../comments1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/vmlDrawing" Target="../drawings/vmlDrawing1.vml"/>
</Relationships>"#;

        let path = find_comments_path_from_rels(rels_xml);
        assert_eq!(path, Some("xl/comments1.xml".to_string()));
    }

    #[test]
    fn test_find_comments_path_no_comments_rel() {
        let rels_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing" Target="../drawings/drawing1.xml"/>
</Relationships>"#;

        let path = find_comments_path_from_rels(rels_xml);
        assert!(path.is_none());
    }

    #[test]
    fn test_parse_comments_multi_run_text() {
        // Comments with multiple <r><t>...</t></r> runs should concatenate text
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<comments xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <authors><author>Author</author></authors>
  <commentList>
    <comment ref="A1" authorId="0">
      <text>
        <r><t>First part </t></r>
        <r><t>second part</t></r>
      </text>
    </comment>
  </commentList>
</comments>"#;

        let comments = parse_comments(xml);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "First part second part");
    }
}
