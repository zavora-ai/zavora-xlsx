//! Example: tokenize sample formulas and print token sequences.
//!
//! Run with: cargo run --example formula_tokenizer

use zavora_xlsx::formula_engine::tokenize;

fn main() {
    let formulas = [
        "A1+B2*C3",
        "SUM(A1:A10)",
        "IF(A1>0,A1*0.1,0)",
        "VLOOKUP(A1,Sheet2!B1:D100,3,FALSE)",
        "'My Sheet'!$A$1:$B$10",
        "Table1[Revenue]*1.08",
        r#"CONCATENATE("Hello"," ","World")"#,
        "{1,2,3;4,5,6}",
        "R[-1]C[2]+R1C1",
        "#VALUE!+#REF!",
    ];

    for formula in &formulas {
        println!("Formula: {formula}");
        match tokenize(formula) {
            Ok(tokens) => {
                for (i, token) in tokens.iter().enumerate() {
                    println!("  [{i}] {token:?}");
                }
            }
            Err(e) => {
                println!("  ERROR: {e}");
            }
        }
        println!();
    }
}
