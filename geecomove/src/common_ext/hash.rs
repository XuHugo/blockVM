use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt,
    fmt::{Formatter, LowerHex},
    str::FromStr,
};

/// A hex encoded 32-byte hash value
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct HashValue(pub aptos_crypto::hash::HashValue);

impl From<aptos_crypto::hash::HashValue> for HashValue {
    fn from(val: aptos_crypto::hash::HashValue) -> Self {
        Self(val)
    }
}

impl From<HashValue> for aptos_crypto::hash::HashValue {
    fn from(val: HashValue) -> Self {
        val.0
    }
}

impl FromStr for HashValue {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, anyhow::Error> {
        if let Some(hex) = s.strip_prefix("0x") {
            Ok(hex.parse::<aptos_crypto::hash::HashValue>()?.into())
        } else {
            Ok(s.parse::<aptos_crypto::hash::HashValue>()?.into())
        }
    }
}

impl Serialize for HashValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HashValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hash = <String>::deserialize(deserializer)?;
        hash.parse().map_err(D::Error::custom)
    }
}

impl fmt::Display for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl LowerHex for HashValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl HashValue {
    pub fn zero() -> Self {
        Self(aptos_crypto::hash::HashValue::zero())
    }
}
