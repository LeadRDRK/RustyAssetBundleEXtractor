use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
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