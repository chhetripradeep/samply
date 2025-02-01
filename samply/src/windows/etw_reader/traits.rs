use std::iter;

pub trait EncodeUtf16 {
    fn to_utf16(self) -> Vec<u16>;
}

impl EncodeUtf16 for &str {
    fn to_utf16(self) -> Vec<u16> {
        self.encode_utf16() // Make a UTF-16 iterator
            .chain(iter::once(0)) // Append a null
            .collect() // Collect the iterator into a vector
    }
}

impl EncodeUtf16 for String {
    fn to_utf16(self) -> Vec<u16> {
        self.as_str().to_utf16()
    }
}
