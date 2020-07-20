use super::{
    common::{self, CommonRecordInfo, GeneralRecord, Index},
    fields::{
        common::{rgbu, GeneralField},
        edid,
    },
};
use crate::{
    collect_one, dispatch_all, impl_from_field, impl_static_type_named, make_single_value_field,
    parse::PResult,
    util::{DataSize, Writable},
};
use bstr::BStr;
use common::{FromRecord, FromRecordError, TypeNamed};
use derive_more::From;
use std::io::Write;

/// Holds information about actions
/// Note: There can be an Empty-Record of this.
#[derive(Debug, Clone)]
pub struct AACTRecord<'data> {
    pub common: CommonRecordInfo,
    /// Editor id. EDID
    pub action_name_index: Option<Index>,
    /// RGB colors. CNAM (RGBU)
    pub rgb_index: Option<Index>,

    pub fields: Vec<AACTField<'data>>,
}
impl<'data> FromRecord<'data> for AACTRecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<AACTRecord<'data>, FromRecordError> {
        let mut edid_index = None;
        let mut cname_index = None;
        let mut fields = Vec::new();
        for field in record.fields {
            match field.type_name.as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; edid_index),
                b"CNAM" => collect_one!(CNAM, field => fields; cname_index),
                _ => fields.push(AACTField::Unknown(field)),
            }
        }

        Ok((
            &[],
            AACTRecord {
                common: record.common,
                action_name_index: edid_index,
                rgb_index: cname_index,
                fields,
            },
        ))
    }
}
impl_static_type_named!(AACTRecord<'_>, b"AACT");
impl<'data> DataSize for AACTRecord<'data> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
        4 + // data size len
        self.common.data_size() +
        self.fields.data_size()
    }
}
impl<'data> Writable for AACTRecord<'data> {
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
pub enum AACTField<'data> {
    EDID(edid::EDID<'data>),
    CNAM(CNAM),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for AACTField<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(AACTField, self, [EDID, CNAM, Unknown], x, { x.type_name() })
    }
}
impl<'data> DataSize for AACTField<'data> {
    fn data_size(&self) -> usize {
        match self {
            AACTField::EDID(x) => x.data_size(),
            AACTField::CNAM(x) => x.data_size(),
            AACTField::Unknown(x) => x.data_size(),
        }
    }
}
impl<'data> Writable for AACTField<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        match self {
            AACTField::EDID(x) => x.write_to(w),
            AACTField::CNAM(x) => x.write_to(w),
            AACTField::Unknown(x) => x.write_to(w),
        }
    }
}

make_single_value_field!([Debug, Copy, Clone, Eq, PartialEq], CNAM, color, rgbu::RGBU);
impl_from_field!(CNAM, [color: rgbu::RGBU]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;

    #[test]
    fn aactrecord_check() {
        let record = AACTRecord {
            common: CommonRecordInfo::test_default(),
            action_name_index: None,
            rgb_index: None,
            fields: vec![],
        };

        println!("{:#?}", record);

        assert_size_output!(record);
    }
}
