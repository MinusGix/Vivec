use super::BStrw;
use crate::{
    parse::{le_u16, take, PResult},
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};

/// A string that is prefixed by 2 bytes for the length
/// and is encoded in Windows-1252
/// TODO: for now we just store it as a Byte-string, rather than properly decoding/encoding it.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Windows1252String16<'data> {
    pub value: BStrw<'data>,
}
impl<'data> Windows1252String16<'data> {
    pub fn new(value: &'data BStr) -> Windows1252String16<'data> {
        Windows1252String16 {
            value: BStrw::from(value),
        }
    }

    pub fn from_ascii_bytes(value: &'data [u8]) -> Windows1252String16<'data> {
        Windows1252String16::new(value.as_bstr())
    }

    pub fn parse(data: &'data [u8]) -> PResult<Self> {
        // TODO: test that this is little endian
        let (data, length) = le_u16(data)?;
        let (data, string) = take(data, length as usize)?;
        Ok((data, Windows1252String16::from_ascii_bytes(string)))
    }
}
impl<'data> Writable for Windows1252String16<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        // TODO: assert length fits within usize
        (self.value.len() as u16).write_to(w)?;
        self.value.write_to(w)
    }
}
impl<'data> DataSize for Windows1252String16<'data> {
    fn data_size(&self) -> usize {
        2 + self.value.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wstring() {
        let w = Windows1252String16::new(b"Test".as_bstr());
        assert_eq!(w.data_size(), 6);
        let mut data = Vec::new();
        data.reserve(6);
        w.write_to(&mut data).unwrap();
        assert_eq!(data.data_size(), 6);
        assert_eq!(data[0], 0x04);
        assert_eq!(data[1], 0x00);
        assert_eq!(data[2], b'T');
        assert_eq!(data[3], b'e');
        assert_eq!(data[4], b's');
        assert_eq!(data[5], b't');
    }
}
