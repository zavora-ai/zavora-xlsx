//! Deserialize worksheet rows into structs.
//!
//! Uses a custom serde `Deserializer` that maps column headers to struct fields
//! and reads cell values from each row.

use crate::cell::CellValue;
use crate::worksheet::Worksheet;
use serde::de::{self, DeserializeSeed, MapAccess, Visitor};

/// Error type for serde deserialization within zavora-xlsx.
#[derive(Debug)]
pub struct DeError(pub(crate) String);

impl std::fmt::Display for DeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Serde deserialization error: {}", self.0)
    }
}

impl std::error::Error for DeError {}

impl de::Error for DeError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DeError(msg.to_string())
    }
}

/// Read rows from a worksheet and deserialize them into a Vec<T>.
///
/// Row 0 is treated as the header row (field names). Each subsequent row
/// is deserialized into a T using the header-to-field mapping.
pub fn read_rows<T: de::DeserializeOwned>(ws: &Worksheet) -> crate::Result<Vec<T>> {
    let range = ws.used_range();
    let (min_row, min_col, max_row, max_col) = match range {
        Some(r) => r,
        None => return Ok(Vec::new()),
    };

    if min_row == max_row {
        // Only header row, no data
        return Ok(Vec::new());
    }

    // Read header row
    let mut headers: Vec<String> = Vec::new();
    for col in min_col..=max_col {
        let val = ws.read_cell(min_row, col);
        let name = match &val {
            CellValue::String(s) => s.clone(),
            CellValue::Number(n) => n.to_string(),
            CellValue::Bool(b) => b.to_string(),
            _ => format!("col_{}", col),
        };
        headers.push(name);
    }

    // Read data rows
    let mut results = Vec::new();
    for row in (min_row + 1)..=max_row {
        let mut values: Vec<CellValue> = Vec::new();
        for col in min_col..=max_col {
            values.push(ws.read_cell(row, col));
        }

        let row_de = RowDeserializer {
            headers: &headers,
            values: &values,
        };
        let item = T::deserialize(row_de).map_err(|e| crate::Error::InvalidData(e.0))?;
        results.push(item);
    }

    Ok(results)
}

/// Deserializer for a single row.
struct RowDeserializer<'a> {
    headers: &'a [String],
    values: &'a [CellValue],
}

impl<'de, 'a> de::Deserializer<'de> for RowDeserializer<'a> {
    type Error = DeError;

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(DeError(
            "deserialize_any not supported; use a struct".into(),
        ))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_map(RowMapAccess {
            headers: self.headers,
            values: self.values,
            index: 0,
        })
    }

    // Forward all other types to deserialize_any
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

/// MapAccess that iterates over header/value pairs.
struct RowMapAccess<'a> {
    headers: &'a [String],
    values: &'a [CellValue],
    index: usize,
}

impl<'de, 'a> MapAccess<'de> for RowMapAccess<'a> {
    type Error = DeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        if self.index >= self.headers.len() {
            return Ok(None);
        }
        let key = seed.deserialize(de::value::StrDeserializer::new(&self.headers[self.index]))?;
        Ok(Some(key))
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let value = if self.index < self.values.len() {
            &self.values[self.index]
        } else {
            &CellValue::Empty
        };
        self.index += 1;
        seed.deserialize(CellValueDeserializer(value))
    }
}

/// Deserializer for a single CellValue.
struct CellValueDeserializer<'a>(&'a CellValue);

impl<'de, 'a> de::Deserializer<'de> for CellValueDeserializer<'a> {
    type Error = DeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            CellValue::String(s) => visitor.visit_string(s.clone()),
            CellValue::Number(n) => visitor.visit_f64(*n),
            CellValue::Bool(b) => visitor.visit_bool(*b),
            CellValue::Empty => visitor.visit_none(),
            CellValue::DateTime(dt) => visitor.visit_f64(dt.serial()),
            CellValue::Error(e) => Err(DeError(format!("cell contains error: {}", e))),
            CellValue::Formula { cached_value, .. } => {
                CellValueDeserializer(cached_value).deserialize_any(visitor)
            }
            CellValue::RichText(rt) => visitor.visit_string(rt.plain_text()),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            CellValue::Bool(b) => visitor.visit_bool(*b),
            CellValue::Number(n) => visitor.visit_bool(*n != 0.0),
            CellValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => visitor.visit_bool(true),
                "false" | "0" | "no" => visitor.visit_bool(false),
                _ => Err(DeError(format!("cannot convert '{}' to bool", s))),
            },
            CellValue::Empty => visitor.visit_bool(false),
            _ => Err(DeError(format!(
                "type mismatch: expected bool, got {:?}",
                self.0
            ))),
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            CellValue::Number(n) => visitor.visit_i64(*n as i64),
            CellValue::String(s) => {
                let n: i64 = s
                    .parse()
                    .map_err(|_| DeError(format!("cannot parse '{}' as integer", s)))?;
                visitor.visit_i64(n)
            }
            CellValue::Bool(b) => visitor.visit_i64(if *b { 1 } else { 0 }),
            CellValue::Empty => visitor.visit_i64(0),
            _ => Err(DeError(format!(
                "type mismatch: expected integer, got {:?}",
                self.0
            ))),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            CellValue::Number(n) => visitor.visit_u64(*n as u64),
            CellValue::String(s) => {
                let n: u64 = s
                    .parse()
                    .map_err(|_| DeError(format!("cannot parse '{}' as unsigned integer", s)))?;
                visitor.visit_u64(n)
            }
            CellValue::Bool(b) => visitor.visit_u64(if *b { 1 } else { 0 }),
            CellValue::Empty => visitor.visit_u64(0),
            _ => Err(DeError(format!(
                "type mismatch: expected unsigned integer, got {:?}",
                self.0
            ))),
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_f64(visitor)
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            CellValue::Number(n) => visitor.visit_f64(*n),
            CellValue::String(s) => {
                let n: f64 = s
                    .parse()
                    .map_err(|_| DeError(format!("cannot parse '{}' as float", s)))?;
                visitor.visit_f64(n)
            }
            CellValue::Bool(b) => visitor.visit_f64(if *b { 1.0 } else { 0.0 }),
            CellValue::Empty => visitor.visit_f64(0.0),
            CellValue::DateTime(dt) => visitor.visit_f64(dt.serial()),
            _ => Err(DeError(format!(
                "type mismatch: expected float, got {:?}",
                self.0
            ))),
        }
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            CellValue::String(s) if s.len() == 1 => visitor.visit_char(s.chars().next().unwrap()),
            _ => Err(DeError(format!(
                "type mismatch: expected char, got {:?}",
                self.0
            ))),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            CellValue::String(s) => visitor.visit_string(s.clone()),
            CellValue::Number(n) => visitor.visit_string(n.to_string()),
            CellValue::Bool(b) => visitor.visit_string(b.to_string()),
            CellValue::Empty => visitor.visit_string(String::new()),
            CellValue::RichText(rt) => visitor.visit_string(rt.plain_text()),
            _ => Err(DeError(format!(
                "type mismatch: expected string, got {:?}",
                self.0
            ))),
        }
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(DeError("bytes deserialization not supported".into()))
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(DeError("byte_buf deserialization not supported".into()))
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            CellValue::Empty => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(DeError(
            "sequence deserialization not supported for cells".into(),
        ))
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Err(DeError(
            "tuple deserialization not supported for cells".into(),
        ))
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Err(DeError(
            "tuple struct deserialization not supported for cells".into(),
        ))
    }

    fn deserialize_map<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(DeError(
            "map deserialization not supported for cells".into(),
        ))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Err(DeError(
            "nested struct deserialization not supported for cells".into(),
        ))
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.0 {
            CellValue::String(s) => visitor.visit_enum(de::value::StrDeserializer::new(s)),
            _ => Err(DeError(format!(
                "type mismatch: expected enum string, got {:?}",
                self.0
            ))),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }
}
