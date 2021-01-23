use anyhow::Result;
use serde_derive::Serialize;
#[macro_use]
extern crate erased_serde;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::fmt;

macro_rules! tri {
    ($e:expr) => {
        match $e {
            anyhow::Result::Ok(val) => val,
            anyhow::Result::Err(err) => return anyhow::Result::Err(err),
        }
    };
    ($e:expr,) => {
        tri!($e)
    };
}

pub trait Json: erased_serde::Serialize + std::fmt::Debug {
    fn is_str(&self) -> bool {
        false
    }

    fn is_i64(&self) -> bool {
        false
    }

    fn is_f64(&self) -> bool {
        false
    }

    fn is_bool(&self) -> bool {
        false
    }

    fn is_null(&self) -> bool {
        false
    }

    fn is_array(&self) -> bool {
        false
    }

    fn is_object(&self) -> bool {
        false
    }

    fn as_str(&self) -> Option<&str> {
        None
    }

    fn as_i64(&self) -> Option<i64> {
        None
    }

    fn as_f64(&self) -> Option<f64> {
        None
    }

    fn as_bool(&self) -> Option<bool> {
        None
    }

    fn as_array(&self) -> Option<&Vec<Box<dyn Json>>> {
        None
    }

    fn as_object(&self) -> Option<&BTreeMap<String, Box<dyn Json>>> {
        None
    }
}

#[serde(transparent)]
#[derive(Debug, Serialize)]
pub struct Str {
    value: String,
}

impl Json for Str {
    fn is_str(&self) -> bool {
        true
    }

    fn as_str(&self) -> Option<&str> {
        Some(self.value.as_str())
    }
}
#[serde(transparent)]
#[derive(Debug, Serialize)]
pub struct Int {
    value: i64,
}

impl Json for Int {
    fn is_i64(&self) -> bool {
        true
    }

    fn as_i64(&self) -> Option<i64> {
        Some(self.value)
    }
}
#[serde(transparent)]
#[derive(Debug, Serialize)]
pub struct Double {
    value: f64,
}

impl Json for Double {
    fn is_f64(&self) -> bool {
        true
    }
    fn as_f64(&self) -> Option<f64> {
        Some(self.value)
    }
}

#[serde(transparent)]
#[derive(Debug, Serialize)]
pub struct Bool {
    value: bool,
}

impl Json for Bool {
    fn is_bool(&self) -> bool {
        true
    }
    fn as_bool(&self) -> Option<bool> {
        Some(self.value)
    }
}
#[serde(transparent)]
#[derive(Debug, Serialize)]
pub struct Null {
    value: Option<u8>,
}

impl Json for Null {
    fn is_null(&self) -> bool {
        true
    }
}

serialize_trait_object!(Json);

#[serde(transparent)]
#[derive(Debug, Serialize)]
pub struct Array {
    value: Vec<Box<dyn Json>>,
}

impl Json for Array {
    fn is_array(&self) -> bool {
        true
    }

    fn as_array(&self) -> Option<&Vec<Box<dyn Json>>> {
        Some(&self.value)
    }
}

#[serde(transparent)]
#[derive(Debug, Serialize)]
pub struct Object {
    value: BTreeMap<String, Box<dyn Json>>,
}

impl Json for Object {
    fn is_object(&self) -> bool {
        true
    }

    fn as_object(&self) -> Option<&BTreeMap<String, Box<dyn Json>>> {
        Some(&self.value)
    }
}

impl<'de> Deserialize<'de> for Box<dyn Json> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Box<dyn Json>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct JsonVisitor;

        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = Box<dyn Json>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Box<dyn Json>, E> {
                Ok(Box::new(Bool { value }))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Box<dyn Json>, E> {
                Ok(Box::new(Int { value }))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Box<dyn Json>, E> {
                Ok(Box::new(Int {
                    value: value as i64,
                }))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Box<dyn Json>, E> {
                Ok(Box::new(Double { value }))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Box<dyn Json>, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Box<dyn Json>, E> {
                Ok(Box::new(Str { value }))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Box<dyn Json>, E> {
                Ok(Box::new(Null { value: None }))
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Box<dyn Json>, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Box<dyn Json>, E> {
                Ok(Box::new(Null { value: None }))
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Box<dyn Json>, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = tri!(visitor.next_element()) {
                    vec.push(elem);
                }

                Ok(Box::new(Array { value: vec }))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Box<dyn Json>, V::Error>
            where
                V: MapAccess<'de>,
            {
                match visitor.next_key_seed(KeyClassifier)? {
                    Some(KeyClass::Map(first_key)) => {
                        let mut values = BTreeMap::new();

                        values.insert(first_key, tri!(visitor.next_value()));
                        while let Some((key, value)) = tri!(visitor.next_entry()) {
                            values.insert(key, value);
                        }

                        Ok(Box::new(Object { value: values }))
                    }
                    None => Ok(Box::new(Object {
                        value: BTreeMap::new(),
                    })),
                }
            }
        }

        deserializer.deserialize_any(JsonVisitor)
    }
}

struct KeyClassifier;

enum KeyClass {
    Map(String),
    #[cfg(feature = "arbitrary_precision")]
    Number,
    #[cfg(feature = "raw_value")]
    RawValue,
}

impl<'de> DeserializeSeed<'de> for KeyClassifier {
    type Value = KeyClass;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyClassifier {
    type Value = KeyClass;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match s {
            #[cfg(feature = "arbitrary_precision")]
            crate::number::TOKEN => Ok(KeyClass::Number),
            #[cfg(feature = "raw_value")]
            crate::raw::TOKEN => Ok(KeyClass::RawValue),
            _ => Ok(KeyClass::Map(s.to_owned())),
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match s.as_str() {
            #[cfg(feature = "arbitrary_precision")]
            crate::number::TOKEN => Ok(KeyClass::Number),
            #[cfg(feature = "raw_value")]
            crate::raw::TOKEN => Ok(KeyClass::RawValue),
            _ => Ok(KeyClass::Map(s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn test_ser_map() {
        let mut map: BTreeMap<String, Box<dyn Json>> = BTreeMap::new();
        let k1 = Str {
            value: String::from("k1_val"),
        };

        let k2 = Str {
            value: String::from("k2_val"),
        };

        let k3 = Double { value: 0.0 };
        map.insert("k1".to_string(), Box::new(k1));
        map.insert("k2".to_string(), Box::new(k2));
        map.insert("k3".to_string(), Box::new(k3));

        let obj = Object { value: map };

        let str_val = serde_json::to_string_pretty(&obj).unwrap();
        println!("{}", str_val);
    }

    #[test]
    fn test_deser_map() {
        let literal = r#"
        {
            "k1": "k1_val",
            "k2": "k2_val",
            "k3": 0.0
        }
        "#;

        let json: Box<dyn Json> = serde_json::from_str(&literal).unwrap();

        let str = serde_json::to_string_pretty(&json).unwrap();

        println!("{}", str);
    }
}
