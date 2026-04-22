//! Example: parse formulas into AST and print tree structure.
//!
//! Run with: cargo run --example formula_parser

use zavora_xlsx::formula_engine::{parse, print_formula, tokenize};

fn main() {
    let formulas = [
        "1+2*3",
        "(1+2)*3",
        "SUM(A1:A10)",
        "IF(A1>0,A1*0.1,0)",
        "VLOOKUP(A1,Sheet1!B1:D100,3,FALSE)",
        "{1,2;3,4}",
        "-A1+B2^2",
    ];

    for formula in &formulas {
        println!("Formula: {formula}");
        let tokens = tokenize(formula).unwrap();
        let ast = parse(&tokens).unwrap();
        println!("  AST: {ast:#?}");
        let printed = print_formula(&ast);
        println!("  Printed: {printed}");
        println!();
    }
}
