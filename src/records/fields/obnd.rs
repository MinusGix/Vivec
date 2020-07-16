use super::common::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE};
use crate::{
    impl_static_data_size, impl_static_type_named,
    parse::{PResult, Parse, ParseError},
    util::{Position3, Writable},
};
use bstr::BStr;

/// Object Bounds
/// bin format:
/// x1,y1,z1,x2,y2,z2
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct OBND {
    pub p1: Position3<i16>,
    pub p2: Position3<i16>,
}
impl OBND {
    const EXPECTED_BYTES: usize = 12;

    pub fn new(p1: Position3<i16>, p2: Position3<i16>) -> OBND {
        OBND { p1, p2 }
    }
}
impl FromField<'_> for OBND {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        if field.data.len() != OBND::EXPECTED_BYTES {
            return Err(ParseError::ExpectedExact {
                expected: OBND::EXPECTED_BYTES,
                found: field.data.len(),
            }
            .into());
        }

        let data = field.data;
        let (data, x1) = i16::parse(data)?;
        let (data, y1) = i16::parse(data)?;
        let (data, z1) = i16::parse(data)?;
        let (data, x2) = i16::parse(data)?;
        let (data, y2) = i16::parse(data)?;
        let (data, z2) = i16::parse(data)?;

        Ok((
            data,
            OBND::new(Position3::new(x1, y1, z1), Position3::new(x2, y2, z2)),
        ))
    }
}
impl_static_type_named!(OBND, b"OBND");
impl_static_data_size!(OBND, FIELDH_SIZE + Position3::<i16>::static_data_size() * 2);
impl Writable for OBND {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        write_field_header(self, w)?;
        self.p1.write_to(w)?;
        self.p2.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;
    #[test]
    fn obnd_test() {
        let obnd = OBND::new(Position3::new(4, 5, 9), Position3::new(52, 566, 42));
        assert_size_output!(obnd);
    }
}
