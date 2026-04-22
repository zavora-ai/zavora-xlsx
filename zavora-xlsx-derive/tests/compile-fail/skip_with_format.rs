use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow)]
struct Foo {
    #[excel(skip, format = "0.00")]
    val: f64,
}

fn main() {}
