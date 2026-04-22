//! Example: Save a workbook as a macro-enabled XLSM file.
//!
//! This demonstrates how to create a workbook and save it with VBA project data.
//! In a real scenario, the VBA binary would come from an existing .xlsm file.

use zavora_xlsx::Workbook;

fn main() -> zavora_xlsx::Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "Macro-Enabled Workbook")?;
    ws.write(1, 0, "This workbook contains VBA macros")?;
    ws.write(2, 0, 42.0)?;

    // In practice, you'd read vbaProject.bin from an existing .xlsm file.
    // Here we use a minimal placeholder to demonstrate the API.
    let fake_vba_project = b"FAKE_VBA_PROJECT_BIN_DATA";

    wb.save_as_xlsm("output/macro_workbook.xlsm", fake_vba_project)?;
    println!("Saved macro-enabled workbook to output/macro_workbook.xlsm");

    Ok(())
}
