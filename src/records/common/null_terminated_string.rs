use super::BStrw;
use crate::util::{DataSize, Writable};
use bstr::{BStr, ByteSlice};
use nom::{
    bytes::complete::{tag, take_until},
    IResult,
};

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

    pub fn parse(data: &[u8]) -> IResult<&[u8], NullTerminatedString> {
        // TODO: i hate this reference thing
        let zero = [0u8];
        let zeror: &[u8] = &zero;
        let (data, info) = take_until(zeror)(data)?;
        let (data, _) = tag(zeror)(data)?;
        Ok((data, NullTerminatedString::from_ascii_bytes(info)))
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
impl<'data> DataSize for NullTerminatedString<'data> {
    fn data_size(&self) -> usize {
        self.value.len() + 0x00u8.data_size()
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
