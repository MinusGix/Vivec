use crate::util::{byte, StaticDataSize, Writable};
use nom::{number::complete::le_u32, IResult};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FormId {
    pub id: u32,
}
impl FormId {
    pub fn new(id: u32) -> FormId {
        FormId { id }
    }

    pub fn from_bytes(id: [u8; 4]) -> FormId {
        FormId::new(byte::as_u32(&id))
    }

    pub fn parse(data: &[u8]) -> IResult<&[u8], FormId> {
        let (data, id) = le_u32(data)?;
        Ok((data, FormId::new(id)))
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        self.id.to_le_bytes()
    }
}
impl Writable for FormId {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.id.write_to(w)
    }
}
impl StaticDataSize for FormId {
    fn static_data_size() -> usize {
        4
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