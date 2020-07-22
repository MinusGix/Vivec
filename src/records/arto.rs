use super::{
    common::{
        CommonRecordInfo, ConversionError, FromRecord, FromRecordError, GeneralRecord,
        StaticTypeNamed, TypeNamed,
    },
    fields::{
        common::{write_field_header, GeneralField, FIELDH_SIZE},
        edid, modl, obnd,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_from_field, impl_static_data_size,
    impl_static_type_named, make_field_getter,
    parse::{PResult, Parse},
    util::{DataSize, Writable},
};
use derive_more::From;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, PartialEq)]
pub struct ARTORecord<'data> {
    pub common: CommonRecordInfo,
    pub fields: Vec<ARTOField<'data>>,
}
impl<'data> ARTORecord<'data> {
    make_field_getter!(
        editor_id_index,
        editor_id,
        editor_id_mut,
        ARTOField::EDID,
        edid::EDID<'data>
    );

    make_field_getter!(
        object_bounds_index,
        object_bounds,
        object_bounds_mut,
        ARTOField::OBND,
        obnd::OBND
    );

    make_field_getter!(
        optional: model_index,
        model,
        model_mut,
        ARTOField::MODLCollection,
        modl::MODLCollection<'data>
    );

    make_field_getter!(
        art_type_index,
        art_type,
        art_type_mut,
        ARTOField::DNAM,
        DNAM
    );
}
impl<'data> FromRecord<'data> for ARTORecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut edid_index = None;
        let mut obnd_index = None;
        let mut modl_collection_index = None;
        let mut dnam_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();

        while let Some(field) = field_iter.next() {
            match field.type_name().as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; edid_index),
                b"OBND" => collect_one!(obnd::OBND, field => fields; obnd_index),
                b"MODL" => {
                    collect_one_collection!(modl::MODL, modl::MODLCollection; field, field_iter => fields; modl_collection_index)
                }
                b"DNAM" => collect_one!(DNAM, field => fields; dnam_index),
                _ => fields.push(field.into()),
            }
        }

        if edid_index.is_none() {
            Err(FromRecordError::ExpectedField(
                edid::EDID::static_type_name(),
            ))
        } else if obnd_index.is_none() {
            Err(FromRecordError::ExpectedField(
                obnd::OBND::static_type_name(),
            ))
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
impl_static_type_named!(ARTORecord<'_>, b"ARTO");
impl DataSize for ARTORecord<'_> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
			4 + // data len
            self.common.data_size() +
            self.fields.data_size()
    }
}
impl Writable for ARTORecord<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert size fits within
        (self.fields.data_size() as u32).write_to(w)?;
        self.common.write_to(w)?;
        self.fields.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq, From)]
pub enum ARTOField<'data> {
    EDID(edid::EDID<'data>),
    OBND(obnd::OBND),
    MODLCollection(modl::MODLCollection<'data>),
    DNAM(DNAM),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for ARTOField<'data> {
    fn type_name(&self) -> &'data bstr::BStr {
        dispatch_all!(
            ARTOField,
            self,
            [EDID, OBND, MODLCollection, DNAM, Unknown],
            x,
            { x.type_name() }
        )
    }
}
impl DataSize for ARTOField<'_> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            ARTOField,
            self,
            [EDID, OBND, MODLCollection, DNAM, Unknown],
            x,
            { x.data_size() }
        )
    }
}
impl Writable for ARTOField<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        dispatch_all!(
            ARTOField,
            self,
            [EDID, OBND, MODLCollection, DNAM, Unknown],
            x,
            { x.write_to(w) }
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DNAM {
    pub art_type: ArtType,
}
impl_from_field!(DNAM, [art_type: ArtType]);
impl_static_type_named!(DNAM, b"DNAM");
impl_static_data_size!(DNAM, FIELDH_SIZE + ArtType::static_data_size());
impl Writable for DNAM {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        write_field_header(self, w)?;
        self.art_type.write_to(w)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum ArtType {
    MagicCasting = 0,
    MagicHitEffect = 1,
    EnchantmentEffect = 2,
}
impl ArtType {
    pub fn code(&self) -> u32 {
        *self as u32
    }
}
impl Parse<'_> for ArtType {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, value) = u32::parse(data)?;
        let art_type = value.try_into()?;
        Ok((data, art_type))
    }
}
impl_static_data_size!(ArtType, u32::static_data_size());
impl Writable for ArtType {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.code().write_to(w)
    }
}
impl TryFrom<u32> for ArtType {
    type Error = ConversionError<u32>;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => ArtType::MagicCasting,
            1 => ArtType::MagicHitEffect,
            2 => ArtType::EnchantmentEffect,
            _ => return Err(ConversionError::InvalidEnumerationValue(value)),
        })
    }
}
