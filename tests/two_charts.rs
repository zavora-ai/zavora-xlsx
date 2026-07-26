//! A second chart must not collide with the first.
//!
//! Adding a chart to a workbook that already had one failed the save with "Duplicate filename":
//! the new chart was written as `xl/charts/chart1.xml` while the original was still being carried
//! through under the same name. So a User could chart their data once, and the second chart threw
//! the whole save away.

use zavora_xlsx::{Chart, ChartType, Workbook};

fn a_workbook_with_one_chart(path: &str) {
    let _ = std::fs::remove_file(path);
    let mut book = Workbook::new();
    {
        let sheet = book.worksheet(0).unwrap();
        for row in 0..4u32 {
            sheet.write(row, 0, (row + 1) as f64 * 10.0).unwrap();
        }
        let mut chart = Chart::new(ChartType::Column);
        chart.add_series().set_values("Sheet1!$A$1:$A$4");
        sheet.insert_chart(4, 4, &chart).unwrap();
    }
    book.save(path).unwrap();
}

#[test]
fn a_second_chart_can_be_added_to_a_saved_workbook() {
    let path = "/tmp/zavora-two-charts.xlsx";
    a_workbook_with_one_chart(path);

    let mut reopened = Workbook::open(path).unwrap();
    {
        let sheet = reopened.worksheet(0).unwrap();
        let mut second = Chart::new(ChartType::Line);
        second.add_series().set_values("Sheet1!$A$1:$A$4");
        sheet.insert_chart(12, 4, &second).unwrap();
    }
    reopened
        .save(path)
        .expect("saving a workbook with a second chart should work");

    let check = Workbook::open_readonly(path).unwrap();
    let charts = check.worksheet_ref(0).unwrap().charts();
    assert_eq!(charts.len(), 2, "both charts should be in the file");
}
