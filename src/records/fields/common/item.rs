// Common item (ALCH, AMMO, etc) fields

use super::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE};
use crate::{
    impl_static_data_size, impl_static_type_named, make_formid_field, make_single_value_field,
    parse::PResult, records::common::NullTerminatedString, util::Writable,
};
use std::io::Write;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct QUAL {
    quality: Quality,
}
impl_static_type_named!(QUAL, b"QUAL");
impl_static_data_size!(QUAL, FIELDH_SIZE + Quality::static_data_size());
impl Writable for QUAL {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.quality.write_to(w)
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Quality {
    Novice = 0,
    Apprentice = 1,
    Journeyman = 2,
    Expert = 3,
    Master = 4,
}
impl_static_data_size!(Quality, u32::static_data_size());
impl Writable for Quality {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        (*self as u32).write_to(w)
    }
}
