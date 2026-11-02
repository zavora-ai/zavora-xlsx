//! Parts the crate does not model survive an open-edit-save round trip, whatever their name.

use std::io::{Cursor, Read, Write};

use zavora_xlsx::*;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const CUSTOM_XML: &str = "customXml/item1.xml";
const CUSTOM_XML_DATA: &[u8] = b"<root xmlns=\"urn:example\"><v>1</v></root>";

// `ZipWriter` writes UTF-8 names only, so the test writes a placeholder name and then
// overwrites its bytes, in the local header and in the central directory, with a name
// of the same length that is not valid UTF-8.
const PLACEHOLDER: &[u8] = b"custom/placeholder.bin";
const NON_UTF8_NAME: &[u8] = b"custom/\x80\x81\x82ceholder.bin";
const NON_UTF8_DATA: &[u8] = b"opaque part";

/// A workbook written by this crate, re-zipped with two extra parts: a custom XML part
/// and a part whose name is not valid UTF-8.
fn workbook_with_custom_parts() -> Vec<u8> {
    let mut wb = Workbook::new();
    wb.worksheet(0).unwrap().write(0, 0, "Original").unwrap();
    let saved = wb.save_to_buffer().unwrap();

    let mut source = ZipArchive::new(Cursor::new(saved)).unwrap();
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..source.len() {
        writer.raw_copy_file(source.by_index(i).unwrap()).unwrap();
    }
    let options = SimpleFileOptions::default();
    writer.start_file(CUSTOM_XML, options).unwrap();
    writer.write_all(CUSTOM_XML_DATA).unwrap();
    writer
        .start_file(std::str::from_utf8(PLACEHOLDER).unwrap(), options)
        .unwrap();
    writer.write_all(NON_UTF8_DATA).unwrap();
    let mut data = writer.finish().unwrap().into_inner();

    assert_eq!(PLACEHOLDER.len(), NON_UTF8_NAME.len());
    let mut replaced = 0;
    let mut pos = 0;
    while let Some(offset) = data[pos..]
        .windows(PLACEHOLDER.len())
        .position(|w| w == PLACEHOLDER)
    {
        let start = pos + offset;
        data[start..start + NON_UTF8_NAME.len()].copy_from_slice(NON_UTF8_NAME);
        pos = start + NON_UTF8_NAME.len();
        replaced += 1;
    }
    assert_eq!(replaced, 2, "local header and central directory");

    let archive = ZipArchive::new(Cursor::new(&data)).unwrap();
    assert!(archive.file_names_raw().any(|raw| raw == NON_UTF8_NAME));
    data
}

/// The name zip gives the non-UTF-8 entry when it decodes it.
fn decoded_non_utf8_name(data: &[u8]) -> String {
    let archive = ZipArchive::new(Cursor::new(data)).unwrap();
    let index = archive
        .file_names_raw()
        .position(|raw| raw == NON_UTF8_NAME)
        .unwrap();
    archive.name_for_index(index).unwrap().unwrap().into_owned()
}

fn read_part(data: &[u8], name: &str) -> Vec<u8> {
    let mut archive = ZipArchive::new(Cursor::new(data)).unwrap();
    let mut entry = archive.by_name(name).unwrap();
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).unwrap();
    buf
}

#[test]
fn custom_parts_survive_open_save_reopen() {
    let original = workbook_with_custom_parts();
    let non_utf8 = decoded_non_utf8_name(&original);

    let mut wb = Workbook::open_from_buffer(&original).unwrap();
    let names: Vec<&str> = wb
        .passthrough_entries()
        .iter()
        .map(|(n, _)| n.as_str())
        .collect();
    assert!(names.contains(&CUSTOM_XML), "{names:?}");
    assert!(names.contains(&non_utf8.as_str()), "{names:?}");

    wb.worksheet(0).unwrap().write(0, 0, "Modified").unwrap();
    let saved = wb.save_to_buffer().unwrap();

    assert_eq!(read_part(&saved, CUSTOM_XML), CUSTOM_XML_DATA);
    assert_eq!(read_part(&saved, &non_utf8), NON_UTF8_DATA);

    let mut reopened = Workbook::open_from_buffer(&saved).unwrap();
    assert_eq!(
        reopened.worksheet(0).unwrap().read_cell(0, 0),
        CellValue::String("Modified".into())
    );
    assert!(
        reopened
            .passthrough_entries()
            .iter()
            .any(|(n, d)| n == CUSTOM_XML && d == CUSTOM_XML_DATA)
    );
}

#[test]
fn non_utf8_entry_name_does_not_stop_a_read() {
    let original = workbook_with_custom_parts();
    let mut wb = Workbook::open_readonly_from_buffer(&original).unwrap();
    assert_eq!(
        wb.worksheet(0).unwrap().read_cell(0, 0),
        CellValue::String("Original".into())
    );
}
