use crate::{
    dispatch_all,
    util::{DataSize, Writable},
};
use bstr::BStr;
use common::TypeNamed;
use derive_more::From;

pub mod aact;
pub mod achr;
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

pub mod tes4;

pub mod fields;

pub mod common;

#[derive(Debug, Clone, PartialEq, From)]
pub enum Record<'data> {
    TES4(tes4::TES4Record<'data>),
    AACT(aact::AACTRecord<'data>),
    ACTI(acti::ACTIRecord<'data>),
    ADDN(addn::ADDNRecord<'data>),
    ACHR(achr::ACHRRecord<'data>),
    ALCH(alch::ALCHRecord<'data>),
    AMMO(ammo::AMMORecord<'data>),
    ANIO(anio::ANIORecord<'data>),
    APPA(appa::APPARecord<'data>),
    ARMA(arma::ARMARecord<'data>),
    ARMO(armo::ARMORecord<'data>),
    ARTO(arto::ARTORecord<'data>),
    ASPC(aspc::ASPCRecord<'data>),
    Unknown(common::GeneralRecord<'data>),
}
impl<'data> TypeNamed<'data> for Record<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(
            Record,
            self,
            [
                TES4, AACT, ACTI, ADDN, ACHR, ALCH, AMMO, ANIO, APPA, ARMA, ARMO, ARTO, ASPC,
                Unknown
            ],
            x,
            { x.type_name() }
        )
    }
}
impl<'data> DataSize for Record<'data> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            Record,
            self,
            [
                TES4, AACT, ACTI, ADDN, ACHR, ALCH, AMMO, ANIO, APPA, ARMA, ARMO, ARTO, ASPC,
                Unknown
            ],
            x,
            { x.data_size() }
        )
    }
}
impl<'data> Writable for Record<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        dispatch_all!(
            Record,
            self,
            [
                TES4, AACT, ACTI, ADDN, ACHR, ALCH, AMMO, ANIO, APPA, ARMA, ARMO, ARTO, ASPC,
                Unknown
            ],
            x,
            { x.write_to(w) }
        )
    }
}
