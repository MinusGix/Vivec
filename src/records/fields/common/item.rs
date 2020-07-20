// Common item (ALCH, AMMO, etc) fields

use super::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE};
use crate::{
    impl_from_field, impl_static_data_size, impl_static_type_named, make_formid_field,
    make_single_value_field,
    parse::{take, PResult, Parse, ParseError},
    records::common::{lstring::LString, ConversionError, NullTerminatedString},
    util::{DataSize, StaticDataSize, Writable},
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
impl_from_field!(ICON, 'data, [filename: NullTerminatedString]);

make_single_value_field!(
    /// Message icon filename
    [Debug, Clone],
    MICO,
    filename,
    NullTerminatedString,
    'data
);
impl_from_field!(MICO, 'data, [filename: NullTerminatedString]);

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

#[derive(Debug, Clone)]
pub struct BODT {
    pub part_node_flags: BodyPartNodeFlags,
    pub flags: BODTFlags,
    /// UESP thinks this is junk data. it is in the padding position
    pub unknown: [u8; 3],
    /// Some rare records of BODT do not have the skill field.
    /// It defaults to ArmorSkill::None, but we can't / shouldn't store it like that.
    pub skill: Option<ArmorSkill>,
}
impl FromField<'_> for BODT {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, part_node_flags) = BodyPartNodeFlags::parse(field.data)?;
        let (data, flags) = BODTFlags::parse(data)?;
        let (data, unknown) = take(data, 3)?;
        let unknown = [unknown[0], unknown[1], unknown[2]];
        let (data, skill) = if !data.is_empty() {
            let (data, skill) = ArmorSkill::parse(data)?;
            (data, Some(skill))
        } else {
            (data, None)
        };

        Ok((
            data,
            Self {
                part_node_flags,
                flags,
                unknown,
                skill,
            },
        ))
    }
}
impl_static_type_named!(BODT, b"BODT");
impl DataSize for BODT {
    fn data_size(&self) -> usize {
        FIELDH_SIZE
            + self.part_node_flags.data_size()
            + self.flags.data_size()
            + (u8::static_data_size() * 3)
            + self.skill.data_size()
    }
}
impl Writable for BODT {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.part_node_flags.write_to(w)?;
        self.flags.write_to(w)?;
        self.unknown[0].write_to(w)?;
        self.unknown[1].write_to(w)?;
        self.unknown[2].write_to(w)?;
        if let Some(skill) = &self.skill {
            skill.write_to(w)?;
        }
        Ok(())
    }
}

// TODO: implement getters and comments on bit meanings
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BodyPartNodeFlags {
    pub flags: u32,
}
impl Parse for BodyPartNodeFlags {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = u32::parse(data)?;
        Ok((data, Self { flags }))
    }
}
impl_static_data_size!(BodyPartNodeFlags, u32::static_data_size());
impl Writable for BodyPartNodeFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

// TODO: implement getters and comments on bit meanings
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BODTFlags {
    /// 0x1: Modulates voice. (ARMA only)
    /// 0x10: Non-playable (ARMO only)
    pub flags: u8,
}
impl Parse for BODTFlags {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = u8::parse(data)?;
        Ok((data, Self { flags }))
    }
}
impl_static_data_size!(BODTFlags, u8::static_data_size());
impl Writable for BODTFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum ArmorSkill {
    LightArmor = 0,
    HeavyArmor = 1,
    /// No armor value
    None = 2,
}
impl ArmorSkill {
    pub fn code(&self) -> u32 {
        *self as u32
    }
}
impl Parse for ArmorSkill {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, value) = u32::parse(data)?;
        let skill = value.try_into().map_err(|e| match e {
            ConversionError::InvalidEnumerationValue(_) => ParseError::InvalidEnumerationValue,
        })?;
        Ok((data, skill))
    }
}
impl_static_data_size!(ArmorSkill, u32::static_data_size());
impl Writable for ArmorSkill {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.code().write_to(w)
    }
}
impl TryFrom<u32> for ArmorSkill {
    type Error = ConversionError<u32>;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => ArmorSkill::LightArmor,
            1 => ArmorSkill::HeavyArmor,
            2 => ArmorSkill::None,
            _ => return Err(ConversionError::InvalidEnumerationValue(value)),
        })
    }
}

/// Essentially a 'new'/'updated' trimmed down version of BODT
#[derive(Debug, Clone)]
pub struct BOD2 {
    pub part_node_flags: BodyPartNodeFlags,
    pub skill: ArmorSkill,
}
impl_from_field!(
    BOD2,
    [part_node_flags: BodyPartNodeFlags, skill: ArmorSkill]
);
impl_static_type_named!(BOD2, b"BOD2");
impl_static_data_size!(
    BOD2,
    FIELDH_SIZE + BodyPartNodeFlags::static_data_size() + ArmorSkill::static_data_size()
);
impl Writable for BOD2 {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.part_node_flags.write_to(w)?;
        self.skill.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;

    #[test]
    fn test_bodt() {
        let bodt = BODT {
            part_node_flags: BodyPartNodeFlags { flags: 0x0 },
            flags: BODTFlags { flags: 0x0 },
            unknown: [0, 0, 0],
            skill: Some(ArmorSkill::HeavyArmor),
        };
        assert_size_output!(bodt);
    }

    #[test]
    fn test_bod2() {
        let bod2 = BOD2 {
            part_node_flags: BodyPartNodeFlags { flags: 0x0 },
            skill: ArmorSkill::HeavyArmor,
        };
        assert_size_output!(bod2);
    }
}
