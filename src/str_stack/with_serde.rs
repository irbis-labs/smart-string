use core::fmt;

use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::ser::SerializeSeq;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::StrStack;

struct SeqVisitor;
struct SeqInPlaceVisitor<'a>(&'a mut StrStack);

impl<'de> Visitor<'de> for SeqVisitor {
    type Value = StrStack;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut stack = StrStack::new();
        while let Some(s) = seq.next_element()? {
            stack.push(s);
        }
        Ok(stack)
    }
}

impl<'a, 'de> Visitor<'de> for SeqInPlaceVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.0.clear();
        while let Some(s) = seq.next_element()? {
            self.0.push(s);
        }
        Ok(())
    }
}

impl Serialize for StrStack {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for s in self.iter() {
            seq.serialize_element(s)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for StrStack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SeqVisitor)
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SeqInPlaceVisitor(place))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let mut stack = StrStack::new();

        stack.push("123");
        stack.push("456");
        stack.push("789");

        let json = serde_json::to_string(&stack).unwrap();
        assert_eq!(json, r#"["123","456","789"]"#);
    }

    #[test]
    fn test_deserialize() {
        let json = r#"["123","456","789"]"#;
        let stack: StrStack = serde_json::from_str(json).unwrap();

        let mut it = stack.iter();
        assert_eq!(it.len(), 3);
        assert_eq!(it.next(), Some("123"));
        assert_eq!(it.next(), Some("456"));
        assert_eq!(it.next(), Some("789"));
        assert_eq!(it.next(), None);
    }
}
