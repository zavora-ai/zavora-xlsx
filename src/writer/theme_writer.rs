/// Default Office theme XML (required for chart rendering in Excel).
pub fn write_theme() -> Vec<u8> {
    include_bytes!("theme1.xml").to_vec()
}
