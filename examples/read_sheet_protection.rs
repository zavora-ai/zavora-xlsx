/// Example: protect a sheet, save, read back, and print protection state.
use zavora_xlsx::Workbook;

fn main() {
    let path = "target/example_read_sheet_protection.xlsx";

    // Create a workbook with a protected sheet
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Confidential Data").unwrap();
        ws.write(1, 0, 42.0).unwrap();
        ws.protect_with_password("secret");
        wb.save(path).unwrap();
        println!("Saved protected workbook to {path}");
    }

    // Read it back and inspect protection
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet_ref(0).unwrap();

        match ws.protection() {
            Some(prot) => {
                println!("\nSheet '{}' is protected:", ws.name());
                println!("  sheet:                {}", prot.sheet);
                println!("  objects:              {}", prot.objects);
                println!("  scenarios:            {}", prot.scenarios);
                println!("  format_cells:         {}", prot.format_cells);
                println!("  format_columns:       {}", prot.format_columns);
                println!("  format_rows:          {}", prot.format_rows);
                println!("  insert_columns:       {}", prot.insert_columns);
                println!("  insert_rows:          {}", prot.insert_rows);
                println!("  insert_hyperlinks:    {}", prot.insert_hyperlinks);
                println!("  delete_columns:       {}", prot.delete_columns);
                println!("  delete_rows:          {}", prot.delete_rows);
                println!("  select_locked_cells:  {}", prot.select_locked_cells);
                println!("  sort:                 {}", prot.sort);
                println!("  auto_filter:          {}", prot.auto_filter);
                println!("  pivot_tables:         {}", prot.pivot_tables);
                println!("  select_unlocked_cells:{}", prot.select_unlocked_cells);
                match prot.password_hash() {
                    Some(hash) => println!("  password_hash:        {hash}"),
                    None => println!("  password_hash:        (none)"),
                }
            }
            None => {
                println!("Sheet '{}' is not protected.", ws.name());
            }
        }
    }

    std::fs::remove_file(path).ok();
}
