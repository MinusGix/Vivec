use super::{
    common::{
        CommonRecordInfo, FromRecord, FromRecordError, GeneralRecord, Index, NullTerminatedString,
        StaticTypeNamed, TypeNamed,
    },
    fields::{
        common::{FromField, FromFieldError, GeneralField},
        edid, modl,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_static_type_named,
    make_single_value_field,
    parse::PResult,
    util::{DataSize, Writable},
};
use bstr::BStr;
use derive_more::From;

#[derive(Debug, Clone)]
pub struct ANIORecord<'data> {
    pub common: CommonRecordInfo,
    pub editor_id_index: Index,
    pub model_collection_index: Index,
    pub unload_event_index: Option<Index>,

    pub fields: Vec<ANIOField<'data>>,
}
impl<'data> FromRecord<'data> for ANIORecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut editor_id_index = None;
        let mut model_collection_index = None;
        let mut unload_event_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();

        while let Some(field) = field_iter.next() {
            match field.type_name().as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; editor_id_index),
                b"MODL" => {
                    collect_one_collection!(modl::MODL, modl::MODLCollection; field, field_iter => fields; model_collection_index)
                }
                b"BNAM" => collect_one!(BNAM, field => fields; unload_event_index),
                _ => fields.push(ANIOField::Unknown(field)),
            }
        }

        let editor_id_index = editor_id_index
            .ok_or_else(|| FromRecordError::ExpectedField(edid::EDID::static_type_name()))?;
        let model_collection_index = model_collection_index.ok_or_else(|| {
            FromRecordError::ExpectedField(modl::MODLCollection::static_type_name())
        })?;

        Ok((
            &[],
            Self {
                common: record.common,
                editor_id_index,
                model_collection_index,
                unload_event_index,
                fields,
            },
        ))
    }
}
impl_static_type_named!(ANIORecord<'_>, b"ANIO");
impl DataSize for ANIORecord<'_> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
            4 + // data size
            self.common.data_size() +
            self.fields.data_size()
    }
}

impl Writable for ANIORecord<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert that size fits within a u32
        (self.fields.data_size() as u32).write_to(w)?;
        self.common.write_to(w)?;
        self.fields.write_to(w)?;
        Ok(())
    }
}

#[derive(Debug, Clone, From)]
pub enum ANIOField<'data> {
    EDID(edid::EDID<'data>),
    MODLCollection(modl::MODLCollection<'data>),
    BNAM(BNAM<'data>),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for ANIOField<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(ANIOField, self, [EDID, MODLCollection, BNAM, Unknown], x, {
            x.type_name()
        })
    }
}
impl DataSize for ANIOField<'_> {
    fn data_size(&self) -> usize {
        dispatch_all!(ANIOField, self, [EDID, MODLCollection, BNAM, Unknown], x, {
            x.data_size()
        })
    }
}
impl Writable for ANIOField<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        dispatch_all!(ANIOField, self, [EDID, MODLCollection, BNAM, Unknown], x, {
            x.write_to(w)
        })
    }
}

make_single_value_field!(
    [Debug, Clone, Eq, PartialEq],
    BNAM,
    /// Almost always 'AnimObjectUnequip'
    unload_event,
    NullTerminatedString,
    'data
);
impl<'data> FromField<'data> for BNAM<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, unload_event) = NullTerminatedString::parse(field.data)?;
        // TODO: check that is all.
        Ok((data, Self { unload_event }))
    }
}
