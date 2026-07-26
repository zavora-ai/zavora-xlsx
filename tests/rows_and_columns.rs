//! Inserting a row must make room, not make a copy.
//!
//! Inserting one row at row 6 of a saved workbook left rows 6 and 7 both holding what row 6 had.
//! The cells below moved down correctly, so the shift itself worked — what stayed behind was the
//! original, which is the shape of a copy rather than a move.

use zavora_xlsx::Workbook;

fn a_saved_sheet(path: &str) {
    let _ = std::fs::remove_file(path);
    let mut book = Workbook::new();
    {
        // `Workbook::new` already has a sheet. Adding another and writing to it, then reading
        // the first, is a fixture that tests nothing.
        let sheet = book.worksheet(0).unwrap();
        for row in 0..4u32 {
            // Numbers, because a string read back from a saved file comes through the shared
            // string table and this test is about positions, not about strings.
            sheet.write(row, 0, (row + 1) as f64 * 10.0).unwrap();
        }
    }
    book.save(path).unwrap();
}

fn column_a(path: &str) -> Vec<String> {
    // Read the way the application reads: open_readonly and worksheet_ref, which is the path
    // that keeps the parsed cells available without deserialising them first.
    let book = Workbook::open_readonly(path).unwrap();
    let sheet = book.worksheet_ref(0).unwrap();
    (0..8)
        .map(|row| match sheet.read_cell(row, 0) {
            zavora_xlsx::CellValue::Number(number) => format!("{number:.0}"),
            _ => String::new(),
        })
        .collect()
}

#[test]
fn inserting_a_row_makes_room_rather_than_a_copy() {
    let path = "/tmp/zavora-insert-room.xlsx";
    a_saved_sheet(path);
    assert_eq!(
        column_a(path)[..4],
        ["10", "20", "30", "40"],
        "the fixture should read plainly"
    );

    let mut book = Workbook::open(path).unwrap();
    // Row 2, zero-based, which is "row2".
    book.worksheet(0).unwrap().insert_rows(1, 1).unwrap();
    book.save(path).unwrap();

    let after = column_a(path);
    assert_eq!(
        after[..5],
        ["10", "", "20", "30", "40"],
        "inserting should leave a blank row, not a second copy: {after:?}"
    );
}

#[test]
fn deleting_a_row_closes_the_gap() {
    let path = "/tmp/zavora-delete-gap.xlsx";
    a_saved_sheet(path);

    let mut book = Workbook::open(path).unwrap();
    book.worksheet(0).unwrap().remove_rows(1, 1).unwrap();
    book.save(path).unwrap();

    let after = column_a(path);
    assert_eq!(
        after[..3],
        ["10", "30", "40"],
        "deleting should close the gap: {after:?}"
    );
}

/// In memory, before any save, the same must hold — so a failure can be told apart from one that
/// only appears on the way to disk.
#[test]
fn the_same_holds_before_it_is_ever_saved() {
    let mut book = Workbook::new();
    {
        // `Workbook::new` already has a sheet. Adding another and writing to it, then reading
        // the first, is a fixture that tests nothing.
        let sheet = book.worksheet(0).unwrap();
        for row in 0..4u32 {
            // Numbers, because a string read back from a saved file comes through the shared
            // string table and this test is about positions, not about strings.
            sheet.write(row, 0, (row + 1) as f64 * 10.0).unwrap();
        }
        sheet.insert_rows(1, 1).unwrap();

        let read = |row: u32| match sheet.read_cell(row, 0) {
            zavora_xlsx::CellValue::Number(number) => format!("{number:.0}"),
            _ => String::new(),
        };
        assert_eq!(read(0), "10");
        assert_eq!(read(1), "", "the new row should be empty");
        assert_eq!(read(2), "20");
    }
}

/// The capability server recalculates before every save, and recalculating asks the workbook for
/// each sheet again. That second ask used to re-parse the file and put the original rows back, so
/// an insert left a copy of the row behind — correct on its own, wrong through the application.
#[test]
fn a_recalculation_after_an_insert_does_not_bring_the_old_row_back() {
    let path = "/tmp/zavora-insert-recalc.xlsx";
    a_saved_sheet(path);

    let mut book = Workbook::open(path).unwrap();
    book.worksheet(0).unwrap().insert_rows(1, 1).unwrap();
    // Exactly what the server does between the edit and the save.
    let _ = book.recalculate();
    book.save(path).unwrap();

    let after = column_a(path);
    assert_eq!(
        after[..5],
        ["10", "", "20", "30", "40"],
        "recalculating reinstated the row that moved: {after:?}"
    );
}

/// The same fault, in its worse form: a cell the User deleted coming back.
#[test]
fn a_deleted_row_stays_deleted_through_a_recalculation() {
    let path = "/tmp/zavora-delete-recalc.xlsx";
    a_saved_sheet(path);

    let mut book = Workbook::open(path).unwrap();
    book.worksheet(0).unwrap().remove_rows(1, 1).unwrap();
    let _ = book.recalculate();
    book.save(path).unwrap();

    let after = column_a(path);
    assert_eq!(
        after[..3],
        ["10", "30", "40"],
        "the deleted row came back: {after:?}"
    );
}
