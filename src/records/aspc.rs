use super::{
    common::{CommonRecordInfo, FromRecord, FromRecordError, GeneralRecord, TypeNamed},
    fields::{common::GeneralField, edid, obnd},
};
use crate::{
    collect_one, dispatch_all, impl_static_type_named, make_formid_field,
    parse::PResult,
    util::{DataSize, Writable},
};
use derive_more::From;

#[derive(Debug, Clone)]
pub struct ASPCRecord<'data> {
    pub common: CommonRecordInfo,
    pub fields: Vec<ASPCField<'data>>,
}
impl<'data> FromRecord<'data> for ASPCRecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut edid_index = None;
        let mut obnd_index = None;
        let mut snam_index = None;
        let mut rdat_index = None;
        let mut bnam_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();

        while let Some(field) = field_iter.next() {
            match field.type_name().as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; edid_index),
                b"OBND" => collect_one!(obnd::OBND, field => fields; obnd_index),
                b"SNAM" => collect_one!(SNAM, field => fields; snam_index),
                b"RDAT" => collect_one!(RDAT, field => fields; rdat_index),
                b"BNAM" => collect_one!(BNAM, field => fields; bnam_index),
                _ => fields.push(field.into()),
            }
        }

        Ok((
            &[],
            Self {
                common: record.common,
                fields,
            },
        ))
    }
}
impl_static_type_named!(ASPCRecord<'_>, b"ASPC");
impl DataSize for ASPCRecord<'_> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
			4 + // data len
            self.common.data_size() +
            self.fields.data_size()
    }
}
impl Writable for ASPCRecord<'_> {
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

#[derive(Debug, Clone, From)]
pub enum ASPCField<'data> {
    EDID(edid::EDID<'data>),
    OBND(obnd::OBND),
    SNAM(SNAM),
    RDAT(RDAT),
    BNAM(BNAM),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for ASPCField<'data> {
    fn type_name(&self) -> &'data bstr::BStr {
        dispatch_all!(
            ASPCField,
            self,
            [EDID, OBND, SNAM, RDAT, BNAM, Unknown],
            x,
            { x.type_name() }
        )
    }
}
impl DataSize for ASPCField<'_> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            ASPCField,
            self,
            [EDID, OBND, SNAM, RDAT, BNAM, Unknown],
            x,
            { x.data_size() }
        )
    }
}
impl Writable for ASPCField<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        dispatch_all!(
            ASPCField,
            self,
            [EDID, OBND, SNAM, RDAT, BNAM, Unknown],
            x,
            { x.write_to(w) }
        )
    }
}

make_formid_field!(
    /// Ambient sound ->SNDR
    SNAM
);
make_formid_field!(
    /// Region sound ->REGN
    RDAT
);
make_formid_field!(
    /// Reverb for acoustic space ->REVB
    BNAM
);
