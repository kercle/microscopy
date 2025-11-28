use std::string::{String, ToString};

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {std::format, ts_rs::TS, crate::std::borrow::ToOwned};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(type = "number | string | boolean"))]
pub enum Value {
    Number(f64),
    Text(String),
    Boolean(bool),
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Number(n) => serializer.serialize_f64(*n),
            Value::Text(s) => serializer.serialize_str(s),
            Value::Boolean(b) => serializer.serialize_bool(*b),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct InputValueVisitor;

        impl<'de> serde::de::Visitor<'de> for InputValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a number, string, or boolean")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Number(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Text(value.to_string()))
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Boolean(value))
            }
        }

        deserializer.deserialize_any(InputValueVisitor)
    }
}
