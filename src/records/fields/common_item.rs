// Common item (ALCH, AMMO, etc) fields

use super::common::{FromField, FromFieldError, GeneralField};
use crate::{
    make_formid_field, make_single_value_field, parse::PResult,
    records::common::NullTerminatedString,
};

make_single_value_field!(
    /// Inventory icon filename
    [Debug, Clone],
    ICON,
    filename,
    NullTerminatedString,
    'data
);
impl<'data> FromField<'data> for ICON<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, filename) = NullTerminatedString::parse(field.data)?;
        Ok((data, Self { filename }))
    }
}

make_single_value_field!(
    /// Message icon filename
    [Debug, Clone],
    MICO,
    filename,
    NullTerminatedString,
    'data
);
impl<'data> FromField<'data> for MICO<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, filename) = NullTerminatedString::parse(field.data)?;
        Ok((data, Self { filename }))
    }
}

make_formid_field!(
    /// Pickup ->SNDR
    YNAM
);

make_formid_field!(
    /// Drop ->SNDR
    ZNAM
);
