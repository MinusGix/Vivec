use super::common::{FromFieldError, GeneralField};
use crate::{
    impl_from_field, impl_static_type_named, make_single_value_field,
    parse::{count, PResult, Parse},
    records::common::{FormId, StaticTypeNamed, TypeNamed},
    util::{DataSize, Writable},
};

make_single_value_field!(
    /// 'Keyword Size'
    [Debug, Copy, Clone, Eq, PartialEq],
    KSIZ,
    /// Number of formids in following KWDA record
    amount,
    u32
);
impl_from_field!(KSIZ, [amount: u32]);

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

/// KWDACollection
#[derive(Debug, Clone)]
pub struct KWDACollection {
    // Note: we don't keep the KSIZ instance in here, since it can be generated from the KWDA instance :]
    keywords: KWDA,
}
impl KWDACollection {
    pub fn collect<'data, I>(
        ksiz: KSIZ,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<Self, FromFieldError<'data>>
    where
        I: std::iter::Iterator<Item = GeneralField<'data>>,
    {
        let next_field = field_iter.peek();
        if next_field
            .map(|x| x.type_name())
            .filter(|x| *x == KWDA::static_type_name())
            .is_none()
        {
            Err(FromFieldError::ExpectedSpecificField(
                KWDA::static_type_name(),
            ))
        } else {
            let field = field_iter.next().unwrap();
            let (_, field) = KWDA::from_field(field, ksiz.amount)?;
            Ok((&[], KWDACollection { keywords: field }))
        }
    }
    pub fn create_ksiz(&self) -> KSIZ {
        // TODO: check that it fits
        KSIZ {
            amount: self.keywords.keywords.len() as u32,
        }
    }
}
impl_static_type_named!(KWDACollection, KSIZ::static_type_name());
impl DataSize for KWDACollection {
    fn data_size(&self) -> usize {
        self.create_ksiz().data_size() + self.keywords.data_size()
    }
}
impl Writable for KWDACollection {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.create_ksiz().write_to(w)?;
        self.keywords.write_to(w)
    }
}
