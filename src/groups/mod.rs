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
pub mod appa;
pub mod arma;
pub mod armo;
pub mod arto;
pub mod aspc;
pub mod astp;
pub mod avif;
pub mod book;

pub mod common;

#[derive(Debug, Clone, PartialEq, From)]
pub enum Group<'data> {
    AACT(aact::AACTGroup<'data>),
    ACTI(acti::ACTIGroup<'data>),
    ADDN(addn::ADDNGroup<'data>),
    ALCH(alch::ALCHGroup<'data>),
    AMMO(ammo::AMMOGroup<'data>),
    ANIO(anio::ANIOGroup<'data>),
    APPA(appa::APPAGroup<'data>),
    ARMA(arma::ARMAGroup<'data>),
    ARMO(armo::ARMOGroup<'data>),
    ARTO(arto::ARTOGroup<'data>),
    ASPC(aspc::ASPCGroup<'data>),
    ASTP(astp::ASTPGroup<'data>),
    AVIF(avif::AVIFGroup<'data>),
    BOOK(book::BOOKGroup<'data>),
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
            [
                AACT, ACTI, ADDN, ALCH, AMMO, ANIO, APPA, ARMA, ARMO, ARTO, ASPC, ASTP, AVIF, BOOK,
                Unknown, UnknownTop
            ],
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
            [
                AACT, ACTI, ADDN, ALCH, AMMO, ANIO, APPA, ARMA, ARMO, ARTO, ASPC, ASTP, AVIF, BOOK,
                Unknown, UnknownTop
            ],
            x,
            { x.write_to(w) }
        )
    }
}
