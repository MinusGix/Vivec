use super::{
    common::{FromField, FromFieldError, GeneralField},
    rgbu,
};
use crate::{
    make_single_value_field,
    parse::{count, le_u32, PResult},
    records::common::FormId,
};

make_single_value_field!(
    /// 'Keyword Size'
    [Debug, Copy, Clone, Eq, PartialEq],
    KSIZ,
    /// Number of formids in following KWDA record
    amount,
    u32
);
impl FromField<'_> for KSIZ {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, amount) = le_u32(field.data)?;
        Ok((data, Self { amount }))
    }
}

make_single_value_field!(
    /// 'Keyword array'
    [Debug, Clone],
    KWDA,
    /// FormId array that points to keywords (?)
    keywords,
    Vec<FormId>
);
//impl FromField<'_> for KWDA {
impl KWDA {
    pub fn from_field(field: GeneralField<'_>, amount: u32) -> PResult<Self, FromFieldError> {
        let (data, keywords) = count(field.data, FormId::parse, amount as usize)?;
        Ok((data, Self { keywords }))
    }
}
