use zavora_xlsx::*;

/// Helper: create a workbook, save it, reopen readonly, and return the workbook.
fn roundtrip_workbook(setup: impl FnOnce(&mut Worksheet)) -> Workbook {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dv_roundtrip.xlsx");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        setup(ws);
        wb.save(&path).unwrap();
    }

    Workbook::open_readonly(&path).unwrap()
}

#[test]
fn test_list_validation_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        let mut dv = DataValidation::new(ValidationRule::List(vec![
            "Apple".into(),
            "Banana".into(),
            "Cherry".into(),
        ]));
        dv.set_input_message("Choose", "Select from list");
        dv.set_error_message(ErrorStyle::Stop, "Invalid", "Not in list");
        ws.add_data_validation(0, 0, 9, 0, &dv).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let dvs = ws.validations();

    assert_eq!(dvs.len(), 1, "expected 1 validation rule");
    let dv = &dvs[0];

    // Verify range
    assert_eq!(dv.first_row(), 0);
    assert_eq!(dv.first_col(), 0);
    assert_eq!(dv.last_row(), 9);
    assert_eq!(dv.last_col(), 0);

    // Verify rule
    match dv.rule() {
        ValidationRule::List(items) => {
            assert_eq!(items, &["Apple", "Banana", "Cherry"]);
        }
        other => panic!("expected List rule, got {:?}", other),
    }

    // Verify messages
    assert_eq!(dv.input_title(), Some("Choose"));
    assert_eq!(dv.input_message(), Some("Select from list"));
    assert_eq!(dv.error_style(), ErrorStyle::Stop);
    assert_eq!(dv.error_title(), Some("Invalid"));
    assert_eq!(dv.error_message(), Some("Not in list"));
}

#[test]
fn test_whole_number_between_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        let dv = DataValidation::new(ValidationRule::WholeNumber {
            min: Some(1),
            max: Some(100),
        });
        ws.add_data_validation(0, 1, 9, 1, &dv).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let dvs = ws.validations();

    assert_eq!(dvs.len(), 1);
    let dv = &dvs[0];

    assert_eq!(dv.first_row(), 0);
    assert_eq!(dv.first_col(), 1);
    assert_eq!(dv.last_row(), 9);
    assert_eq!(dv.last_col(), 1);

    match dv.rule() {
        ValidationRule::WholeNumber { min, max } => {
            assert_eq!(*min, Some(1));
            assert_eq!(*max, Some(100));
        }
        other => panic!("expected WholeNumber rule, got {:?}", other),
    }
}

#[test]
fn test_decimal_validation_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        let dv = DataValidation::new(ValidationRule::Decimal {
            min: Some(0.0),
            max: Some(99.99),
        });
        ws.add_data_validation(0, 2, 4, 2, &dv).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let dvs = ws.validations();

    assert_eq!(dvs.len(), 1);
    let dv = &dvs[0];

    match dv.rule() {
        ValidationRule::Decimal { min, max } => {
            assert_eq!(*min, Some(0.0));
            assert_eq!(*max, Some(99.99));
        }
        other => panic!("expected Decimal rule, got {:?}", other),
    }
}

#[test]
fn test_text_length_validation_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        let dv = DataValidation::new(ValidationRule::TextLength {
            min: Some(1),
            max: Some(255),
        });
        ws.add_data_validation(0, 3, 9, 3, &dv).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let dvs = ws.validations();

    assert_eq!(dvs.len(), 1);
    let dv = &dvs[0];

    match dv.rule() {
        ValidationRule::TextLength { min, max } => {
            assert_eq!(*min, Some(1));
            assert_eq!(*max, Some(255));
        }
        other => panic!("expected TextLength rule, got {:?}", other),
    }
}

#[test]
fn test_custom_formula_validation_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        let dv = DataValidation::new(ValidationRule::Custom("AND(A1>0,A1<100)".into()));
        ws.add_data_validation(0, 4, 9, 4, &dv).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let dvs = ws.validations();

    assert_eq!(dvs.len(), 1);
    let dv = &dvs[0];

    match dv.rule() {
        ValidationRule::Custom(formula) => {
            assert_eq!(formula, "AND(A1>0,A1<100)");
        }
        other => panic!("expected Custom rule, got {:?}", other),
    }
}

#[test]
fn test_multiple_validations_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        // List validation on A1:A10
        let mut dv1 = DataValidation::new(ValidationRule::List(vec!["Yes".into(), "No".into()]));
        dv1.set_input_message("Answer", "Pick yes or no");
        ws.add_data_validation(0, 0, 9, 0, &dv1).unwrap();

        // Whole number on B1:B10
        let dv2 = DataValidation::new(ValidationRule::WholeNumber {
            min: Some(0),
            max: Some(1000),
        });
        ws.add_data_validation(0, 1, 9, 1, &dv2).unwrap();

        // Custom formula on C1:C10
        let dv3 = DataValidation::new(ValidationRule::Custom("LEN(C1)>0".into()));
        ws.add_data_validation(0, 2, 9, 2, &dv3).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let dvs = ws.validations();

    assert_eq!(dvs.len(), 3, "expected 3 validation rules");

    // Verify first rule is List
    match dvs[0].rule() {
        ValidationRule::List(items) => {
            assert_eq!(items, &["Yes", "No"]);
        }
        other => panic!("expected List rule, got {:?}", other),
    }
    assert_eq!(dvs[0].input_title(), Some("Answer"));
    assert_eq!(dvs[0].input_message(), Some("Pick yes or no"));

    // Verify second rule is WholeNumber
    match dvs[1].rule() {
        ValidationRule::WholeNumber { min, max } => {
            assert_eq!(*min, Some(0));
            assert_eq!(*max, Some(1000));
        }
        other => panic!("expected WholeNumber rule, got {:?}", other),
    }

    // Verify third rule is Custom
    match dvs[2].rule() {
        ValidationRule::Custom(formula) => {
            assert_eq!(formula, "LEN(C1)>0");
        }
        other => panic!("expected Custom rule, got {:?}", other),
    }
}

#[test]
fn test_error_style_warning_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        let mut dv = DataValidation::new(ValidationRule::WholeNumber {
            min: Some(1),
            max: Some(10),
        });
        dv.set_error_message(ErrorStyle::Warning, "Warning", "Value out of range");
        ws.add_data_validation(0, 0, 4, 0, &dv).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let dvs = ws.validations();

    assert_eq!(dvs.len(), 1);
    assert_eq!(dvs[0].error_style(), ErrorStyle::Warning);
    assert_eq!(dvs[0].error_title(), Some("Warning"));
    assert_eq!(dvs[0].error_message(), Some("Value out of range"));
}

#[test]
fn test_edit_mode_validation_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dv_edit_roundtrip.xlsx");

    // Write a file with validations
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        let dv = DataValidation::new(ValidationRule::List(vec![
            "Red".into(),
            "Green".into(),
            "Blue".into(),
        ]));
        ws.add_data_validation(0, 0, 9, 0, &dv).unwrap();
        wb.save(&path).unwrap();
    }

    // Open in edit mode and verify validations are parsed
    {
        let mut wb = Workbook::open(&path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        let dvs = ws.validations();

        assert_eq!(dvs.len(), 1);
        match dvs[0].rule() {
            ValidationRule::List(items) => {
                assert_eq!(items, &["Red", "Green", "Blue"]);
            }
            other => panic!("expected List rule, got {:?}", other),
        }
    }
}
