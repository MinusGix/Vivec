use super::BStrw;
use crate::{
    parse::PResult,
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};

/// String that is just bytes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FullString<'data> {
    pub value: BStrw<'data>,
}
impl<'data> FullString<'data> {
    pub fn new(value: &'data BStr) -> Self {
        Self {
            value: BStrw::from(value),
        }
    }

    /// Note: ascii bytes should _not_ be null terminated
    pub fn from_ascii_bytes(value: &'data [u8]) -> Self {
        Self::new(value.as_bstr())
    }

    pub fn parse(data: &'data [u8]) -> PResult<Self> {
        Ok((&[], Self::from_ascii_bytes(data)))
    }
}
impl<'data> DataSize for FullString<'data> {
    fn data_size(&self) -> usize {
        self.value.len()
    }
}
impl<'data> Writable for FullString<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.value.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nstring() {
        let s = FullString::new(b"Test".as_bstr());
        assert_eq!(s.data_size(), 4);
        let mut data = Vec::new();
        data.reserve(4);
        s.write_to(&mut data).unwrap();
        assert_eq!(data.data_size(), 4);
        assert_eq!(data[0], b'T');
        assert_eq!(data[1], b'e');
        assert_eq!(data[2], b's');
        assert_eq!(data[3], b't');
    }
}
