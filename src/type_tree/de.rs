use crate::TypeTreeValue;

#[derive(Debug)]
pub struct Deserializer<'de>(&'de TypeTreeValue);

impl<'de> Deserializer<'de> {
    pub fn new(input: &'de TypeTreeValue) -> Self {
        Self(input)
    }
}

#[cfg(feature = "serde")]
use serde::{de::value::{MapDeserializer, SeqDeserializer}, forward_to_deserialize_any};

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserializer<'de> for Deserializer<'de> {
    type Error = crate::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        match self.0 {
            TypeTreeValue::SInt8(v) => visitor.visit_i8(*v),
            TypeTreeValue::UInt8(v) => visitor.visit_u8(*v),
            TypeTreeValue::Char(v) => visitor.visit_char(*v),
            TypeTreeValue::SInt16(v) => visitor.visit_i16(*v),
            TypeTreeValue::UInt16(v) => visitor.visit_u16(*v),
            TypeTreeValue::SInt32(v) => visitor.visit_i32(*v),
            TypeTreeValue::UInt32(v) | TypeTreeValue::Type(v) => visitor.visit_u32(*v),
            TypeTreeValue::SInt64(v) => visitor.visit_i64(*v),
            TypeTreeValue::UInt64(v) | TypeTreeValue::FileSize(v) => visitor.visit_u64(*v),
            TypeTreeValue::Float(v) => visitor.visit_f32(*v),
            TypeTreeValue::Double(v) => visitor.visit_f64(*v),
            TypeTreeValue::Bool(v) => visitor.visit_bool(*v),
            TypeTreeValue::String(v) => visitor.visit_borrowed_str(v),
            TypeTreeValue::TypelessData(v) => visitor.visit_bytes(v),
            TypeTreeValue::Map(v) => visitor.visit_map(MapDeserializer::new(
                v.iter().map(|&(ref k, ref v)| (k, v)))
            ),
            TypeTreeValue::Array(v) => visitor.visit_seq(SeqDeserializer::new(v.iter())),
            TypeTreeValue::Class(v) => visitor.visit_map(MapDeserializer::new(
                v.iter().map(|(name, value)| (name.as_str(), value)))
            )
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        visitor.visit_some(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map
        struct newtype_struct enum identifier ignored_any
    }
}

#[cfg(all(test, feature = "objects"))]
mod tests {
    use std::collections::HashMap;

    use serde::Deserialize;

    use super::super::*;
    use crate::objects::classes::*;

    fn class_value(fields: Vec<(&str, Value)>) -> Value {
        Value::Class(fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    #[test]
    fn aabb_deserialization() {
        let value = class_value(vec![
            ("m_Center", Value::Class(HashMap::from([
                ("x".to_string(), Value::Float(1.0)),
                ("y".to_string(), Value::Float(2.0)),
                ("z".to_string(), Value::Float(3.0)),
            ]))),
            ("m_Extent", Value::Class(HashMap::from([
                ("x".to_string(), Value::Float(4.0)),
                ("y".to_string(), Value::Float(5.0)),
                ("z".to_string(), Value::Float(6.0)),
            ]))),
        ]);

        let deserializer = Deserializer::new(&value);
        let aabb = AABB::deserialize(deserializer).unwrap();
        assert_eq!(aabb.m_Center.x, 1.0);
        assert_eq!(aabb.m_Center.y, 2.0);
        assert_eq!(aabb.m_Center.z, 3.0);
        assert_eq!(aabb.m_Extent.x, 4.0);
        assert_eq!(aabb.m_Extent.y, 5.0);
        assert_eq!(aabb.m_Extent.z, 6.0);
    }

    #[test]
    fn astc_importer_deserialization() {
        let value = class_value(vec![
            ("m_AssetBundleName", Value::String("bundle".to_string())),
            ("m_AssetBundleVariant", Value::String("variant".to_string())),
            ("m_Name", Value::String("asset".to_string())),
            ("m_UserData", Value::String("data".to_string())),
        ]);

        let deserializer = Deserializer::new(&value);
        let astc = ASTCImporter::deserialize(deserializer).unwrap();
        assert_eq!(astc.m_AssetBundleName, "bundle");
        assert_eq!(astc.m_AssetBundleVariant, "variant");
        assert_eq!(astc.m_Name, "asset");
        assert_eq!(astc.m_UserData, "data");
    }

    #[test]
    fn added_game_object_deserialization() {
        let value = class_value(vec![
            ("addedObject", Value::Class(HashMap::from([
                ("m_FileID".to_string(), Value::SInt64(111)),
                ("m_PathID".to_string(), Value::SInt64(222)),
            ]))),
            ("insertIndex", Value::SInt32(2)),
            ("targetCorrespondingSourceObject", Value::Class(HashMap::from([
                ("m_FileID".to_string(), Value::SInt64(333)),
                ("m_PathID".to_string(), Value::SInt64(444)),
            ]))),
        ]);

        let deserializer = Deserializer::new(&value);
        let game_object = AddedGameObject::deserialize(deserializer).unwrap();
        assert_eq!(game_object.addedObject.m_FileID, 111);
        assert_eq!(game_object.addedObject.m_PathID, 222);
        assert_eq!(game_object.insertIndex, 2);
        assert_eq!(game_object.targetCorrespondingSourceObject.m_FileID, 333);
        assert_eq!(game_object.targetCorrespondingSourceObject.m_PathID, 444);
    }

    #[test]
    fn aim_constraint_deserialization() {
        let value = class_value(vec![
            ("m_AffectRotationX", Value::Bool(true)),
            ("m_AffectRotationY", Value::Bool(false)),
            ("m_AffectRotationZ", Value::Bool(true)),
            ("m_AimVector", Value::Class(HashMap::from([
                ("x".to_string(), Value::Float(0.0)),
                ("y".to_string(), Value::Float(1.0)),
                ("z".to_string(), Value::Float(0.0)),
            ]))),
            ("m_Enabled", Value::UInt8(1)),
            ("m_GameObject", Value::Class(HashMap::from([
                ("m_FileID".to_string(), Value::SInt64(555)),
                ("m_PathID".to_string(), Value::SInt64(666)),
            ]))),
            ("m_RotationAtRest", Value::Class(HashMap::from([
                ("x".to_string(), Value::Float(0.0)),
                ("y".to_string(), Value::Float(0.0)),
                ("z".to_string(), Value::Float(0.0)),
            ]))),
            ("m_RotationOffset", Value::Class(HashMap::from([
                ("x".to_string(), Value::Float(0.1)),
                ("y".to_string(), Value::Float(0.2)),
                ("z".to_string(), Value::Float(0.3)),
            ]))),
            ("m_Sources", Value::Array(vec![])),
            ("m_UpType", Value::SInt32(0)),
            ("m_UpVector", Value::Class(HashMap::from([
                ("x".to_string(), Value::Float(0.0)),
                ("y".to_string(), Value::Float(1.0)),
                ("z".to_string(), Value::Float(0.0)),
            ]))),
            ("m_Weight", Value::Float(1.0)),
            ("m_WorldUpObject", Value::Class(HashMap::from([
                ("m_FileID".to_string(), Value::SInt64(777)),
                ("m_PathID".to_string(), Value::SInt64(888)),
            ]))),
            ("m_WorldUpVector", Value::Class(HashMap::from([
                ("x".to_string(), Value::Float(0.0)),
                ("y".to_string(), Value::Float(1.0)),
                ("z".to_string(), Value::Float(0.0)),
            ]))),
            ("m_Active", Value::Bool(true)),
            ("m_IsContraintActive", Value::Bool(false)),
        ]);

        let deserializer = Deserializer::new(&value);
        let aim = AimConstraint::deserialize(deserializer).unwrap();
        assert_eq!(aim.m_AffectRotationX, true);
        assert_eq!(aim.m_AffectRotationY, false);
        assert_eq!(aim.m_AffectRotationZ, true);
        assert_eq!(aim.m_AimVector.x, 0.0);
        assert_eq!(aim.m_Enabled, 1);
        assert_eq!(aim.m_GameObject.m_FileID, 555);
        assert_eq!(aim.m_Weight, 1.0);
        assert_eq!(aim.m_Active, Some(true));
        assert_eq!(aim.m_IsContraintActive, Some(false));
    }

    #[test]
    fn animation_clip_deserialization() {
        let value = class_value(vec![
            ("m_Bounds", Value::Class(HashMap::from([
                ("m_Center".to_string(), Value::Class(HashMap::from([
                    ("x".to_string(), Value::Float(0.0)),
                    ("y".to_string(), Value::Float(0.0)),
                    ("z".to_string(), Value::Float(0.0)),
                ]))),
                ("m_Extent".to_string(), Value::Class(HashMap::from([
                    ("x".to_string(), Value::Float(1.0)),
                    ("y".to_string(), Value::Float(1.0)),
                    ("z".to_string(), Value::Float(1.0)),
                ]))),
            ]))),
            ("m_Compressed", Value::Bool(false)),
            ("m_CompressedRotationCurves", Value::Array(vec![])),
            ("m_Events", Value::Array(vec![])),
            ("m_FloatCurves", Value::Array(vec![])),
            ("m_Name", Value::String("clip".to_string())),
            ("m_PositionCurves", Value::Array(vec![])),
            ("m_RotationCurves", Value::Array(vec![])),
            ("m_SampleRate", Value::Float(60.0)),
            ("m_ScaleCurves", Value::Array(vec![])),
            ("m_WrapMode", Value::SInt32(0)),
            ("m_AnimationType", Value::SInt32(1)),
            ("m_HasGenericRootTransform", Value::Bool(true)),
            ("m_HasMotionFloatCurves", Value::Bool(false)),
            ("m_Legacy", Value::Bool(true)),
        ]);

        let deserializer = Deserializer::new(&value);
        let clip = AnimationClip::deserialize(deserializer).unwrap();
        assert_eq!(clip.m_Bounds.m_Center.x, 0.0);
        assert_eq!(clip.m_Compressed, false);
        assert_eq!(clip.m_Name, "clip");
        assert_eq!(clip.m_SampleRate, 60.0);
        assert_eq!(clip.m_WrapMode, 0);
        assert_eq!(clip.m_AnimationType, Some(1));
        assert_eq!(clip.m_HasGenericRootTransform, Some(true));
        assert_eq!(clip.m_HasMotionFloatCurves, Some(false));
        assert_eq!(clip.m_Legacy, Some(true));
    }
}