use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow)]
struct Foo(u32, String);

fn main() {}
