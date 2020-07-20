use super::{
    common::{
        get_field, CommonRecordInfo, ConversionError, FromRecord, FromRecordError, GeneralRecord,
        StaticTypeNamed, TypeNamed,
    },
    fields::{
        common::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE},
        edid,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_from_field, impl_static_data_size,
    impl_static_type_named, make_formid_field, make_model_fields, make_single_value_field,
    parse::{take, PResult, Parse, ParseError},
    util::{DataSize, StaticDataSize, Writable},
};
use derive_more::From;
use std::{
    convert::{TryFrom, TryInto},
    io::Write,
};

#[derive(Debug, Clone)]
pub struct ARMARecord<'data> {
    pub common: CommonRecordInfo,
    pub fields: Vec<ARMAField<'data>>,
}
impl<'data> FromRecord<'data> for ARMARecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut edid_index = None;
        let mut bodt_index = None;
        let mut bod2_index = None;
        let mut rnam_index = None;
        let mut dnam_index = None;
        let mut mod2_index = None;
        let mut mod3_index = None;
        let mut mod4_index = None;
        let mut mod5_index = None;
        let mut nam0_index = None;
        let mut nam1_index = None;
        let mut nam2_index = None;
        let mut nam3_index = None;
        let mut modl_list_index = None;
        let mut sndd_index = None;
        let mut onam_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();
        while let Some(field) = field_iter.next() {
            match field.type_name().as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; edid_index),
                b"BODT" => collect_one!(BODT, field => fields; bodt_index),
                b"BOD2" => collect_one!(BOD2, field => fields; bod2_index),
                b"RNAM" => collect_one!(RNAM, field => fields; rnam_index),
                b"DNAM" => collect_one!(DNAM, field => fields; dnam_index),
                b"MOD2" => {
                    collect_one_collection!(MOD2, MOD2Collection; field, field_iter => fields; mod2_index)
                }
                b"MOD3" => {
                    collect_one_collection!(MOD3, MOD3Collection; field, field_iter => fields; mod3_index)
                }
                b"MOD4" => {
                    collect_one_collection!(MOD4, MOD4Collection; field, field_iter => fields; mod4_index)
                }
                b"MOD5" => {
                    collect_one_collection!(MOD5, MOD5Collection; field, field_iter => fields; mod5_index)
                }
                b"NAM0" => collect_one!(NAM0, field => fields; nam0_index),
                b"NAM1" => collect_one!(NAM1, field => fields; nam1_index),
                b"NAM2" => collect_one!(NAM2, field => fields; nam2_index),
                b"NAM3" => collect_one!(NAM3, field => fields; nam3_index),
                b"MODL" => {
                    collect_one_collection!(MODL, MODLList; field, field_iter => fields; modl_list_index)
                }
                b"SNDD" => collect_one!(SNDD, field => fields; sndd_index),
                b"ONAM" => collect_one!(ONAM, field => fields; onam_index),
                _ => fields.push(ARMAField::Unknown(field)),
            }
        }

        // Note: UESP says MOD2 is required, but I ran into a entry in Skyrim.esm that did not have it.

        if edid_index.is_none() {
            Err(FromRecordError::ExpectedField(
                edid::EDID::static_type_name(),
            ))
        // TODO: check if it should be one or the other depending on version
        } else if bod2_index.is_none() && bodt_index.is_none() {
            Err(FromRecordError::ExpectedField(BOD2::static_type_name()))
        } else if rnam_index.is_none() {
            Err(FromRecordError::ExpectedField(RNAM::static_type_name()))
        } else if dnam_index.is_none() {
            Err(FromRecordError::ExpectedField(DNAM::static_type_name()))
        } else {
            Ok((
                &[],
                Self {
                    common: record.common,
                    fields,
                },
            ))
        }
    }
}
impl_static_type_named!(ARMARecord<'_>, b"ARMA");
impl DataSize for ARMARecord<'_> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
        4 + // data size len
        self.common.data_size() +
        self.fields.data_size()
    }
}
impl Writable for ARMARecord<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert that size fits within a u32
        (self.fields.data_size() as u32).write_to(w)?;
        self.common.write_to(w)?;
        self.fields.write_to(w)
    }
}

#[derive(Debug, Clone, From)]
pub enum ARMAField<'data> {
    EDID(edid::EDID<'data>),
    BODT(BODT),
    BOD2(BOD2),
    RNAM(RNAM),
    DNAM(DNAM),
    MOD2Collection(MOD2Collection<'data>),
    MOD3Collection(MOD3Collection<'data>),
    MOD4Collection(MOD4Collection<'data>),
    MOD5Collection(MOD5Collection<'data>),
    NAM0(NAM0),
    NAM1(NAM1),
    NAM2(NAM2),
    NAM3(NAM3),
    MODLList(MODLList),
    SNDD(SNDD),
    ONAM(ONAM),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for ARMAField<'data> {
    fn type_name(&self) -> &'data bstr::BStr {
        dispatch_all!(
            ARMAField,
            self,
            [
                EDID,
                BODT,
                BOD2,
                RNAM,
                DNAM,
                MOD2Collection,
                MOD3Collection,
                MOD4Collection,
                MOD5Collection,
                NAM0,
                NAM1,
                NAM2,
                NAM3,
                MODLList,
                SNDD,
                ONAM,
                Unknown
            ],
            x,
            { x.type_name() }
        )
    }
}
impl DataSize for ARMAField<'_> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            ARMAField,
            self,
            [
                EDID,
                BODT,
                BOD2,
                RNAM,
                DNAM,
                MOD2Collection,
                MOD3Collection,
                MOD4Collection,
                MOD5Collection,
                NAM0,
                NAM1,
                NAM2,
                NAM3,
                MODLList,
                SNDD,
                ONAM,
                Unknown
            ],
            x,
            { x.data_size() }
        )
    }
}
impl Writable for ARMAField<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            ARMAField,
            self,
            [
                EDID,
                BODT,
                BOD2,
                RNAM,
                DNAM,
                MOD2Collection,
                MOD3Collection,
                MOD4Collection,
                MOD5Collection,
                NAM0,
                NAM1,
                NAM2,
                NAM3,
                MODLList,
                SNDD,
                ONAM,
                Unknown
            ],
            x,
            { x.write_to(w) }
        )
    }
}

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

make_formid_field!(
    /// -> RACE
    RNAM
);

#[derive(Debug, Clone)]
pub struct DNAM {
    pub male_priority: u8,
    pub female_priority: u8,
    pub unknown1: u32,
    pub detection_sound_value: u8,
    pub unknown2: u8,
    pub weapon_adjust: f32,
}
impl_from_field!(
    DNAM,
    [
        male_priority: u8,
        female_priority: u8,
        unknown1: u32,
        detection_sound_value: u8,
        unknown2: u8,
        weapon_adjust: f32
    ]
);
impl_static_type_named!(DNAM, b"DNAM");
impl_static_data_size!(
    DNAM,
    FIELDH_SIZE + (u8::static_data_size() * 4) + u32::static_data_size() + f32::static_data_size()
);
impl Writable for DNAM {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.male_priority.write_to(w)?;
        self.female_priority.write_to(w)?;
        self.unknown1.write_to(w)?;
        self.detection_sound_value.write_to(w)?;
        self.unknown2.write_to(w)?;
        self.weapon_adjust.write_to(w)
    }
}

make_model_fields!(MOD2; MO2T; MO2S; MOD2Collection);
make_model_fields!(MOD3; MO3T; MO3S; MOD3Collection);
make_model_fields!(MOD4; MO4T; MO4S; MOD4Collection);
make_model_fields!(MOD5; MO5T; MO5S; MOD5Collection);

make_formid_field!(
    /// Base male texture, ->TXST
    NAM0
);
make_formid_field!(
    /// Base female texture, ->TXST
    NAM1
);
make_formid_field!(
    /// Base male first person texture, ->TXST
    NAM2
);
make_formid_field!(
    /// Base female first person texture, ->TXST
    NAM3
);
make_formid_field!(
    /// Race which this is applicable to. ->RACE
    MODL
);
make_formid_field!(
    /// Footstep sound. Mostly creatures.
    SNDD
);
make_formid_field!(
    /// Art object, ->ARTO. Mostly creatures(??)
    ONAM
);

#[derive(Debug, Clone)]
pub struct MODLList {
    races: Vec<MODL>,
}
impl MODLList {
    pub fn collect<'aleph, I>(
        first: MODL,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<Self, FromFieldError<'aleph>>
    where
        I: std::iter::Iterator<Item = GeneralField<'aleph>>,
    {
        let mut races = vec![first];

        loop {
            let (_, modl) = get_field(field_iter, MODL::static_type_name())?;
            match modl {
                Some(modl) => races.push(modl),
                None => break,
            };
        }

        Ok((&[], Self { races }))
    }
}
impl_static_type_named!(MODLList, MODL::static_type_name());
impl DataSize for MODLList {
    fn data_size(&self) -> usize {
        self.races.data_size()
    }
}
impl Writable for MODLList {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.races.write_to(w)
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

    #[test]
    fn test_dnam() {
        let dnam = DNAM {
            male_priority: 0,
            female_priority: 1,
            unknown1: 2,
            detection_sound_value: 3,
            unknown2: 4,
            weapon_adjust: 5.4,
        };
        assert_size_output!(dnam);
    }
}
