use crate::{impl_from_field, make_single_value_field, records::common::NullTerminatedString};

make_single_value_field!(
    /// MUST BE NAMED EDID, currently this value is hardcoded.
    [Debug, Clone, Eq, PartialEq], EDID, id, NullTerminatedString, 'data);
impl_from_field!(EDID, 'data, [id: NullTerminatedString]);

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
