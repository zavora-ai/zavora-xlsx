//! A chart has to read back where the file puts it.
//!
//! Every chart came back anchored at A1, because the drawing that says where a chart sits was
//! parsed only for which chart it referenced. An application drawing a chart at its stated
//! position would have laid every one of them over the top-left of the User's data.

use zavora_xlsx::{Chart, ChartType, Workbook};

#[test]
fn a_chart_reads_back_at_the_cell_it_was_placed_at() {
    let path = "/tmp/zavora-chart-position.xlsx";
    let _ = std::fs::remove_file(path);

    let mut book = Workbook::new();
    {
        let sheet = book.worksheet(0).unwrap();
        for row in 0..4u32 {
            sheet.write(row, 0, (row + 1) as f64 * 10.0).unwrap();
        }
        let mut chart = Chart::new(ChartType::Column);
        chart.add_series().set_values("Sheet1!$A$1:$A$4");
        // Row 4, column 4 — "E5" as a person would say it.
        sheet.insert_chart(4, 4, &chart).unwrap();
    }
    book.save(path).unwrap();

    let reopened = Workbook::open_readonly(path).unwrap();
    let sheet = reopened.worksheet_ref(0).unwrap();
    let charts = sheet.charts();
    assert_eq!(charts.len(), 1, "the chart should be read back");
    assert_eq!(
        charts[0].anchor(),
        (4, 4),
        "the chart came back somewhere other than where it was put"
    );
}
