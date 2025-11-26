use std::string::{String, ToString};

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {std::format, ts_rs::TS, crate::std::borrow::ToOwned};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(type = "number | string | boolean"))]
pub enum InputValue {
    Number(f64),
    Text(String),
    Boolean(bool),
}

impl Serialize for InputValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            InputValue::Number(n) => serializer.serialize_f64(*n),
            InputValue::Text(s) => serializer.serialize_str(s),
            InputValue::Boolean(b) => serializer.serialize_bool(*b),
        }
    }
}

impl<'de> Deserialize<'de> for InputValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct InputValueVisitor;

        impl<'de> serde::de::Visitor<'de> for InputValueVisitor {
            type Value = InputValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a number, string, or boolean")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(InputValue::Number(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(InputValue::Text(value.to_string()))
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(InputValue::Boolean(value))
            }
        }

        deserializer.deserialize_any(InputValueVisitor)
    }
}
