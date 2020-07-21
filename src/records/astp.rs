use super::{
    common::{
        CommonRecordInfo, FromRecord, FromRecordError, GeneralRecord, NullTerminatedString,
        StaticTypeNamed, TypeNamed,
    },
    fields::{common::GeneralField, edid},
};
use crate::{
    collect_one, dispatch_all, impl_from_field, impl_static_data_size, impl_static_type_named,
    make_single_value_field,
    parse::{PResult, Parse},
    util::{DataSize, Writable},
};
use derive_more::From;
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct ASTPRecord<'data> {
    pub common: CommonRecordInfo,
    pub fields: Vec<ASTPField<'data>>,
}
impl<'data> FromRecord<'data> for ASTPRecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut edid_index = None;
        let mut mprt_index = None;
        let mut fprt_index = None;
        let mut fcht_index = None;
        let mut mcht_index = None;
        let mut data_index = None;

        let mut fields = Vec::new();

        for field in record.fields {
            match field.type_name().as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; edid_index),
                b"MPRT" => collect_one!(MPRT, field => fields; mprt_index),
                b"FPRT" => collect_one!(FPRT, field => fields; fprt_index),
                b"FCHT" => collect_one!(FCHT, field => fields; fcht_index),
                b"MCHT" => collect_one!(MCHT, field => fields; mcht_index),
                b"DATA" => collect_one!(DATA, field => fields; data_index),
                _ => fields.push(field.into()),
            }
        }

        if edid_index.is_none() {
            Err(FromRecordError::ExpectedField(
                edid::EDID::static_type_name(),
            ))
        } else if mprt_index.is_none() {
            Err(FromRecordError::ExpectedField(MPRT::static_type_name()))
        } else if fprt_index.is_none() {
            Err(FromRecordError::ExpectedField(FPRT::static_type_name()))
        } else if data_index.is_none() {
            Err(FromRecordError::ExpectedField(DATA::static_type_name()))
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
impl_static_type_named!(ASTPRecord<'_>, b"ASTP");
impl DataSize for ASTPRecord<'_> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
        4 + // data size len
        self.common.data_size() +
        self.fields.data_size()
    }
}
impl Writable for ASTPRecord<'_> {
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

#[derive(Debug, Clone, PartialEq, From)]
pub enum ASTPField<'data> {
    EDID(edid::EDID<'data>),
    MPRT(MPRT<'data>),
    FPRT(FPRT<'data>),
    FCHT(FCHT<'data>),
    MCHT(MCHT<'data>),
    DATA(DATA),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for ASTPField<'data> {
    fn type_name(&self) -> &'data bstr::BStr {
        dispatch_all!(
            ASTPField,
            self,
            [EDID, MPRT, FPRT, FCHT, MCHT, DATA, Unknown],
            x,
            { x.type_name() }
        )
    }
}
impl DataSize for ASTPField<'_> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            ASTPField,
            self,
            [EDID, MPRT, FPRT, FCHT, MCHT, DATA, Unknown],
            x,
            { x.data_size() }
        )
    }
}
impl Writable for ASTPField<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            ASTPField,
            self,
            [EDID, MPRT, FPRT, FCHT, MCHT, DATA, Unknown],
            x,
            { x.write_to(w) }
        )
    }
}

make_single_value_field!(
    /// Male parent label
    [Debug, Clone, Eq, PartialEq],
    MPRT,
    label,
    NullTerminatedString,
    'data
);
impl_from_field!(MPRT, 'data, [label: NullTerminatedString<'data>]);

make_single_value_field!(
    /// Female parent label
    [Debug, Clone, Eq, PartialEq],
    FPRT,
    label,
    NullTerminatedString,
    'data
);
impl_from_field!(FPRT, 'data, [label: NullTerminatedString<'data>]);

make_single_value_field!(
    /// Female child label
    [Debug, Clone, Eq, PartialEq],
    FCHT,
    label,
    NullTerminatedString,
    'data
);
impl_from_field!(FCHT, 'data, [label: NullTerminatedString<'data>]);

make_single_value_field!(
    /// Male child label
    [Debug, Clone, Eq, PartialEq],
    MCHT,
    label,
    NullTerminatedString,
    'data
);
impl_from_field!(MCHT, 'data, [label: NullTerminatedString<'data>]);

make_single_value_field!([Debug, Copy, Clone, Eq, PartialEq], DATA, flags, DATAFlags);
impl_from_field!(DATA, [flags: DATAFlags]);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DATAFlags {
    pub flags: u32,
}
impl Parse<'_> for DATAFlags {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = u32::parse(data)?;
        Ok((data, Self { flags }))
    }
}
impl_static_data_size!(DATAFlags, u32::static_data_size());
impl Writable for DATAFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}
