//! Formatting has to survive an edit.
//!
//! Cells remember the style index they were parsed with, but the table those indices point into
//! was rebuilt from an empty registry on save. So a workbook that was opened, changed in one
//! cell and saved came back with its formatting gone — bold headings plain, shaded totals
//! white — while the save reported success. Editing a formatted file destroyed the formatting,
//! which is the one thing a spreadsheet must never do quietly.

use std::io::Read;

use zavora_xlsx::{Format, Workbook};

fn styles_of(path: &str) -> String {
    let file = std::fs::File::open(path).expect("the file should be there");
    let mut zip = zip::ZipArchive::new(file).expect("it should be a workbook");
    let mut out = String::new();
    zip.by_name("xl/styles.xml")
        .expect("a saved workbook has a style table")
        .read_to_string(&mut out)
        .expect("and it should be readable");
    out
}

fn a_formatted_workbook(path: &str) {
    let _ = std::fs::remove_file(path);
    let mut book = Workbook::new();
    {
        let sheet = book.add_worksheet();
        sheet.write(0, 0, "Heading").unwrap();
        sheet
            .set_cell_format(0, 0, &Format::new().bold().background_color("#FFF3C4"))
            .unwrap();
        sheet.write(1, 0, 1234.5).unwrap();
        sheet
            .set_cell_format(1, 0, &Format::new().num_format("#,##0.00"))
            .unwrap();
    }
    book.save(path).unwrap();
}

#[test]
fn an_edit_elsewhere_keeps_the_formatting() {
    let path = "/tmp/zavora-parity-elsewhere.xlsx";
    a_formatted_workbook(path);

    let before = styles_of(path);
    assert!(
        before.to_uppercase().contains("FFF3C4"),
        "the fixture itself should be shaded"
    );

    // Exactly what a hand edit does: open, change a cell, save.
    let mut reopened = Workbook::open(path).unwrap();
    reopened
        .worksheet(0)
        .unwrap()
        .write(5, 3, "elsewhere")
        .unwrap();
    reopened.save(path).unwrap();

    let after = styles_of(path);
    assert!(
        after.to_uppercase().contains("FFF3C4"),
        "the shading was lost by an edit to a different cell"
    );
    assert!(
        after.contains("#,##0.00"),
        "the number format was lost by an edit to a different cell"
    );
}

#[test]
fn the_formatted_cell_itself_keeps_its_look_when_its_value_changes() {
    let path = "/tmp/zavora-parity-same-cell.xlsx";
    a_formatted_workbook(path);

    let mut reopened = Workbook::open(path).unwrap();
    // Overwriting the number in a formatted cell must change the number, not the formatting.
    reopened
        .worksheet(0)
        .unwrap()
        .write(1, 0, 99.0)
        .unwrap();
    reopened.save(path).unwrap();

    let after = styles_of(path);
    assert!(
        after.contains("#,##0.00"),
        "changing the value threw away the cell's number format"
    );

    let mut check = Workbook::open(path).unwrap();
    let value = check.worksheet(0).unwrap().read_cell(1, 0);
    assert!(
        format!("{value:?}").contains("99"),
        "the new value should be there: {value:?}"
    );
}

#[test]
fn a_workbook_saved_twice_does_not_grow_a_second_copy_of_every_style() {
    let path = "/tmp/zavora-parity-growth.xlsx";
    a_formatted_workbook(path);

    let count = |styles: &str| styles.matches("<xf ").count();
    let first = count(&styles_of(path));

    for _ in 0..3 {
        let mut reopened = Workbook::open(path).unwrap();
        reopened
            .worksheet(0)
            .unwrap()
            .write(9, 9, "again")
            .unwrap();
        reopened.save(path).unwrap();
    }

    let last = count(&styles_of(path));
    assert_eq!(
        first, last,
        "opening and saving three times turned {first} styles into {last}"
    );
}
