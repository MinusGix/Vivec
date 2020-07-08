use super::{
    common::{self, CommonRecordInfo, GeneralRecord, Index},
    fields::{
        common::{FromField, GeneralField},
        edid, rgbu,
    },
};
use crate::{
    collect_one, dispatch_all, make_single_value_field,
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use common::{FromRecord, TypeNamed};
use derive_more::From;
use nom::IResult;
use std::io::Write;

/// Holds information about actions
/// Note: There can be an Empty-Record of this.
#[derive(Debug, Clone)]
pub struct AACTRecord<'data> {
    common_info: CommonRecordInfo,
    /// Editor id. EDID
    action_name_index: Option<Index>,
    /// RGB colors. CNAM (RGBU)
    rgb_index: Option<Index>,

    fields: Vec<AACTField<'data>>,
}
impl<'data> TypeNamed<'static> for AACTRecord<'data> {
    fn type_name(&self) -> &'static BStr {
        b"AACT".as_bstr()
    }
}
impl<'data> FromRecord<'data> for AACTRecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> IResult<&[u8], AACTRecord<'data>> {
        let mut edid_index = None;
        let mut cname_index = None;
        let mut fields = Vec::new();
        for field in record.fields {
            match field.type_name.as_ref() {
                b"EDID" => collect_one!(EDID, field => fields; edid_index),
                b"CNAM" => collect_one!(CNAM, field => fields; cname_index),
                _ => fields.push(AACTField::Unknown(field)),
            }
        }

        Ok((
            &[],
            AACTRecord {
                common_info: record.common_info,
                action_name_index: edid_index,
                rgb_index: cname_index,
                fields,
            },
        ))
    }
}
impl<'data> AACTRecord<'data> {
    pub fn fields_size(&self) -> usize {
        self.fields.iter().fold(0, |acc, x| acc + x.data_size())
    }
}
impl<'data> DataSize for AACTRecord<'data> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
        4 + // data size len
        self.common_info.data_size() +
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
        (self.fields_size() as u32).write_to(w)?;
        self.common_info.write_to(w)?;
        for field in self.fields.iter() {
            field.write_to(w)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, From)]
pub enum AACTField<'data> {
    EDID(EDID<'data>),
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

type EDID<'data> = edid::EDID<'data>;

make_single_value_field!([Debug, Copy, Clone, Eq, PartialEq], CNAM, color, rgbu::RGBU);
impl FromField<'_> for CNAM {
    fn from_field(field: GeneralField<'_>) -> IResult<&[u8], Self> {
        let (data, color) = rgbu::RGBU::parse(field.data)?;
        Ok((data, Self { color }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;

    #[test]
    fn aactrecord_check() {
        let record = AACTRecord {
            common_info: CommonRecordInfo::test_default(),
            action_name_index: None,
            rgb_index: None,
            fields: vec![],
        };

        println!("{:#?}", record);

        assert_size_output!(record);
    }
}
