//! YAML implementation for serduh
//! 
//! Provides `YamlSerializer` and `YamlDeserializer` for converting between
//! Rust types and YAML using the `yaml-parse` crate.

pub use serduh_core::{DeserializeOwned, Deserializer, Error, Serialize, Serializer};

use yaml_parse::{Value as YamlValue, ParseError};
use serduh_core::serde_value::Value;

/// YAML serializer - converts Rust types to yaml-parse::Value
pub struct YamlSerializer;

impl YamlSerializer {
    pub fn new() -> Self {
        Self
    }

    pub fn serialize<T: Serialize>(value: &T) -> Result<YamlValue, Error> {
        let serializer = Self::new();
        value.serialize(serializer)
    }
}

impl Default for YamlSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer for YamlSerializer {
    type Ok = YamlValue;
    type Error = Error;
    type SerializeSeq = SerializeSeq;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Number(v as u64))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Number(v as u64))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Number(v as u64))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Number(v as u64))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Number(v as u64))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Number(v as u64))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Number(v as u64))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Number(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Float(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::String(
            v.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(""),
        ))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Null)
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let mut map = Vec::new();
        map.push((variant.to_string(), value.serialize(self)?));
        Ok(YamlValue::Map(map))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq::new())
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeSeq, Self::Error> {
        let mut seq = SerializeSeq::new();
        seq.push(YamlValue::String(variant.to_string()));
        Ok(seq)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap::new())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct::new())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct::with_variant(variant))
    }
}

pub struct SerializeSeq {
    items: Vec<YamlValue>,
}

impl SerializeSeq {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn push(&mut self, item: YamlValue) {
        self.items.push(item);
    }
}

impl serduh_core::SerializeSeq for SerializeSeq {
    type Ok = YamlValue;
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.items.push(value.serialize(YamlSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::List(self.items))
    }
}

pub struct SerializeMap {
    items: Vec<(String, YamlValue)>,
    current_key: Option<String>,
}

impl SerializeMap {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            current_key: None,
        }
    }
}

impl serduh_core::SerializeMap for SerializeMap {
    type Ok = YamlValue;
    type Error = Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        let key_val = key.serialize(YamlSerializer)?;
        match key_val {
            YamlValue::String(s) => {
                self.current_key = Some(s);
                Ok(())
            }
            _ => Err(Error::ExpectedType {
                expected: "string",
                found: "non-string key",
            }),
        }
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self
            .current_key
            .take()
            .ok_or_else(|| Error::Custom("serialize_value called without key".to_string()))?;
        self.items.push((key, value.serialize(YamlSerializer)?));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(YamlValue::Map(self.items))
    }
}

pub struct SerializeStruct {
    fields: Vec<(String, YamlValue)>,
    variant: Option<String>,
}

impl SerializeStruct {
    fn new() -> Self {
        Self {
            fields: Vec::new(),
            variant: None,
        }
    }

    fn with_variant(variant: &'static str) -> Self {
        Self {
            fields: Vec::new(),
            variant: Some(variant.to_string()),
        }
    }
}

impl serduh_core::SerializeStruct for SerializeStruct {
    type Ok = YamlValue;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.fields
            .push((key.to_string(), value.serialize(YamlSerializer)?));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(variant) = self.variant {
            let mut map = vec![(variant, YamlValue::Map(self.fields))];
            Ok(YamlValue::Map(map))
        } else {
            Ok(YamlValue::Map(self.fields))
        }
    }
}

/// YAML deserializer - converts yaml-parse::Value to Rust types
pub struct YamlDeserializer {
    value: YamlValue,
}

impl YamlDeserializer {
    pub fn new(value: YamlValue) -> Self {
        Self { value }
    }

    pub fn deserialize<T: DeserializeOwned>(value: YamlValue) -> Result<T, Error> {
        let mut deserializer = Self::new(value);
        T::deserialize(&mut deserializer)
    }
}

impl<'de> Deserializer for &'de mut YamlDeserializer {
    type Error = Error;

    fn deserialize_bool(self) -> Result<bool, Self::Error> {
        match &self.value {
            YamlValue::Bool(v) => Ok(*v),
            _ => Err(Error::ExpectedType {
                expected: "bool",
                found: "non-bool",
            }),
        }
    }

    fn deserialize_i8(self) -> Result<i8, Self::Error> {
        match &self.value {
            YamlValue::Number(v) => Ok(*v as i8),
            _ => Err(Error::ExpectedType {
                expected: "number",
                found: "non-number",
            }),
        }
    }

    fn deserialize_i16(self) -> Result<i16, Self::Error> {
        match &self.value {
            YamlValue::Number(v) => Ok(*v as i16),
            _ => Err(Error::ExpectedType {
                expected: "number",
                found: "non-number",
            }),
        }
    }

    fn deserialize_i32(self) -> Result<i32, Self::Error> {
        match &self.value {
            YamlValue::Number(v) => Ok(*v as i32),
            _ => Err(Error::ExpectedType {
                expected: "number",
                found: "non-number",
            }),
        }
    }

    fn deserialize_i64(self) -> Result<i64, Self::Error> {
        match &self.value {
            YamlValue::Number(v) => Ok(*v as i64),
            _ => Err(Error::ExpectedType {
                expected: "number",
                found: "non-number",
            }),
        }
    }

    fn deserialize_u8(self) -> Result<u8, Self::Error> {
        match &self.value {
            YamlValue::Number(v) => Ok(*v as u8),
            _ => Err(Error::ExpectedType {
                expected: "number",
                found: "non-number",
            }),
        }
    }

    fn deserialize_u16(self) -> Result<u16, Self::Error> {
        match &self.value {
            YamlValue::Number(v) => Ok(*v as u16),
            _ => Err(Error::ExpectedType {
                expected: "number",
                found: "non-number",
            }),
        }
    }

    fn deserialize_u32(self) -> Result<u32, Self::Error> {
        match &self.value {
            YamlValue::Number(v) => Ok(*v as u32),
            _ => Err(Error::ExpectedType {
                expected: "number",
                found: "non-number",
            }),
        }
    }

    fn deserialize_u64(self) -> Result<u64, Self::Error> {
        match &self.value {
            YamlValue::Number(v) => Ok(*v),
            _ => Err(Error::ExpectedType {
                expected: "number",
                found: "non-number",
            }),
        }
    }

    fn deserialize_f32(self) -> Result<f32, Self::Error> {
        match &self.value {
            YamlValue::Float(v) => Ok(*v as f32),
            YamlValue::Number(v) => Ok(*v as f32),
            _ => Err(Error::ExpectedType {
                expected: "float",
                found: "non-float",
            }),
        }
    }

    fn deserialize_f64(self) -> Result<f64, Self::Error> {
        match &self.value {
            YamlValue::Float(v) => Ok(*v),
            YamlValue::Number(v) => Ok(*v as f64),
            _ => Err(Error::ExpectedType {
                expected: "float",
                found: "non-float",
            }),
        }
    }

    fn deserialize_char(self) -> Result<char, Self::Error> {
        match &self.value {
            YamlValue::String(ref s) if s.len() == 1 => Ok(s.chars().next().unwrap()),
            _ => Err(Error::ExpectedType {
                expected: "char",
                found: "non-char",
            }),
        }
    }

    fn deserialize_str(self) -> Result<String, Self::Error> {
        match &self.value {
            YamlValue::String(v) => Ok(v.clone()),
            _ => Err(Error::ExpectedType {
                expected: "string",
                found: "non-string",
            }),
        }
    }

    fn deserialize_string(self) -> Result<String, Self::Error> {
        self.deserialize_str()
    }

    fn deserialize_bytes(self) -> Result<Vec<u8>, Self::Error> {
        match &self.value {
            YamlValue::String(ref s) => {
                let bytes = s
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(2)
                    .map(|c| u8::from_str_radix(&format!("{}{}", c[0], c[1]), 16).unwrap_or(0))
                    .collect();
                Ok(bytes)
            }
            _ => Err(Error::ExpectedType {
                expected: "bytes",
                found: "non-bytes",
            }),
        }
    }

    fn deserialize_byte_buf(self) -> Result<Vec<u8>, Self::Error> {
        self.deserialize_bytes()
    }

    fn deserialize_option(self) -> Result<Option<()>, Self::Error> {
        match &self.value {
            YamlValue::Null => Ok(None),
            _ => Ok(Some(())),
        }
    }

    fn deserialize_unit(self) -> Result<(), Self::Error> {
        match &self.value {
            YamlValue::Null => Ok(()),
            _ => Err(Error::ExpectedType {
                expected: "null",
                found: "non-null",
            }),
        }
    }

    fn deserialize_unit_struct(self, _name: &'static str) -> Result<(), Self::Error> {
        self.deserialize_unit()
    }

    fn deserialize_newtype_struct<T>(self, _name: &'static str) -> Result<T, Self::Error>
    where
        T: DeserializeOwned,
    {
        T::deserialize(self)
    }

    fn deserialize_seq(self) -> Result<Vec<Value>, Self::Error> {
        match &self.value {
            YamlValue::List(items) => Ok(items.iter().map(|v| yaml_to_value(v.clone())).collect()),
            _ => Err(Error::ExpectedType {
                expected: "sequence",
                found: "non-sequence",
            }),
        }
    }

    fn deserialize_tuple(self, _len: usize) -> Result<Vec<Value>, Self::Error> {
        self.deserialize_seq()
    }

    fn deserialize_map(self) -> Result<Vec<(Value, Value)>, Self::Error> {
        match &self.value {
            YamlValue::Map(items) => Ok(items
                .iter()
                .map(|(k, v)| (Value::String(k.clone()), yaml_to_value(v.clone())))
                .collect()),
            _ => Err(Error::ExpectedType {
                expected: "map",
                found: "non-map",
            }),
        }
    }

    fn deserialize_struct(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
    ) -> Result<Value, Self::Error> {
        match &self.value {
            YamlValue::Map(items) => Ok(Value::Struct(
                items
                    .iter()
                    .map(|(k, v)| (k.clone(), yaml_to_value(v.clone())))
                    .collect(),
            )),
            _ => Err(Error::ExpectedType {
                expected: "struct",
                found: "non-struct",
            }),
        }
    }
}

// Helper to convert YamlValue to Value
fn yaml_to_value(yaml: YamlValue) -> Value {
    match yaml {
        YamlValue::Bool(b) => Value::Bool(b),
        YamlValue::Number(n) => Value::U64(n),
        YamlValue::Float(f) => Value::F64(f),
        YamlValue::String(s) => Value::String(s),
        YamlValue::List(items) => Value::Seq(items.into_iter().map(yaml_to_value).collect()),
        YamlValue::Map(items) => Value::Map(
            items
                .into_iter()
                .map(|(k, v)| (Value::String(k), yaml_to_value(v)))
                .collect(),
        ),
        YamlValue::Null => Value::Unit,
    }
}
