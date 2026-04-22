use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use zavora_xlsx::{DocProperties, Workbook};

/// Create a small xlsx test fixture with known data and properties.
fn create_test_fixture(path: &std::path::Path) {
    let mut wb = Workbook::new();
    wb.set_properties(DocProperties::new().title("Test").author("Tester"));
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Name").unwrap();
    ws.write(0, 1, "Value").unwrap();
    ws.write(1, 0, "Alice").unwrap();
    ws.write(1, 1, 42.0).unwrap();
    ws.write(2, 0, "Bob").unwrap();
    ws.write(2, 1, 100.0).unwrap();
    wb.save(path).unwrap();
}

#[test]
fn test_help_exits_0() {
    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inspect"));
}

#[test]
fn test_version_exits_0() {
    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn test_no_args_exits_nonzero() {
    Command::cargo_bin("zavora-xlsx").unwrap().assert().code(2);
}

#[test]
fn test_invalid_subcommand_exits_2() {
    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("badcmd")
        .assert()
        .code(2);
}

#[test]
fn test_inspect_valid_file() {
    let dir = TempDir::new().unwrap();
    let fixture = dir.path().join("test.xlsx");
    create_test_fixture(&fixture);

    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("inspect")
        .arg(fixture.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Sheet1"))
        .stdout(predicate::str::contains("Sheets:"))
        .stdout(predicate::str::contains("Title: Test"))
        .stdout(predicate::str::contains("Author: Tester"));
}

#[test]
fn test_inspect_bad_file() {
    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("inspect")
        .arg("nonexistent.xlsx")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_export_default() {
    let dir = TempDir::new().unwrap();
    let fixture = dir.path().join("test.xlsx");
    create_test_fixture(&fixture);

    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("export")
        .arg(fixture.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Name"))
        .stdout(predicate::str::contains("Alice"));
}

#[test]
fn test_export_with_output() {
    let dir = TempDir::new().unwrap();
    let fixture = dir.path().join("test.xlsx");
    create_test_fixture(&fixture);

    let output_file = dir.path().join("output.csv");

    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("export")
        .arg(fixture.to_str().unwrap())
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .assert()
        .success();

    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("Name"));
    assert!(content.contains("Alice"));
}

#[test]
fn test_export_bad_sheet() {
    let dir = TempDir::new().unwrap();
    let fixture = dir.path().join("test.xlsx");
    create_test_fixture(&fixture);

    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("export")
        .arg(fixture.to_str().unwrap())
        .arg("--sheet")
        .arg("NoSuchSheet")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_export_tsv() {
    let dir = TempDir::new().unwrap();
    let fixture = dir.path().join("test.xlsx");
    create_test_fixture(&fixture);

    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("export")
        .arg(fixture.to_str().unwrap())
        .arg("--tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("\t"));
}

#[test]
fn test_convert_xlsx_to_csv() {
    let dir = TempDir::new().unwrap();
    let fixture = dir.path().join("test.xlsx");
    create_test_fixture(&fixture);

    let out_dir = dir.path().join("csv_out");

    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("convert")
        .arg(fixture.to_str().unwrap())
        .arg("--format")
        .arg("csv")
        .arg("--output")
        .arg(out_dir.to_str().unwrap())
        .assert()
        .success();

    let csv_file = out_dir.join("Sheet1.csv");
    assert!(
        csv_file.exists(),
        "Sheet1.csv should exist in output directory"
    );
}

#[test]
fn test_convert_csv_to_xlsx() {
    let dir = TempDir::new().unwrap();
    let csv_path = dir.path().join("data.csv");
    std::fs::write(&csv_path, "Name,Value\nAlice,42\nBob,100\n").unwrap();

    let output_path = dir.path().join("data.xlsx");

    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("convert")
        .arg(csv_path.to_str().unwrap())
        .arg("--format")
        .arg("xlsx")
        .arg("--output")
        .arg(output_path.to_str().unwrap())
        .assert()
        .success();

    assert!(output_path.exists(), "Output xlsx file should exist");
}

#[test]
fn test_convert_bad_file() {
    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("convert")
        .arg("nonexistent.xlsx")
        .arg("--format")
        .arg("csv")
        .assert()
        .code(1);
}

#[test]
fn test_convert_incompatible_format() {
    let dir = TempDir::new().unwrap();
    let csv_path = dir.path().join("data.csv");
    std::fs::write(&csv_path, "a,b\n1,2\n").unwrap();

    Command::cargo_bin("zavora-xlsx")
        .unwrap()
        .arg("convert")
        .arg(csv_path.to_str().unwrap())
        .arg("--format")
        .arg("xlsm")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}
