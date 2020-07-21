use super::{
    common::{
        CommonRecordInfo, FromRecord, FromRecordError, GeneralRecord, Index, StaticTypeNamed,
        TypeNamed,
    },
    fields::{
        common::{item, object, GeneralField},
        dest, edid, modl, obnd, vmad,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_static_type_named,
    parse::PResult,
    util::{DataSize, Writable},
};
use bstr::BStr;
use derive_more::From;
use std::io::Write;

/// Apparatus
/// No use in TES5, but were used in Morrowind and Oblivion.
#[derive(Debug, Clone, PartialEq)]
pub struct APPARecord<'data> {
    pub common: CommonRecordInfo,
    /// EDID
    pub editor_id_index: Index,
    /// VMAD
    pub script_index: Option<Index>,
    /// OBND
    pub object_bounds_index: Index,
    /// FULL
    pub name_index: Index,
    /// MODLCollection
    pub model_collection_index: Option<Index>,
    /// ICON
    pub image_index: Option<Index>,
    /// MICO
    pub message_image_index: Option<Index>,
    /// DESTCollection
    pub destruction_collection_index: Option<Index>,
    /// YNAM
    pub pickup_sound_index: Option<Index>,
    /// ZNAM
    pub drop_sound_index: Option<Index>,
    /// QUAL
    pub quality_index: Index,
    /// DESC
    pub description_index: Index,
    /// DATA
    pub data_index: Index,

    pub fields: Vec<APPAField<'data>>,
}
impl<'data> FromRecord<'data> for APPARecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut editor_id_index = None;
        let mut script_index = None;
        let mut object_bounds_index = None;
        let mut name_index = None;
        let mut model_collection_index = None;
        let mut image_index = None;
        let mut message_image_index = None;
        let mut destruction_collection_index = None;
        let mut pickup_sound_index = None;
        let mut drop_sound_index = None;
        let mut quality_index = None;
        let mut description_index = None;
        let mut data_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();

        while let Some(field) = field_iter.next() {
            match field.type_name().as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; editor_id_index),
                b"VMAD" => {
                    collect_one!(vmad::VMAD<'data, vmad::NoFragments>, field => fields; script_index)
                }
                b"OBND" => collect_one!(obnd::OBND, field => fields; object_bounds_index),
                b"FULL" => collect_one!(object::FULL, field => fields; name_index),
                b"MODL" => {
                    collect_one_collection!(modl::MODL, modl::MODLCollection; field, field_iter => fields; model_collection_index)
                }
                b"ICON" => collect_one!(item::ICON, field => fields; image_index),
                b"MICO" => collect_one!(item::MICO, field => fields; message_image_index),
                b"DEST" => {
                    collect_one_collection!(dest::DEST, dest::DESTCollection; field, field_iter => fields; destruction_collection_index)
                }
                b"YNAM" => collect_one!(item::YNAM, field => fields; pickup_sound_index),
                b"ZNAM" => collect_one!(item::ZNAM, field => fields; drop_sound_index),
                b"QUAL" => collect_one!(item::QUAL, field => fields; quality_index),
                b"DESC" => collect_one!(item::DESC, field => fields; description_index),
                b"DATA" => collect_one!(item::DATA, field => fields; data_index),
                _ => fields.push(field.into()),
            }
        }

        let editor_id_index = editor_id_index
            .ok_or_else(|| FromRecordError::ExpectedField(edid::EDID::static_type_name()))?;
        let object_bounds_index = object_bounds_index
            .ok_or_else(|| FromRecordError::ExpectedField(obnd::OBND::static_type_name()))?;
        let name_index = name_index
            .ok_or_else(|| FromRecordError::ExpectedField(object::FULL::static_type_name()))?;
        let quality_index = quality_index
            .ok_or_else(|| FromRecordError::ExpectedField(item::QUAL::static_type_name()))?;
        let description_index = description_index
            .ok_or_else(|| FromRecordError::ExpectedField(item::DESC::static_type_name()))?;
        let data_index = data_index
            .ok_or_else(|| FromRecordError::ExpectedField(item::DATA::static_type_name()))?;

        Ok((
            &[],
            Self {
                common: record.common,
                editor_id_index,
                script_index,
                object_bounds_index,
                name_index,
                model_collection_index,
                image_index,
                message_image_index,
                destruction_collection_index,
                pickup_sound_index,
                drop_sound_index,
                quality_index,
                description_index,
                data_index,
                fields,
            },
        ))
    }
}
impl_static_type_named!(APPARecord<'_>, b"APPA");
impl Writable for APPARecord<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert that size fits within a u32
        (self.fields.data_size() as u32).write_to(w)?;
        self.common.write_to(w)?;
        self.fields.write_to(w)?;
        Ok(())
    }
}
impl DataSize for APPARecord<'_> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
            4 + // data size
            self.common.data_size() +
            self.fields.data_size()
    }
}

#[derive(Debug, Clone, PartialEq, From)]
pub enum APPAField<'data> {
    EDID(edid::EDID<'data>),
    VMAD(vmad::VMAD<'data, vmad::NoFragments>),
    OBND(obnd::OBND),
    FULL(object::FULL),
    MODLCollection(modl::MODLCollection<'data>),
    ICON(item::ICON<'data>),
    MICO(item::MICO<'data>),
    DESTCollection(dest::DESTCollection<'data>),
    YNAM(item::YNAM),
    ZNAM(item::ZNAM),
    QUAL(item::QUAL),
    DESC(item::DESC),
    DATA(item::DATA),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for APPAField<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(
            APPAField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                MODLCollection,
                ICON,
                MICO,
                DESTCollection,
                YNAM,
                ZNAM,
                QUAL,
                DESC,
                DATA,
                Unknown
            ],
            x,
            { x.type_name() }
        )
    }
}
impl DataSize for APPAField<'_> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            APPAField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                MODLCollection,
                ICON,
                MICO,
                DESTCollection,
                YNAM,
                ZNAM,
                QUAL,
                DESC,
                DATA,
                Unknown
            ],
            x,
            { x.data_size() }
        )
    }
}
impl Writable for APPAField<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            APPAField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                MODLCollection,
                ICON,
                MICO,
                DESTCollection,
                YNAM,
                ZNAM,
                QUAL,
                DESC,
                DATA,
                Unknown
            ],
            x,
            { x.write_to(w) }
        )
    }
}
