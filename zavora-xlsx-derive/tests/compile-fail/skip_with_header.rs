use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow)]
struct Foo {
    #[excel(skip, header = "X")]
    name: String,
}

fn main() {}
