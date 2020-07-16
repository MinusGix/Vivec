use crate::{
    dispatch_all,
    util::{DataSize, Writable},
};
use derive_more::From;

pub mod aact;
pub mod acti;
pub mod addn;
pub mod alch;
pub mod ammo;
pub mod anio;

pub mod common;

#[derive(Debug, Clone, From)]
pub enum Group<'data> {
    AACT(aact::AACTGroup<'data>),
    ACTI(acti::ACTIGroup<'data>),
    ADDN(addn::ADDNGroup<'data>),
    ALCH(alch::ALCHGroup<'data>),
    AMMO(ammo::AMMOGroup<'data>),
    ANIO(anio::ANIOGroup<'data>),
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
        dispatch_all!(
            Group,
            self,
            [AACT, ACTI, ADDN, ALCH, AMMO, ANIO, Unknown, UnknownTop],
            x,
            { x.data_size() }
        )
    }
}
impl<'data> Writable for Group<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        dispatch_all!(
            Group,
            self,
            [AACT, ACTI, ADDN, ALCH, AMMO, ANIO, Unknown, UnknownTop],
            x,
            { x.write_to(w) }
        )
    }
}
