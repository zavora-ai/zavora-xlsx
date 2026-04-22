use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow)]
struct Foo {
    #[excel(unknown_key = "x")]
    name: String,
}

fn main() {}
