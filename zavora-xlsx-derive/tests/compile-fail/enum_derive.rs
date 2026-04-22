use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow)]
enum Foo {
    A,
    B,
}

fn main() {}
