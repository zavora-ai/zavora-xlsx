use zavora_xlsx::Workbook;

fn main() {
    // Create a workbook with comments
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Revenue").unwrap();
    ws.write(0, 1, 50000.0).unwrap();
    ws.write(1, 0, "Expenses").unwrap();
    ws.write(1, 1, 30000.0).unwrap();
    ws.write(2, 0, "Profit").unwrap();
    ws.write(2, 1, 20000.0).unwrap();

    // Add comments with different authors
    ws.add_comment_with_author(0, 1, "Q4 revenue figure, includes deferred income", "Finance Team");
    ws.add_comment_with_author(1, 1, "Excludes one-time restructuring costs", "Controller");
    ws.add_comment(2, 1, "Net profit before tax");

    // Save to buffer and read back
    let buf = wb.save_to_buffer().unwrap();
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();

    println!("=== Comments on Sheet: {} ===\n", ws2.name());

    let comments = ws2.comments();
    println!("Total comments: {}\n", comments.len());

    for comment in comments {
        let col_letter = (b'A' + comment.col as u8) as char;
        let cell_ref = format!("{}{}", col_letter, comment.row + 1);
        println!("Cell {cell_ref}:");
        println!("  Author: {}", comment.author);
        println!("  Text:   {}", comment.text);
        println!();
    }

    // Demonstrate get_comment accessor
    if let Some((author, text)) = ws2.get_comment(0, 1) {
        println!("--- Direct lookup for B1 ---");
        println!("  Author: {author}");
        println!("  Text:   {text}");
    }
}
