use zavora_xlsx::Workbook;

#[test]
fn test_roundtrip_comments_basic() {
    // Write comments to a file
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Cell A1").unwrap();
    ws.write(1, 1, "Cell B2").unwrap();
    ws.write(2, 2, "Cell C3").unwrap();
    ws.add_comment(0, 0, "This is a comment on A1");
    ws.add_comment_with_author(1, 1, "Comment by Bob", "Bob");
    ws.add_comment_with_author(2, 2, "Comment by Alice", "Alice");

    let buf = wb.save_to_buffer().unwrap();

    // Read back and verify
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let comments = ws2.comments();

    assert_eq!(
        comments.len(),
        3,
        "Expected 3 comments, got {}",
        comments.len()
    );

    // Verify first comment (default author)
    assert_eq!(comments[0].row, 0);
    assert_eq!(comments[0].col, 0);
    assert_eq!(comments[0].text, "This is a comment on A1");
    assert_eq!(comments[0].author, "Author");

    // Verify second comment
    assert_eq!(comments[1].row, 1);
    assert_eq!(comments[1].col, 1);
    assert_eq!(comments[1].text, "Comment by Bob");
    assert_eq!(comments[1].author, "Bob");

    // Verify third comment
    assert_eq!(comments[2].row, 2);
    assert_eq!(comments[2].col, 2);
    assert_eq!(comments[2].text, "Comment by Alice");
    assert_eq!(comments[2].author, "Alice");
}

#[test]
fn test_roundtrip_comments_get_comment() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Hello").unwrap();
    ws.add_comment_with_author(0, 0, "A note", "Reviewer");

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();

    // Test get_comment accessor
    let comment = ws2.get_comment(0, 0);
    assert!(comment.is_some());
    let (author, text) = comment.unwrap();
    assert_eq!(author, "Reviewer");
    assert_eq!(text, "A note");

    // Non-existent comment
    assert!(ws2.get_comment(5, 5).is_none());
}

#[test]
fn test_roundtrip_comments_multiple_sheets() {
    let mut wb = Workbook::new();

    // Sheet 1 with comments
    let ws1 = wb.worksheet(0).unwrap();
    ws1.write(0, 0, "Sheet1 A1").unwrap();
    ws1.add_comment_with_author(0, 0, "Comment on sheet 1", "Author1");

    // Sheet 2 with comments
    let ws2 = wb.add_worksheet();
    ws2.write(0, 0, "Sheet2 A1").unwrap();
    ws2.add_comment_with_author(0, 0, "Comment on sheet 2", "Author2");

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();

    // Verify sheet 1 comments
    let ws1_ref = wb2.worksheet_ref(0).unwrap();
    let comments1 = ws1_ref.comments();
    assert_eq!(comments1.len(), 1);
    assert_eq!(comments1[0].text, "Comment on sheet 1");
    assert_eq!(comments1[0].author, "Author1");

    // Verify sheet 2 comments
    let ws2_ref = wb2.worksheet_ref(1).unwrap();
    let comments2 = ws2_ref.comments();
    assert_eq!(comments2.len(), 1);
    assert_eq!(comments2[0].text, "Comment on sheet 2");
    assert_eq!(comments2[0].author, "Author2");
}

#[test]
fn test_no_comments_sheet() {
    // A sheet with no comments should have an empty comments list
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "No comments here").unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    assert!(ws2.comments().is_empty());
}

#[test]
fn test_roundtrip_comments_same_author_multiple_comments() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "A1").unwrap();
    ws.write(1, 0, "A2").unwrap();
    ws.add_comment_with_author(0, 0, "First comment", "SharedAuthor");
    ws.add_comment_with_author(1, 0, "Second comment", "SharedAuthor");

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let comments = ws2.comments();

    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].author, "SharedAuthor");
    assert_eq!(comments[0].text, "First comment");
    assert_eq!(comments[1].author, "SharedAuthor");
    assert_eq!(comments[1].text, "Second comment");
}
