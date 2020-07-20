use crate::{impl_from_field, make_single_value_field, records::common::lstring::LString};

make_single_value_field!(
    /// Ingame name
    [Debug, Clone, Eq, PartialEq],
    FULL,
    name,
    LString
);
impl_from_field!(FULL, [name: LString]);
