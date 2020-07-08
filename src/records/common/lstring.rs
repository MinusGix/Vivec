use crate::util::{StaticDataSize, Writable};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LString {
    /// An index into the string table.
    /// If the file is localized (see TES4 record), then ???, otherwise it points to a null terminated string
    pub index: u32,
}
impl LString {
    pub fn parse(data: &[u8]) -> nom::IResult<&[u8], Self> {
        let (data, index) = nom::number::complete::le_u32(data)?;
        Ok((data, Self { index }))
    }
}
impl StaticDataSize for LString {
    fn static_data_size() -> usize {
        u32::static_data_size()
    }
}
impl Writable for LString {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.index.write_to(w)
    }
}
