use super::{
    common::{
        lstring::LString, CommonRecordInfo, FormId, FromRecord, FromRecordError, GeneralRecord,
        Index, StaticTypeNamed, TypeNamed,
    },
    fields::{
        common::{FromField, FromFieldError, GeneralField},
        dest, edid, full, kwda, modl, obnd, rgbu, vmad,
    },
};
use crate::{
    collect_one, dispatch_all, make_formid_field, make_single_value_field,
    parse::{count, le_u16, le_u32, PResult},
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use derive_more::From;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct ACTIRecord<'data> {
    pub common: CommonRecordInfo,

    /// EDID
    pub editor_id_index: Index,
    /// VMAD
    pub script_index: Option<Index>,
    /// OBND
    pub object_bounds_index: Index,
    /// FULL
    pub ingame_name_index: Option<Index>,
    /// MODLCollection
    pub model_collection_index: Option<Index>,
    /// DESTCollection
    pub destruction_collection_index: Option<Index>,
    /// KWDACollection
    pub keyword_data_index: Option<Index>,
    /// PNAM
    pub marker_color_index: Option<Index>,
    /// SNAM
    pub looping_sound_index: Option<Index>,
    /// VNAM
    pub activation_sound_index: Option<Index>,
    /// WNAM
    pub water_index: Option<Index>,
    /// RNAM
    pub verb_index: Option<Index>,
    /// FNAM
    pub flags_index: Option<Index>,
    /// KNAM
    pub interaction_keyword_index: Option<Index>,

    pub fields: Vec<ACTIField<'data>>,
}
impl<'data> FromRecord<'data> for ACTIRecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError> {
        let mut edid_index = None;
        let mut vmad_index = None;
        let mut obnd_index = None;
        let mut full_index = None;
        let mut modl_collection_index = None;
        let mut dest_collection_index = None;
        let mut keyword_data_index = None;
        let mut pnam_index = None;
        let mut snam_index = None;
        let mut vnam_index = None;
        let mut wnam_index = None;
        let mut rnam_index = None;
        let mut fnam_index = None;
        let mut knam_index = None;

        let mut fields = Vec::new();

        let mut field_iter = record.fields.into_iter().peekable();
        while let Some(field) = field_iter.next() {
            match field.type_name.as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; edid_index),
                b"VMAD" => {
                    collect_one!(vmad::VMAD<'data, vmad::NoFragments>, field => fields; vmad_index)
                }
                b"OBND" => collect_one!(obnd::OBND, field => fields; obnd_index),
                b"FULL" => collect_one!(full::FULL, field => fields; full_index),
                b"MODL" => {
                    let (_, modl) = modl::MODL::from_field(field)?;
                    let (_, col) = modl::MODLCollection::collect(modl, &mut field_iter)?;
                    modl_collection_index = Some(fields.len());
                    fields.push(ACTIField::MODLCollection(col));
                }
                b"DEST" => {
                    let (_, dest) = dest::DEST::from_field(field)?;
                    let (_, col) = dest::DESTCollection::collect(dest, &mut field_iter)?;
                    dest_collection_index = Some(fields.len());
                    fields.push(ACTIField::DESTCollection(col));
                }
                b"KSIZ" => {
                    let (_, ksiz) = kwda::KSIZ::from_field(field)?;
                    let (_, col) = kwda::KWDACollection::collect(ksiz, &mut field_iter)?;
                    keyword_data_index = Some(fields.len());
                    fields.push(ACTIField::KWDACollection(col));
                }
                b"PNAM" => collect_one!(PNAM, field => fields; pnam_index),
                b"SNAM" => collect_one!(SNAM, field => fields; snam_index),
                b"VNAM" => collect_one!(VNAM, field => fields; vnam_index),
                b"WNAM" => collect_one!(WNAM, field => fields; wnam_index),
                b"RNAM" => collect_one!(RNAM, field => fields; rnam_index),
                b"FNAM" => collect_one!(FNAM, field => fields; fnam_index),
                b"KNAM" => collect_one!(KNAM, field => fields; knam_index),
                _ => {
                    println!("Unknown field name: {}", field.type_name());
                    fields.push(ACTIField::Unknown(field));
                }
            }
        }

        let edid_index = edid_index.unwrap();
        let obnd_index = obnd_index.unwrap();

        Ok((
            &[],
            ACTIRecord {
                common: record.common,

                editor_id_index: edid_index,
                script_index: vmad_index,
                object_bounds_index: obnd_index,
                ingame_name_index: full_index,
                model_collection_index: modl_collection_index,
                destruction_collection_index: dest_collection_index,
                keyword_data_index,
                marker_color_index: pnam_index,
                looping_sound_index: snam_index,
                activation_sound_index: vnam_index,
                water_index: wnam_index,
                verb_index: rnam_index,
                flags_index: fnam_index,
                interaction_keyword_index: knam_index,

                fields,
            },
        ))
    }
}
impl<'data> StaticTypeNamed<'static> for ACTIRecord<'data> {
    fn static_type_name() -> &'static BStr {
        b"ACTI".as_bstr()
    }
}
impl<'data> DataSize for ACTIRecord<'data> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
            4 + // data len
            self.common.data_size() +
            self.fields.data_size()
    }
}
impl<'data> Writable for ACTIRecord<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert size fits within
        (self.fields.data_size() as u32).write_to(w)?;
        self.common.write_to(w)?;
        self.fields.write_to(w)
    }
}

#[derive(Debug, Clone, From)]
pub enum ACTIField<'data> {
    EDID(edid::EDID<'data>),
    VMAD(vmad::VMAD<'data, vmad::NoFragments>),
    OBND(obnd::OBND),
    FULL(full::FULL),
    MODLCollection(modl::MODLCollection<'data>),
    DESTCollection(dest::DESTCollection<'data>),
    KWDACollection(kwda::KWDACollection),
    PNAM(PNAM),
    SNAM(SNAM),
    VNAM(VNAM),
    WNAM(WNAM),
    RNAM(RNAM),
    FNAM(FNAM),
    KNAM(KNAM),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for ACTIField<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(
            ACTIField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                MODLCollection,
                DESTCollection,
                KWDACollection,
                PNAM,
                SNAM,
                VNAM,
                WNAM,
                RNAM,
                FNAM,
                KNAM,
                Unknown
            ],
            x,
            { x.type_name() }
        )
    }
}
impl<'data> DataSize for ACTIField<'data> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            ACTIField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                MODLCollection,
                DESTCollection,
                KWDACollection,
                PNAM,
                SNAM,
                VNAM,
                WNAM,
                RNAM,
                FNAM,
                KNAM,
                Unknown
            ],
            x,
            { x.data_size() }
        )
    }
}
impl<'data> Writable for ACTIField<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            ACTIField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                MODLCollection,
                DESTCollection,
                KWDACollection,
                PNAM,
                SNAM,
                VNAM,
                WNAM,
                RNAM,
                FNAM,
                KNAM,
                Unknown
            ],
            x,
            { x.write_to(w) }
        )
    }
}

make_single_value_field!([Debug, Copy, Clone, Eq, PartialEq], PNAM, color, rgbu::RGBU);
impl FromField<'_> for PNAM {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, color) = rgbu::RGBU::parse(field.data)?;
        Ok((data, Self { color }))
    }
}

make_formid_field!(
    /// ->SNDR uesp: 'nirnroot has the wow-wow sound here' (quality comment, I approve)
    SNAM
);

make_formid_field!(
    /// ->SNDR. when activated
    VNAM
);

make_formid_field!(
    /// ->WATR rare
    WNAM
);

make_single_value_field!(
    [Debug, Clone, Eq, PartialEq],
    RNAM,
    /// Verb string. Activate text override. Such as "Mine" or "Place" instead of "Activate"
    verb,
    LString
);
impl FromField<'_> for RNAM {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, verb) = LString::parse(field.data)?;
        Ok((data, Self { verb }))
    }
}

make_single_value_field!(
    /// Flags
    [Debug, Copy, Clone, Eq, PartialEq],
    FNAM,
    /// 0b1 = No displacement (related to water type)
    /// 0b10 = ignored by sandbox
    flags,
    u16
);
impl FromField<'_> for FNAM {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, flags) = le_u16(field.data)?;
        Ok((data, Self { flags }))
    }
}

make_formid_field!(
    /// ->KWYD form id for interaction purposes (??? What)
    KNAM
);
