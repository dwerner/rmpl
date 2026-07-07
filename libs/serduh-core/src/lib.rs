//! Core traits for serduh serialization framework
//! 
//! This module provides the foundational traits for serializing and deserializing
//! Rust types. Implementations exist for various formats (YAML, JSON, etc.)

use std::fmt;

/// Error type for serialization/deserialization
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Expected a different type
    ExpectedType { expected: &'static str, found: &'static str },
    /// Missing required field
    MissingField(&'static str),
    /// Invalid value
    InvalidValue(String),
    /// Index out of bounds
    IndexOutOfBounds(usize),
    /// Custom error message
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ExpectedType { expected, found } => {
                write!(f, "expected {}, found {}", expected, found)
            }
            Error::MissingField(field) => write!(f, "missing field `{}`", field),
            Error::InvalidValue(msg) => write!(f, "invalid value: {}", msg),
            Error::IndexOutOfBounds(idx) => write!(f, "index {} out of bounds", idx),
            Error::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Custom(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Custom(s.to_string())
    }
}

/// Trait for types that can be serialized
pub trait Serialize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

/// Serializer trait - converts Rust types to a format-specific output
pub trait Serializer {
    type Ok;
    type Error;
    type SerializeSeq: SerializeSeq<Ok = Self::Ok, Error = Self::Error>;
    type SerializeMap: SerializeMap<Ok = Self::Ok, Error = Self::Error>;
    type SerializeStruct: SerializeStruct<Ok = Self::Ok, Error = Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error>;
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error>;
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error>;
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error>;
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error>;
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error>;
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error>;
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error>;
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error>;
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error>;
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error>;
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error>;
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error>;
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error>;
    fn serialize_none(self) -> Result<Self::Ok, Self::Error>;
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>;
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error>;
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error>;
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error>;
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>;
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>;
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error>;
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeSeq, Self::Error>;
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeSeq, Self::Error>;
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeSeq, Self::Error>;
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error>;
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error>;
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error>;
}

/// Sequence serializer
pub trait SerializeSeq {
    type Ok;
    type Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>;
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Map serializer
pub trait SerializeMap {
    type Ok;
    type Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>;
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>;
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Struct serializer
pub trait SerializeStruct {
    type Ok;
    type Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>;
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Deserializer trait - converts format-specific data to Rust types
pub trait Deserializer {
    type Error;

    fn deserialize_bool(self) -> Result<bool, Self::Error>;
    fn deserialize_i8(self) -> Result<i8, Self::Error>;
    fn deserialize_i16(self) -> Result<i16, Self::Error>;
    fn deserialize_i32(self) -> Result<i32, Self::Error>;
    fn deserialize_i64(self) -> Result<i64, Self::Error>;
    fn deserialize_u8(self) -> Result<u8, Self::Error>;
    fn deserialize_u16(self) -> Result<u16, Self::Error>;
    fn deserialize_u32(self) -> Result<u32, Self::Error>;
    fn deserialize_u64(self) -> Result<u64, Self::Error>;
    fn deserialize_f32(self) -> Result<f32, Self::Error>;
    fn deserialize_f64(self) -> Result<f64, Self::Error>;
    fn deserialize_char(self) -> Result<char, Self::Error>;
    fn deserialize_str(self) -> Result<String, Self::Error>;
    fn deserialize_string(self) -> Result<String, Self::Error>;
    fn deserialize_bytes(self) -> Result<Vec<u8>, Self::Error>;
    fn deserialize_byte_buf(self) -> Result<Vec<u8>, Self::Error>;
    fn deserialize_option(self) -> Result<Option<()>, Self::Error>;
    fn deserialize_unit(self) -> Result<(), Self::Error>;
    fn deserialize_unit_struct(self, name: &'static str) -> Result<(), Self::Error>;
    fn deserialize_newtype_struct<T>(self, name: &'static str) -> Result<T, Self::Error>
    where
        T: DeserializeOwned;
    fn deserialize_seq(self) -> Result<Vec<serde_value::Value>, Self::Error>;
    fn deserialize_tuple(self, len: usize) -> Result<Vec<serde_value::Value>, Self::Error>;
    fn deserialize_map(self) -> Result<Vec<(serde_value::Value, serde_value::Value)>, Self::Error>;
    fn deserialize_struct(
        self,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> Result<serde_value::Value, Self::Error>;
}

/// Trait for types that can be deserialized
pub trait DeserializeOwned: Sized {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error>;
}

/// Helper module for intermediate value representation
pub mod serde_value {
    use super::*;

    /// Intermediate value type for deserialization
    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Bool(bool),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        F32(f32),
        F64(f64),
        Char(char),
        String(String),
        Bytes(Vec<u8>),
        Seq(Vec<Value>),
        Map(Vec<(Value, Value)>),
        Struct(Vec<(String, Value)>),
        Unit,
    }

    impl Value {
        pub fn as_bool(&self) -> Option<bool> {
            match self {
                Value::Bool(v) => Some(*v),
                _ => None,
            }
        }

        pub fn as_i64(&self) -> Option<i64> {
            match self {
                Value::I64(v) => Some(*v),
                _ => None,
            }
        }

        pub fn as_u64(&self) -> Option<u64> {
            match self {
                Value::U64(v) => Some(*v),
                _ => None,
            }
        }

        pub fn as_f64(&self) -> Option<f64> {
            match self {
                Value::F64(v) => Some(*v),
                _ => None,
            }
        }

        pub fn as_str(&self) -> Option<&str> {
            match self {
                Value::String(v) => Some(v),
                _ => None,
            }
        }

        pub fn as_seq(&self) -> Option<&[Value]> {
            match self {
                Value::Seq(v) => Some(v),
                _ => None,
            }
        }

        pub fn as_map(&self) -> Option<&[(Value, Value)]> {
            match self {
                Value::Map(v) => Some(v),
                _ => None,
            }
        }

        pub fn as_struct(&self) -> Option<&[(String, Value)]> {
            match self {
                Value::Struct(v) => Some(v),
                _ => None,
            }
        }
    }
}

// Primitive implementations
impl Serialize for bool {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(*self)
    }
}

impl Serialize for i8 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i8(*self)
    }
}

impl Serialize for i16 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i16(*self)
    }
}

impl Serialize for i32 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i32(*self)
    }
}

impl Serialize for i64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(*self)
    }
}

impl Serialize for u8 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(*self)
    }
}

impl Serialize for u16 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(*self)
    }
}

impl Serialize for u32 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(*self)
    }
}

impl Serialize for u64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(*self)
    }
}

impl Serialize for f32 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f32(*self)
    }
}

impl Serialize for f64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(*self)
    }
}

impl Serialize for char {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(*self)
    }
}

impl Serialize for str {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self)
    }
}

impl Serialize for String {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self)
    }
}

impl Serialize for () {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit()
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Some(v) => serializer.serialize_some(v),
            None => serializer.serialize_none(),
        }
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for item in self {
            <S::SerializeSeq as SerializeSeq>::serialize_element(&mut seq, item)?;
        }
        <S::SerializeSeq as SerializeSeq>::end(seq)
    }
}

impl<K: Serialize, V: Serialize> Serialize for std::collections::HashMap<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            <S::SerializeMap as SerializeMap>::serialize_key(&mut map, k)?;
            <S::SerializeMap as SerializeMap>::serialize_value(&mut map, v)?;
        }
        <S::SerializeMap as SerializeMap>::end(map)
    }
}
