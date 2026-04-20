//! Example: evaluate formulas in a workbook context and print results.
//!
//! Demonstrates the formula engine's core functions: SUM, AVERAGE, COUNT,
//! COUNTA, MIN, MAX, IF, VLOOKUP, INDEX, and MATCH.
//!
//! Run with: cargo run --example formula_evaluation

use zavora_xlsx::formula_engine::evaluator::{evaluate, SimpleContext, Value};
use zavora_xlsx::formula_engine::{parse, tokenize};

fn eval(formula: &str, ctx: &SimpleContext) -> Value {
    let tokens = tokenize(formula).unwrap();
    let ast = parse(&tokens).unwrap();
    evaluate(&ast, ctx, 0)
}

fn main() {
    // Build a small "spreadsheet" context:
    //
    //       A          B          C
    // 1     1          Apple      0.50
    // 2     2          Banana     0.30
    // 3     3          Cherry     0.75
    // 4     4          Date       1.00
    // 5     5          (empty)    (empty)
    //
    let mut ctx = SimpleContext::new(1);
    let data = [
        (1.0, "Apple", 0.50),
        (2.0, "Banana", 0.30),
        (3.0, "Cherry", 0.75),
        (4.0, "Date", 1.00),
    ];
    for (r, (id, name, price)) in data.iter().enumerate() {
        ctx.set(0, r as u32, 0, Value::Number(*id));
        ctx.set(0, r as u32, 1, Value::String(name.to_string()));
        ctx.set(0, r as u32, 2, Value::Number(*price));
    }
    ctx.set(0, 4, 0, Value::Number(5.0));

    // Aggregate functions
    let formulas = [
        ("SUM(A1:A5)", "Sum of 1..5"),
        ("AVERAGE(A1:A5)", "Average of 1..5"),
        ("COUNT(A1:A5)", "Count of numeric values"),
        ("COUNTA(A1:C4)", "Count of non-empty cells"),
        ("MIN(A1:A5)", "Minimum value"),
        ("MAX(A1:A5)", "Maximum value"),
    ];

    println!("=== Aggregate Functions ===");
    for (formula, desc) in &formulas {
        let result = eval(formula, &ctx);
        println!("  {desc}");
        println!("    {formula} = {result:?}");
    }
    println!();

    // IF function
    println!("=== IF Function ===");
    let if_formulas = [
        ("IF(A1>3,\"big\",\"small\")", "A1=1, so false branch"),
        ("IF(A4>3,\"big\",\"small\")", "A4=4, so true branch"),
        ("IF(TRUE,42)", "Two-arg IF, true"),
        ("IF(FALSE,42)", "Two-arg IF, false → FALSE"),
    ];
    for (formula, desc) in &if_formulas {
        let result = eval(formula, &ctx);
        println!("  {desc}");
        println!("    {formula} = {result:?}");
    }
    println!();

    // VLOOKUP
    println!("=== VLOOKUP ===");
    let vlookup_formulas = [
        ("VLOOKUP(2,A1:C4,2,FALSE)", "Exact match: lookup 2 → Banana"),
        ("VLOOKUP(2,A1:C4,3,FALSE)", "Exact match: lookup 2 → price 0.30"),
        ("VLOOKUP(2.5,A1:C4,2,TRUE)", "Approx match: 2.5 → Banana (largest ≤ 2.5)"),
        ("VLOOKUP(5,A1:C4,2,FALSE)", "Not found → #N/A"),
    ];
    for (formula, desc) in &vlookup_formulas {
        let result = eval(formula, &ctx);
        println!("  {desc}");
        println!("    {formula} = {result:?}");
    }
    println!();

    // INDEX and MATCH
    println!("=== INDEX & MATCH ===");
    let index_match_formulas = [
        ("INDEX(A1:C4,2,2)", "INDEX row 2, col 2 → Banana"),
        ("INDEX(A1:A5,3)", "INDEX row 3 of column → 3"),
        ("MATCH(3,A1:A5,0)", "Exact MATCH for 3 → position 3"),
        ("MATCH(\"Cherry\",B1:B4,0)", "Exact MATCH for Cherry → position 3"),
        ("INDEX(C1:C4,MATCH(\"Cherry\",B1:B4,0))", "INDEX+MATCH → price of Cherry"),
    ];
    for (formula, desc) in &index_match_formulas {
        let result = eval(formula, &ctx);
        println!("  {desc}");
        println!("    {formula} = {result:?}");
    }
    println!();

    // Nested / combined
    println!("=== Combined Formulas ===");
    let combined = [
        ("SUM(A1:A5)*2", "Sum * 2"),
        ("IF(AVERAGE(C1:C4)>0.5,\"expensive\",\"cheap\")", "Conditional on average price"),
        ("SUM(A1:A3)+MAX(A4:A5)", "Sum of first 3 + max of last 2"),
    ];
    for (formula, desc) in &combined {
        let result = eval(formula, &ctx);
        println!("  {desc}");
        println!("    {formula} = {result:?}");
    }
}
