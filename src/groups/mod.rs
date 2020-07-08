use crate::{
    dispatch_all,
    util::{DataSize, Writable},
};
use derive_more::From;

pub mod aact;
pub mod acti;

pub mod common;

#[derive(Debug, Clone, From)]
pub enum Group<'data> {
    AACT(aact::AACTGroup<'data>),
    ACTI(acti::ACTIGroup<'data>),
    Unknown(common::GeneralGroup<'data>),
    UnknownTop(common::TopGroup<'data>),
}
/*impl<'data> TypeNamed<'data> for Group<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(Group, self, [AACT, Unknown], x, { x.type_name() })
    }
}*/
impl<'data> DataSize for Group<'data> {
    fn data_size(&self) -> usize {
        dispatch_all!(Group, self, [AACT, ACTI, Unknown, UnknownTop], x, {
            x.data_size()
        })
    }
}
impl<'data> Writable for Group<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        dispatch_all!(Group, self, [AACT, ACTI, Unknown, UnknownTop], x, {
            x.write_to(w)
        })
    }
}
