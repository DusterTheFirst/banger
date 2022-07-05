use std::{fmt::Display, str::FromStr};

use serde::{de, Deserialize, Serializer};

pub fn serialize<S, T>(to_string: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    serializer.serialize_str(&to_string.to_string())
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FromStr,
    T::Err: Display,
{
    T::from_str(&String::deserialize(deserializer)?).map_err(de::Error::custom)
}
