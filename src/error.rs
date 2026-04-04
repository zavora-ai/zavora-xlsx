use std::fmt;

/// Central error type for zavora-xlsx.
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Xml(quick_xml::Error),
    XmlAttr(quick_xml::events::attributes::AttrError),
    InvalidCellRef(String),
    InvalidRange(String),
    SheetNotFound(String),
    ReadOnly,
    InvalidData(String),
    Password,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::Zip(e) => write!(f, "ZIP error: {e}"),
            Error::Xml(e) => write!(f, "XML error: {e}"),
            Error::XmlAttr(e) => write!(f, "XML attribute error: {e}"),
            Error::InvalidCellRef(s) => write!(f, "Invalid cell reference: {s}"),
            Error::InvalidRange(s) => write!(f, "Invalid range: {s}"),
            Error::SheetNotFound(s) => write!(f, "Sheet not found: {s}"),
            Error::ReadOnly => write!(f, "Workbook is read-only"),
            Error::InvalidData(s) => write!(f, "Invalid data: {s}"),
            Error::Password => write!(f, "File is password-protected"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Error::Io(e) }
}
impl From<zip::result::ZipError> for Error {
    fn from(e: zip::result::ZipError) -> Self { Error::Zip(e) }
}
impl From<quick_xml::Error> for Error {
    fn from(e: quick_xml::Error) -> Self { Error::Xml(e) }
}
impl From<quick_xml::events::attributes::AttrError> for Error {
    fn from(e: quick_xml::events::attributes::AttrError) -> Self { Error::XmlAttr(e) }
}

pub type Result<T> = std::result::Result<T, Error>;
