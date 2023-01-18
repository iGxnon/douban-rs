use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use std::num::ParseIntError;
use std::{
    cmp::Ordering,
    convert::TryFrom,
    fmt::{self, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use uuid::Uuid;

pub struct UUID<T>(Uuid, PhantomData<T>);
pub enum Id<T> {
    Str(String, PhantomData<T>),
    U64(u64, PhantomData<T>),
}

impl<T> fmt::Debug for UUID<T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Id::Str(id, _) => fmt::Debug::fmt(id, f),
            Id::U64(id, _) => fmt::Debug::fmt(id, f),
        }
    }
}

impl<T> fmt::Display for UUID<T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Id::Str(id, _) => fmt::Display::fmt(id, f),
            Id::U64(id, _) => fmt::Display::fmt(id, f),
        }
    }
}

impl<T> Clone for UUID<T> {
    fn clone(&self) -> Self {
        UUID(self.0, PhantomData)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        match self {
            Id::Str(id, _) => Id::Str(id.clone(), PhantomData),
            Id::U64(id, _) => Id::U64(*id, PhantomData),
        }
    }
}

impl<T> Copy for UUID<T> {}

impl<T> PartialEq for UUID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.to_string().eq(&other.to_string())
    }
}

impl<T> Eq for UUID<T> {}
impl<T> Eq for Id<T> {}

impl<T> PartialOrd for UUID<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_string().partial_cmp(&other.to_string())
    }
}

impl<T> Ord for UUID<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl<T> Hash for UUID<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl<T> UUID<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        UUID(Uuid::new_v4(), PhantomData)
    }
}

impl<T> Id<T> {
    #[allow(clippy::new_without_default)]
    pub fn new_u64() -> Self {
        Id::U64(0, PhantomData)
    }
    #[allow(clippy::new_without_default)]
    pub fn new_str() -> Self {
        Id::Str("".to_string(), PhantomData)
    }

    pub fn as_string(&self) -> String {
        match self {
            Id::Str(id, _) => id.to_owned(),
            Id::U64(id, _) => id.to_string(),
        }
    }

    pub fn as_u64(&self) -> Result<u64, ParseIntError> {
        match self {
            Id::Str(id, _) => id.parse::<u64>(),
            Id::U64(id, _) => Ok(id.to_owned()),
        }
    }
}

impl<'a, T> TryFrom<&'a str> for UUID<T> {
    type Error = uuid::Error;

    fn try_from(id: &'a str) -> Result<Self, Self::Error> {
        Ok(UUID(Uuid::parse_str(id)?, PhantomData))
    }
}

impl<'a, T> From<&'a str> for Id<T> {
    fn from(id: &'a str) -> Self {
        Id::Str(id.to_string(), PhantomData)
    }
}

impl<T> From<u64> for Id<T> {
    fn from(id: u64) -> Self {
        Id::U64(id, PhantomData)
    }
}

impl<T> Serialize for UUID<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for UUID<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = Uuid::deserialize(deserializer)?;
        Ok(UUID(id, PhantomData))
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdVisitor<T>(PhantomData<T>);
        impl<'vi, T> serde::de::Visitor<'vi> for IdVisitor<T> {
            type Value = Id<T>;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                write!(formatter, "a str Id")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Id::Str(v.to_string(), PhantomData))
            }
        }
        deserializer.deserialize_str(IdVisitor(PhantomData))
    }
}

/** Generate a new `Id` randomly. */
pub struct NextUUID<T>(PhantomData<T>);

impl<T> Default for NextUUID<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> NextUUID<T> {
    pub fn new() -> Self {
        NextUUID(PhantomData)
    }

    pub fn next(&self) -> UUID<T> {
        UUID::new()
    }
}
