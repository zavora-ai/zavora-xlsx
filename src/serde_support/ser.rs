//! Serialize structs to worksheet rows.
//!
//! Uses a custom serde `Serializer` to extract field names (for headers)
//! and field values (for cell data) from any `Serialize` type.

use serde::ser::{self, Serialize};

/// A collected cell value from serialization.
#[derive(Debug, Clone)]
pub(crate) enum SerValue {
    String(String),
    Number(f64),
    Bool(bool),
    None,
}

/// Serializer that collects field names and values from a struct.
pub(crate) struct RowSerializer {
    pub(crate) fields: Vec<String>,
    pub(crate) values: Vec<SerValue>,
    /// When true, we are collecting field names (first pass).
    collecting_names: bool,
}

impl RowSerializer {
    pub(crate) fn new() -> Self {
        Self {
            fields: Vec::new(),
            values: Vec::new(),
            collecting_names: true,
        }
    }
}

/// Error type for serde serialization within zavora-xlsx.
#[derive(Debug)]
pub struct SerError(pub(crate) String);

impl std::fmt::Display for SerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Serde serialization error: {}", self.0)
    }
}

impl std::error::Error for SerError {}

impl ser::Error for SerError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        SerError(msg.to_string())
    }
}

/// A struct serializer that captures field names and values.
pub(crate) struct StructSerializer<'a> {
    ser: &'a mut RowSerializer,
}

impl<'a> ser::SerializeStruct for StructSerializer<'a> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        if self.ser.collecting_names {
            self.ser.fields.push(key.to_string());
        }
        let mut val_ser = ValueSerializer;
        let v = value.serialize(&mut val_ser)?;
        self.ser.values.push(v);
        Ok(())
    }

    fn end(self) -> Result<(), Self::Error> {
        self.ser.collecting_names = false;
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut RowSerializer {
    type Ok = ();
    type Error = SerError;
    type SerializeSeq = ser::Impossible<(), SerError>;
    type SerializeTuple = ser::Impossible<(), SerError>;
    type SerializeTupleStruct = ser::Impossible<(), SerError>;
    type SerializeTupleVariant = ser::Impossible<(), SerError>;
    type SerializeMap = ser::Impossible<(), SerError>;
    type SerializeStructVariant = ser::Impossible<(), SerError>;
    type SerializeStruct = StructSerializer<'a>;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(StructSerializer { ser: self })
    }

    fn serialize_bool(self, _v: bool) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_i8(self, _v: i8) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_i16(self, _v: i16) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_i32(self, _v: i32) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_i64(self, _v: i64) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_u8(self, _v: u8) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_u16(self, _v: u16) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_u32(self, _v: u32) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_u64(self, _v: u64) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_f32(self, _v: f32) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_f64(self, _v: f64) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_char(self, _v: char) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_str(self, _v: &str) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_none(self) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _v: &T) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_unit(self) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _variant: &'static str,
    ) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _v: &T,
    ) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _idx: u32,
        _variant: &'static str,
        _v: &T,
    ) -> Result<(), SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, SerError> {
        Err(SerError("expected struct".into()))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, SerError> {
        Err(SerError("expected struct".into()))
    }
}

/// Value serializer — extracts a single field value.
struct ValueSerializer;

impl<'a> ser::Serializer for &'a mut ValueSerializer {
    type Ok = SerValue;
    type Error = SerError;
    type SerializeSeq = ser::Impossible<SerValue, SerError>;
    type SerializeTuple = ser::Impossible<SerValue, SerError>;
    type SerializeTupleStruct = ser::Impossible<SerValue, SerError>;
    type SerializeTupleVariant = ser::Impossible<SerValue, SerError>;
    type SerializeMap = ser::Impossible<SerValue, SerError>;
    type SerializeStruct = ser::Impossible<SerValue, SerError>;
    type SerializeStructVariant = ser::Impossible<SerValue, SerError>;

    fn serialize_bool(self, v: bool) -> Result<SerValue, SerError> {
        Ok(SerValue::Bool(v))
    }
    fn serialize_i8(self, v: i8) -> Result<SerValue, SerError> {
        Ok(SerValue::Number(v as f64))
    }
    fn serialize_i16(self, v: i16) -> Result<SerValue, SerError> {
        Ok(SerValue::Number(v as f64))
    }
    fn serialize_i32(self, v: i32) -> Result<SerValue, SerError> {
        Ok(SerValue::Number(v as f64))
    }
    fn serialize_i64(self, v: i64) -> Result<SerValue, SerError> {
        Ok(SerValue::Number(v as f64))
    }
    fn serialize_u8(self, v: u8) -> Result<SerValue, SerError> {
        Ok(SerValue::Number(v as f64))
    }
    fn serialize_u16(self, v: u16) -> Result<SerValue, SerError> {
        Ok(SerValue::Number(v as f64))
    }
    fn serialize_u32(self, v: u32) -> Result<SerValue, SerError> {
        Ok(SerValue::Number(v as f64))
    }
    fn serialize_u64(self, v: u64) -> Result<SerValue, SerError> {
        Ok(SerValue::Number(v as f64))
    }
    fn serialize_f32(self, v: f32) -> Result<SerValue, SerError> {
        Ok(SerValue::Number(v as f64))
    }
    fn serialize_f64(self, v: f64) -> Result<SerValue, SerError> {
        Ok(SerValue::Number(v))
    }
    fn serialize_char(self, v: char) -> Result<SerValue, SerError> {
        Ok(SerValue::String(v.to_string()))
    }
    fn serialize_str(self, v: &str) -> Result<SerValue, SerError> {
        Ok(SerValue::String(v.to_string()))
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<SerValue, SerError> {
        Err(SerError("bytes not supported".into()))
    }
    fn serialize_none(self) -> Result<SerValue, SerError> {
        Ok(SerValue::None)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<SerValue, SerError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<SerValue, SerError> {
        Ok(SerValue::None)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<SerValue, SerError> {
        Ok(SerValue::None)
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
    ) -> Result<SerValue, SerError> {
        Ok(SerValue::String(variant.to_string()))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<SerValue, SerError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _idx: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<SerValue, SerError> {
        value.serialize(self)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, SerError> {
        Err(SerError("sequences not supported as cell values".into()))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, SerError> {
        Err(SerError("tuples not supported as cell values".into()))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerError> {
        Err(SerError(
            "tuple structs not supported as cell values".into(),
        ))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerError> {
        Err(SerError(
            "tuple variants not supported as cell values".into(),
        ))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, SerError> {
        Err(SerError("maps not supported as cell values".into()))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, SerError> {
        Err(SerError(
            "nested structs not supported as cell values".into(),
        ))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, SerError> {
        Err(SerError(
            "struct variants not supported as cell values".into(),
        ))
    }
}

/// Write a slice of serializable structs to a worksheet as rows.
///
/// Row 0 gets the header (field names), and each struct becomes a subsequent row.
pub fn write_rows<T: Serialize>(
    ws: &mut crate::worksheet::Worksheet,
    data: &[T],
) -> crate::Result<()> {
    if data.is_empty() {
        return Ok(());
    }

    // Serialize the first item to get field names and values
    let mut row_ser = RowSerializer::new();
    data[0]
        .serialize(&mut row_ser)
        .map_err(|e| crate::Error::InvalidData(e.0))?;

    let fields = row_ser.fields.clone();
    let first_values = row_ser.values;

    // Write header row
    for (col, name) in fields.iter().enumerate() {
        ws.write(0, col as u16, name.as_str())?;
    }

    // Write first row of data
    write_ser_row(ws, 1, &first_values)?;

    // Write remaining rows
    for (i, item) in data[1..].iter().enumerate() {
        let mut ser = RowSerializer::new();
        ser.collecting_names = false;
        item.serialize(&mut ser)
            .map_err(|e| crate::Error::InvalidData(e.0))?;
        write_ser_row(ws, (i + 2) as u32, &ser.values)?;
    }

    Ok(())
}

fn write_ser_row(
    ws: &mut crate::worksheet::Worksheet,
    row: u32,
    values: &[SerValue],
) -> crate::Result<()> {
    for (col, val) in values.iter().enumerate() {
        match val {
            SerValue::String(s) => {
                ws.write(row, col as u16, s.as_str())?;
            }
            SerValue::Number(n) => {
                ws.write(row, col as u16, *n)?;
            }
            SerValue::Bool(b) => {
                ws.write(row, col as u16, *b)?;
            }
            SerValue::None => { /* leave cell empty */ }
        }
    }
    Ok(())
}
