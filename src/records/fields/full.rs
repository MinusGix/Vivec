use super::common::{FromField, FromFieldError, GeneralField};
use crate::{make_single_value_field, parse::PResult, records::common::lstring::LString};

make_single_value_field!(
    /// Ingame name
    [Debug, Clone, Eq, PartialEq],
    FULL,
    name,
    LString
);
impl FromField<'_> for FULL {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, name) = LString::parse(field.data)?;
        Ok((data, Self { name }))
    }
}
