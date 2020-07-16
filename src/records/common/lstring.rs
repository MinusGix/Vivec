use crate::{impl_static_data_size, util::Writable};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LString {
    /// An index into the string table.
    /// If the file is localized (see TES4 record), then ???, otherwise it points to a null terminated string
    pub index: u32,
}
impl LString {
    pub fn parse(data: &[u8]) -> crate::parse::PResult<Self> {
        use crate::parse::Parse;
        let (data, index) = u32::parse(data)?;
        Ok((data, Self { index }))
    }
}
impl_static_data_size!(LString, u32::static_data_size());
impl Writable for LString {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.index.write_to(w)
    }
}
