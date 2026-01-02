use core::fmt;

use serde::de::Error;
use serde::de::Unexpected;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::SmartString;

struct StringVisitor<const CAPACITY: usize>;

struct StringInPlaceVisitor<'a, const CAPACITY: usize>(&'a mut SmartString<CAPACITY>);

// -------------------------------------------------------------------------------------------------

impl<'de, const CAPACITY: usize> Visitor<'de> for StringVisitor<CAPACITY> {
    type Value = SmartString<CAPACITY>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(SmartString::from(v))
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
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.0.clear();
        self.0.push_str(v);
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

impl<const CAPACITY: usize> Serialize for SmartString<CAPACITY> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self)
    }
}

impl<'de, const CAPACITY: usize> Deserialize<'de> for SmartString<CAPACITY> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_as_string() {
        let s = SmartString::<4>::from("abcde"); // heap
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r#""abcde""#);
    }

    #[test]
    fn test_deserialize_picks_stack_or_heap() {
        let s: SmartString<4> = serde_json::from_str(r#""abcd""#).unwrap();
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "abcd");

        let s: SmartString<4> = serde_json::from_str(r#""abcde""#).unwrap();
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "abcde");
    }

    #[test]
    fn test_deserialize_in_place_overwrites_existing_value() {
        // Start with a heap value, then overwrite it with a shorter string.
        //
        // Note: in-place deserialization overwrites the content but preserves the variant when possible
        // (clearing an existing `String` keeps it heap-backed).
        let mut place = SmartString::<4>::from("abcde");
        assert!(place.is_heap());

        let mut de = serde_json::Deserializer::from_str(r#""ab""#);
        SmartString::deserialize_in_place(&mut de, &mut place).unwrap();

        assert_eq!(place.as_str(), "ab");
        assert!(place.is_heap());
    }

    #[test]
    fn test_deserialize_in_place_promotes_stack_to_heap_if_needed() {
        let mut place = SmartString::<4>::from("ab");
        assert!(place.is_stack());

        let mut de = serde_json::Deserializer::from_str(r#""abcde""#);
        SmartString::deserialize_in_place(&mut de, &mut place).unwrap();

        assert_eq!(place.as_str(), "abcde");
        assert!(place.is_heap());
    }
}
