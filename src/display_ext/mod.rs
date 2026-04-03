use std::fmt;
use std::io;

use crate::PascalString;

pub trait DisplayExt {
    fn is_empty(&self) -> bool;

    fn write_to_fmt<W: fmt::Write>(&self, writer: W) -> fmt::Result;

    fn write_to_bytes<W: io::Write>(&self, writer: W) -> fmt::Result;

    fn try_to_fmt<T: fmt::Write + Default>(&self) -> Result<T, T> {
        let mut writer = T::default();
        match self.write_to_fmt(&mut writer) {
            Ok(_) => Ok(writer),
            Err(_err) => Err(writer),
        }
    }

    fn try_to_bytes<T: io::Write + Default>(&self) -> Result<T, T> {
        let mut writer = T::default();
        match self.write_to_bytes(&mut writer) {
            Ok(_) => Ok(writer),
            Err(_err) => Err(writer),
        }
    }

    fn to_fmt<T: fmt::Write + Default>(&self) -> T {
        self.try_to_fmt()
            .unwrap_or_else(|_writer| panic!("Failed to write to target"))
    }

    fn to_bytes<T: io::Write + Default>(&self) -> T {
        self.try_to_bytes()
            .unwrap_or_else(|_writer| panic!("Failed to write to target"))
    }

    fn format_with<F>(&self, f: F) -> fmt::Result
    where
        F: FnMut(Option<&str>) -> fmt::Result;
}

impl<T> DisplayExt for T
where
    T: fmt::Display + ?Sized,
{
    fn is_empty(&self) -> bool {
        self.write_to_fmt(PascalString::<0>::new()).is_ok()
    }

    fn write_to_fmt<W: fmt::Write>(&self, mut writer: W) -> fmt::Result {
        write!(writer, "{}", self)
    }

    fn write_to_bytes<W: io::Write>(&self, mut writer: W) -> fmt::Result {
        writer
            .write_fmt(format_args!("{}", self))
            .map_err(|_| fmt::Error)
    }

    #[inline]
    fn format_with<F>(&self, mut cb: F) -> fmt::Result
    where
        F: FnMut(Option<&str>) -> fmt::Result,
    {
        use fmt::Write;

        struct CallbackWrapper<F>(F);

        impl<F> Write for CallbackWrapper<F>
        where
            F: FnMut(Option<&str>) -> fmt::Result,
        {
            #[inline]
            fn write_str(&mut self, s: &str) -> fmt::Result {
                (self.0)(Some(s))
            }
        }

        CallbackWrapper(&mut cb).write_fmt(format_args!("{}", self))?;
        (cb)(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Maybe<T>(Option<T>);

    impl<T: fmt::Display> fmt::Display for Maybe<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if let Some(t) = &self.0 {
                t.fmt(f)?
            }
            Ok(())
        }
    }

    fn is_empty(display: impl fmt::Display) -> bool {
        display.is_empty()
    }

    #[test]
    fn test_is_empty() {
        assert!(is_empty(""));
        assert!(!is_empty("Hello"));

        assert!(is_empty("".to_string()));
        assert!(!is_empty("Hello".to_string()));

        assert!(is_empty(&"" as &dyn fmt::Display));
        assert!(!is_empty(&"Hello" as &dyn fmt::Display));

        assert!(is_empty(&"".to_string() as &dyn fmt::Display));
        assert!(!is_empty(&"Hello".to_string() as &dyn fmt::Display));

        assert!(is_empty(Maybe(None::<&str>)));
        assert!(is_empty(Maybe(Some(""))));
        assert!(!is_empty(Maybe(Some("Hello"))));
    }

    #[test]
    fn test_to_fmt() {
        assert_eq!("".to_fmt::<PascalString<0>>(), "");
        assert_eq!("", "".to_fmt::<PascalString<0>>());
        assert_eq!("Hello", "Hello".to_fmt::<PascalString<255>>());
        assert_eq!("Hello", "Hello".try_to_fmt::<PascalString<255>>().unwrap());
        assert!("".try_to_fmt::<PascalString<4>>().is_ok());
        assert!("Hello".try_to_fmt::<PascalString<4>>().is_err());
    }

    #[test]
    #[should_panic]
    fn test_to_fmt_panic() {
        "Hello".to_fmt::<PascalString<4>>();
    }

    #[test]
    fn test_write_to_bytes() {
        let mut buf = Vec::new();
        "Hello".write_to_bytes(&mut buf).unwrap();
        assert_eq!(buf, b"Hello");
    }

    #[test]
    fn test_write_to_bytes_empty() {
        let mut buf = Vec::new();
        "".write_to_bytes(&mut buf).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_try_to_bytes() {
        let result: Result<Vec<u8>, Vec<u8>> = "Hello".try_to_bytes();
        assert_eq!(result.unwrap(), b"Hello");
    }

    #[test]
    fn test_to_bytes() {
        let result: Vec<u8> = "Hello".to_bytes();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_format_with_collects_chunks() {
        let mut chunks = Vec::new();
        "Hello"
            .format_with(|chunk| {
                chunks.push(chunk.map(String::from));
                Ok(())
            })
            .unwrap();
        // Last call is None (terminator)
        assert_eq!(chunks.last().unwrap(), &None);
        // At least one Some chunk with the content
        let content: String = chunks
            .iter()
            .filter_map(|c| c.as_ref().map(|s| s.as_str()))
            .collect();
        assert_eq!(content, "Hello");
    }

    #[test]
    fn test_format_with_empty() {
        let mut chunks = Vec::new();
        "".format_with(|chunk| {
            chunks.push(chunk.map(String::from));
            Ok(())
        })
        .unwrap();
        // Should get at least the None terminator
        assert_eq!(chunks.last().unwrap(), &None);
    }

    #[test]
    fn test_format_with_number() {
        let mut content = String::new();
        42u32
            .format_with(|chunk| {
                if let Some(s) = chunk {
                    content.push_str(s);
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(content, "42");
    }
}
