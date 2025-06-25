use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{de::IntoDeserializer, Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
pub enum Value {
    SInt8(i8),
    UInt8(u8),
    Char(char),
    SInt16(i16),
    UInt16(u16),
    SInt32(i32),
    UInt32(u32),
    Type(u32),
    SInt64(i64),
    UInt64(u64),
    FileSize(u64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(String),
    TypelessData(Vec<u8>),
    Map(Vec<(Value, Value)>),
    Array(Vec<Value>),
    Class(HashMap<String, Value>)
}

#[cfg(feature = "serde")]
impl<'de> IntoDeserializer<'de, crate::Error> for &'de Value {
    type Deserializer = super::Deserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        Self::Deserializer::new(self)
    }
}

impl Value {
    pub fn i8(&self) -> Option<i8> {
        match self {
            Self::SInt8(v) => Some(*v),
            _ => None
        }
    }

    pub fn u8(&self) -> Option<u8> {
        match self {
            Self::UInt8(v) => Some(*v),
            _ => None
        }
    }

    pub fn char(&self) -> Option<char> {
        match self {
            Self::Char(v) => Some(*v),
            _ => None
        }
    }

    pub fn i16(&self) -> Option<i16> {
        match self {
            Self::SInt16(v) => Some(*v),
            _ => None
        }
    }

    pub fn u16(&self) -> Option<u16> {
        match self {
            Self::UInt16(v) => Some(*v),
            _ => None
        }
    }

    pub fn i32(&self) -> Option<i32> {
        match self {
            Self::SInt32(v) => Some(*v),
            _ => None
        }
    }

    pub fn u32(&self) -> Option<u32> {
        match self {
            Self::UInt32(v) | Self::Type(v) => Some(*v),
            _ => None
        }
    }

    pub fn i64(&self) -> Option<i64> {
        match self {
            Self::SInt64(v) => Some(*v),
            _ => None
        }
    }

    pub fn u64(&self) -> Option<u64> {
        match self {
            Self::UInt64(v) | Self::FileSize(v) => Some(*v),
            _ => None
        }
    }

    pub fn f32(&self) -> Option<f32> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None
        }
    }

    pub fn f64(&self) -> Option<f64> {
        match self {
            Self::Double(v) => Some(*v),
            _ => None
        }
    }

    pub fn bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None
        }
    }

    pub fn string(&self) -> Option<&String> {
        match self {
            Self::String(v) => Some(v),
            _ => None
        }
    }

    pub fn typeless_data(&self) -> Option<&Vec<u8>> {
        match self {
            Self::TypelessData(v) => Some(v),
            _ => None
        }
    }

    pub fn map(&self) -> Option<&Vec<(Value, Value)>> {
        match self {
            Self::Map(v) => Some(v),
            _ => None
        }
    }

    pub fn array(&self) -> Option<&Vec<Value>> {
        match self {
            Self::Array(v) => Some(v),
            _ => None
        }
    }

    pub fn class(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Self::Class(v) => Some(v),
            _ => None
        }
    }

    #[cfg(feature = "serde")]
    pub fn parse<'de, T: serde::Deserialize<'de>>(&'de self) -> Result<T, crate::Error> {
        T::deserialize(self.into_deserializer())
    }
}