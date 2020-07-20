// Common item (ALCH, AMMO, etc) fields

use super::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE};
use crate::{
    impl_from_field, impl_static_data_size, impl_static_type_named, make_formid_field,
    make_single_value_field,
    parse::{PResult, Parse, ParseError},
    records::common::{lstring::LString, ConversionError, NullTerminatedString},
    util::Writable,
};
use std::{
    convert::{TryFrom, TryInto},
    io::Write,
};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Gold(u32);
impl Parse for Gold {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, value) = u32::parse(data)?;
        Ok((data, Self(value)))
    }
}
impl_static_data_size!(Gold, u32::static_data_size());
impl Writable for Gold {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.0.write_to(w)
    }
}
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Weight(f32);
impl Parse for Weight {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, value) = f32::parse(data)?;
        Ok((data, Self(value)))
    }
}
impl_static_data_size!(Weight, f32::static_data_size());
impl Writable for Weight {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.0.write_to(w)
    }
}

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
    pub quality: Quality,
}
impl_from_field!(QUAL, [quality: Quality]);
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
impl Parse for Quality {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, value) = u32::parse(data)?;
        let quality = value.try_into().map_err(|e| match e {
            ConversionError::InvalidEnumerationValue(_) => ParseError::InvalidEnumerationValue,
        })?;
        Ok((data, quality))
    }
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
impl TryFrom<u32> for Quality {
    type Error = ConversionError<u32>;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Quality::Novice,
            1 => Quality::Apprentice,
            2 => Quality::Journeyman,
            3 => Quality::Expert,
            4 => Quality::Master,
            _ => return Err(ConversionError::InvalidEnumerationValue(value)),
        })
    }
}

make_single_value_field!(
    /// Description
    [Debug, Copy, Clone, Eq, PartialEq],
    DESC,
    description,
    LString
);
impl_from_field!(DESC, [description: LString]);
