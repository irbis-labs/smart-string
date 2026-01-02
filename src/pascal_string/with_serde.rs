use core::fmt;

use serde::de::Error;
use serde::de::Unexpected;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::pascal_string::TryFromStrError;
use crate::PascalString;

struct StringVisitor<const CAPACITY: usize>;

struct StringInPlaceVisitor<'a, const CAPACITY: usize>(&'a mut PascalString<CAPACITY>);

// -------------------------------------------------------------------------------------------------

impl<'de, const CAPACITY: usize> Visitor<'de> for StringVisitor<CAPACITY> {
    type Value = PascalString<CAPACITY>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a string no longer than {CAPACITY} bytes in length",
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        PascalString::try_from(v)
            .map_err(|TryFromStrError::TooLong| Error::invalid_length(v.len(), &self))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match core::str::from_utf8(v) {
            Ok(s) => self.visit_str(s),
            Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }
}

impl<'a, 'de, const CAPACITY: usize> Visitor<'de> for StringInPlaceVisitor<'a, CAPACITY> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a string no longer than {CAPACITY} bytes in length",
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.0.clear();
        self.0
            .try_push_str(v)
            .map_err(|TryFromStrError::TooLong| Error::invalid_length(v.len(), &self))?;
        Ok(())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match core::str::from_utf8(v) {
            Ok(s) => self.visit_str(s),
            Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }
}

// -------------------------------------------------------------------------------------------------

impl<const CAPACITY: usize> Serialize for PascalString<CAPACITY> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self)
    }
}

impl<'de, const CAPACITY: usize> Deserialize<'de> for PascalString<CAPACITY> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(StringVisitor)
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(StringInPlaceVisitor(place))
    }
}
