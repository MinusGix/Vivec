use crate::{
    impl_static_data_size,
    parse::{PResult, Parse},
    util::Writable,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FormId {
    pub id: u32,
}
impl FormId {
    pub fn new(id: u32) -> FormId {
        FormId { id }
    }

    pub fn from_bytes(id: [u8; 4]) -> FormId {
        FormId::new(u32::from_le_bytes(id))
    }

    pub fn parse(data: &[u8]) -> PResult<FormId> {
        let (data, id) = u32::parse(data)?;
        Ok((data, FormId::new(id)))
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        self.id.to_le_bytes()
    }
}
impl_static_data_size!(FormId, u32::static_data_size());
impl Writable for FormId {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.id.write_to(w)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::DataSize;
    #[test]
    fn test_formid() {
        let formid = FormId::new(0xaa44926b);
        assert_eq!(formid.data_size(), 4);
        let mut data = Vec::new();
        data.reserve(4);
        formid.write_to(&mut data).unwrap();
        println!("data: {:?}", data);
        assert_eq!(data.len(), 4);
        assert_eq!(data[0], 0x6b);
        assert_eq!(data[1], 0x92);
        assert_eq!(data[2], 0x44);
        assert_eq!(data[3], 0xaa);
    }
}
