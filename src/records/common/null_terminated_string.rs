use super::BStrw;
use crate::{
    parse::{tag, take_until, PResult, Parse},
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};

/// Null-terminated-string
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NullTerminatedString<'data> {
    pub value: BStrw<'data>,
}
impl<'data> NullTerminatedString<'data> {
    pub fn new(value: &'data BStr) -> NullTerminatedString<'data> {
        NullTerminatedString {
            value: BStrw::from(value),
        }
    }

    /// Note: ascii bytes should _not_ be null terminated
    pub fn from_ascii_bytes(value: &'data [u8]) -> NullTerminatedString<'data> {
        NullTerminatedString::new(value.as_bstr())
    }
}
impl<'data> Parse<'data> for NullTerminatedString<'data> {
    fn parse(data: &'data [u8]) -> PResult<Self> {
        let (data, info) = take_until(data, 0x00)?;
        let (data, _) = tag(data, &[0x00])?;
        Ok((data, NullTerminatedString::from_ascii_bytes(info)))
    }
}
impl<'data> DataSize for NullTerminatedString<'data> {
    fn data_size(&self) -> usize {
        self.value.len() + 0x00u8.data_size()
    }
}
impl<'data> Writable for NullTerminatedString<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.value.write_to(w)?;
        0x00u8.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nstring() {
        let s = NullTerminatedString::new(b"Test".as_bstr());
        assert_eq!(s.data_size(), 5);
        let mut data = Vec::new();
        data.reserve(5);
        s.write_to(&mut data).unwrap();
        assert_eq!(data.data_size(), 5);
        assert_eq!(data[0], b'T');
        assert_eq!(data[1], b'e');
        assert_eq!(data[2], b's');
        assert_eq!(data[3], b't');
        assert_eq!(data[4], 0x00);
    }
}
