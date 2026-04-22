//! Example: Add phonetic (furigana) annotations to Japanese text (Task 44).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Furigana")?;
    ws.set_column_width(0, 20.0)?;
    ws.set_column_width(1, 25.0)?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Kanji", &hdr)?;
    ws.write_with_format(0, 1, "With Furigana", &hdr)?;

    // 漢字 (kanji) with furigana かんじ
    ws.write(1, 0, "漢字")?;
    ws.write(1, 1, "漢字")?;
    ws.set_phonetic(
        1,
        1,
        vec![PhoneticRun::new(0, 1, "かん"), PhoneticRun::new(1, 2, "じ")],
    )?;

    // 東京 (Tokyo) with furigana とうきょう
    ws.write(2, 0, "東京")?;
    ws.write(2, 1, "東京")?;
    ws.set_phonetic(
        2,
        1,
        vec![
            PhoneticRun::new(0, 1, "とう"),
            PhoneticRun::new(1, 2, "きょう"),
        ],
    )?;

    // 日本語 (Japanese) with furigana にほんご
    ws.write(3, 0, "日本語")?;
    ws.write(3, 1, "日本語")?;
    ws.set_phonetic(
        3,
        1,
        vec![
            PhoneticRun::new(0, 1, "に"),
            PhoneticRun::new(1, 2, "ほん"),
            PhoneticRun::new(2, 3, "ご"),
        ],
    )?;

    wb.save("output/phonetic_text_example.xlsx")?;
    println!("✅ Phonetic text saved to output/phonetic_text_example.xlsx");
    Ok(())
}
