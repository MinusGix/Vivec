use super::common::{FromField, GeneralField};
use crate::{make_single_value_field, records::common::NullTerminatedString};
use nom::IResult;

make_single_value_field!(
    /// MUST BE NAMED EDID, currently this value is hardcoded.
    [Debug, Clone, Eq, PartialEq], EDID, id, NullTerminatedString, 'data);
impl<'data> FromField<'data> for EDID<'data> {
    fn from_field(field: GeneralField<'data>) -> IResult<&[u8], Self> {
        let (data, id) = NullTerminatedString::parse(field.data)?;
        // TODO: check that is all.
        Ok((data, Self { id }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_size_output;
    use bstr::ByteSlice;

    #[test]
    fn edid_test() {
        let edid = EDID {
            id: NullTerminatedString::new(b"Hello mortal".as_bstr()),
        };
        assert_size_output!(edid);
    }
}
