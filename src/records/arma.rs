use super::{
    common::{
        get_field, CommonRecordInfo, ConversionError, FieldList, FromRecord, FromRecordError,
        GeneralRecord, StaticTypeNamed, TypeNamed,
    },
    fields::{
        common::{item, write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE},
        edid,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_from_field, impl_static_data_size,
    impl_static_type_named, make_formid_field, make_model_fields, make_single_value_field,
    parse::{take, PResult, Parse, ParseError},
    util::{DataSize, StaticDataSize, Writable},
};
use bstr::BStr;
use derive_more::From;
use std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    io::Write,
    marker::PhantomData,
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
                b"BODT" => collect_one!(item::BODT, field => fields; bodt_index),
                b"BOD2" => collect_one!(item::BOD2, field => fields; bod2_index),
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
            Err(FromRecordError::ExpectedField(
                item::BOD2::static_type_name(),
            ))
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
    BODT(item::BODT),
    BOD2(item::BOD2),
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
    MODLList(MODLList<'data>),
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

type MODLList<'unused> = FieldList<'unused, MODL>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;

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
