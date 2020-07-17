use super::{
    common::{
        get_field, CommonRecordInfo, FormId, FromRecord, FromRecordError, GeneralRecord, Index,
        NullTerminatedString, StaticTypeNamed, TypeNamed,
    },
    fields::common::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE},
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_static_data_size,
    impl_static_type_named, make_single_value_field,
    parse::{many, PResult, Parse},
    util::{fmt_data, DataSize, Writable},
};
use bstr::BStr;
use derive_more::From;
use std::io::Write;

#[derive(Debug, Clone)]
/// Header record for mod file
pub struct TES4Record<'data> {
    common: CommonRecordInfo,

    // The usize fields are indices into other_fields
    /// HEDR
    pub header_index: Index,
    /// CNAM
    pub author_index: Option<Index>,
    /// SNAM
    pub description_index: Option<Index>,
    /// MasterCollection
    pub master_collection_index: Option<Index>,
    /// ONAM
    pub overrides_index: Option<Index>,
    /// unknown if it's required, or even if it's name means internal version
    pub internal_version_index: Option<Index>,
    pub unknown_incc_index: Option<Index>,
    /// Note: any modificatons of this will have to be matched in the other fields!
    pub fields: Vec<TES4Field<'data>>,
}
impl<'data> TES4Record<'data> {
    pub fn header(&self) -> &HEDR {
        if let TES4Field::HEDR(hedr) = &self.fields[self.header_index] {
            hedr
        } else {
            panic!("ILE: Expected entry at indice to be a HEDR instance");
        }
    }

    pub fn author(&self) -> Option<&CNAM> {
        let index = self.author_index?;
        if let TES4Field::CNAM(cnam) = &self.fields[index] {
            Some(cnam)
        } else {
            panic!("ILE: Expected entry at indice to be a CNAM instance");
        }
    }

    pub fn description(&self) -> Option<&SNAM> {
        let index = self.description_index?;
        if let TES4Field::SNAM(snam) = &self.fields[index] {
            Some(snam)
        } else {
            panic!("ILE: Expected entry at indice to be a SNAM instance");
        }
    }

    pub fn master_collection(&self, mast_data_index: usize) -> Option<&MasterCollection> {
        let index = self.master_collection_index?;
        if let TES4Field::MasterCollection(col) = &self.fields[index] {
            Some(col)
        } else {
            panic!("ILE: Expected entry at indice to be a MasterCollection instance");
        }
    }

    pub fn overrides(&self) -> Option<&ONAM> {
        let index = self.overrides_index?;
        if let TES4Field::ONAM(onam) = &self.fields[index] {
            Some(onam)
        } else {
            panic!("ILE: Expected entry at indice to be an ONAM instance");
        }
    }

    pub fn internal_version(&self) -> Option<&INTV> {
        let index = self.internal_version_index?;
        if let TES4Field::INTV(intv) = &self.fields[index] {
            Some(intv)
        } else {
            panic!("ILE: Expected entry at indice to be an INTV instance");
        }
    }

    pub fn unknown_incc(&self) -> Option<&INCC> {
        let index = self.unknown_incc_index?;
        if let TES4Field::INCC(incc) = &self.fields[index] {
            Some(incc)
        } else {
            panic!("ILE: Expected entry at indice to be an INCC instance");
        }
    }
}
impl<'data> FromRecord<'data> for TES4Record<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<TES4Record, FromRecordError<'data>> {
        let mut fields = Vec::new();
        let mut hedr_index: Option<Index> = None;
        let mut cnam_index: Option<Index> = None;
        let mut snam_index: Option<Index> = None;
        let mut mast_collection_index: Option<Index> = None;
        let mut onam_index: Option<Index> = None;
        let mut intv_index: Option<Index> = None;
        let mut incc_index: Option<Index> = None;

        // TODO: These need to check if it's used up all the space.
        let mut field_iter = record.fields.into_iter().peekable();
        while let Some(field) = field_iter.next() {
            match field.type_name.as_ref() {
                b"HEDR" => collect_one!(HEDR, field => fields; hedr_index),
                b"CNAM" => collect_one!(CNAM, field => fields; cnam_index),
                b"SNAM" => collect_one!(SNAM, field => fields; snam_index),
                b"ONAM" => collect_one!(ONAM, field => fields; onam_index),
                b"INTV" => collect_one!(INTV, field => fields; intv_index),
                b"INCC" => collect_one!(INCC, field => fields; incc_index),
                b"MAST" => {
                    collect_one_collection!(MAST, MasterCollection; field, field_iter => fields; mast_collection_index)
                }
                b"DATA" => {
                    // TODO: continue, just add this to the list
                    panic!("[WARN] Found DATA field in TES4 that did not have a MAST before it.");
                }
                _ => fields.push(TES4Field::Unknown(field)),
            }
        }

        let hedr_index =
            hedr_index.ok_or_else(|| FromRecordError::ExpectedField(HEDR::static_type_name()))?;

        Ok((
            &[],
            TES4Record {
                common: record.common.clone(),

                header_index: hedr_index,
                author_index: cnam_index,
                description_index: snam_index,
                master_collection_index: mast_collection_index,
                overrides_index: onam_index,
                internal_version_index: intv_index,
                unknown_incc_index: incc_index,
                fields,
            },
        ))
    }
}
impl_static_type_named!(TES4Record<'_>, b"TES4");
impl<'data> DataSize for TES4Record<'data> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
            4 + // data size
            self.common.data_size() +
            self.fields.data_size()
    }
}
impl<'data> Writable for TES4Record<'data> {
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
pub enum TES4Field<'data> {
    HEDR(HEDR),
    CNAM(CNAM<'data>),
    SNAM(SNAM<'data>),
    ONAM(ONAM),
    INTV(INTV),
    INCC(INCC),
    MasterCollection(MasterCollection<'data>),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for TES4Field<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(
            TES4Field,
            self,
            [
                HEDR,
                CNAM,
                SNAM,
                ONAM,
                INTV,
                INCC,
                MasterCollection,
                Unknown
            ],
            x,
            { x.type_name() }
        )
    }
}
impl<'data> DataSize for TES4Field<'data> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            TES4Field,
            self,
            [
                HEDR,
                CNAM,
                SNAM,
                ONAM,
                INTV,
                INCC,
                MasterCollection,
                Unknown
            ],
            x,
            { x.data_size() }
        )
    }
}
impl<'data> Writable for TES4Field<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            TES4Field,
            self,
            [
                HEDR,
                CNAM,
                SNAM,
                ONAM,
                INTV,
                INCC,
                MasterCollection,
                Unknown
            ],
            x,
            { x.write_to(w) }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct HEDR {
    /// 0.94 in most files, 1.7 in recent versions of Update.esm
    pub version: f32,
    /// Numbers of records and groups. (not including TES4 record)
    pub record_count: u32,
    /// Next available object id
    pub next_object_id: u32,
}
impl FromField<'_> for HEDR {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, version) = f32::parse(field.data)?;
        let (data, record_count) = u32::parse(data)?;
        let (data, next_object_id) = u32::parse(data)?;
        // TODO: assure that it's at the end.

        Ok((
            data,
            HEDR {
                version,
                record_count,
                next_object_id,
            },
        ))
    }
}
impl_static_type_named!(HEDR, b"HEDR");
impl_static_data_size!(
    HEDR,
    FIELDH_SIZE +
    f32::static_data_size() + // version
    u32::static_data_size() + // record count
    u32::static_data_size() // next object id
);
impl Writable for HEDR {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.version.write_to(w)?;
        self.record_count.write_to(w)?;
        self.next_object_id.write_to(w)
    }
}

make_single_value_field!([Debug, Clone, Eq, PartialEq], CNAM,
    /// max-size: 512 bytes (including null!)
    author,
    NullTerminatedString,
    'data
);
impl<'data> FromField<'data> for CNAM<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, author) = NullTerminatedString::parse(field.data)?;
        Ok((data, CNAM { author }))
    }
}

make_single_value_field!([Debug, Clone, Eq, PartialEq], SNAM,
    /// max-size: 512 bytes (including null!)
    description,
    NullTerminatedString,
    'data
);
impl<'data> FromField<'data> for SNAM<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, description) = NullTerminatedString::parse(field.data)?;
        Ok((data, SNAM { description }))
    }
}

/// Holds a MAST,DATA pair
#[derive(Debug, Clone)]
pub struct MASTCollection<'data> {
    master: MAST<'data>,
    data: DATA,
}
impl<'data> MASTCollection<'data> {
    pub fn collect<I>(
        master: MAST<'data>,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<'data, Self, FromFieldError<'data>>
    where
        I: std::iter::Iterator<Item = GeneralField<'data>>,
    {
        let (_, data) = get_field(field_iter, DATA::static_type_name())?;
        let data =
            data.ok_or_else(|| FromFieldError::ExpectedSpecificField(DATA::static_type_name()))?;

        Ok((&[], Self { master, data }))
    }
}
impl_static_type_named!(MASTCollection<'_>, MAST::static_type_name());
impl DataSize for MASTCollection<'_> {
    fn data_size(&self) -> usize {
        self.master.data_size() + self.data.data_size()
    }
}
impl Writable for MASTCollection<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.master.write_to(w)?;
        self.data.write_to(w)
    }
}

#[derive(Debug, Clone)]
pub struct MasterCollection<'data> {
    masters: Vec<MASTCollection<'data>>,
}
impl<'data> MasterCollection<'data> {
    pub fn collect<I>(
        master: MAST<'data>,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<'data, Self, FromFieldError<'data>>
    where
        I: std::iter::Iterator<Item = GeneralField<'data>>,
    {
        let mut masters = Vec::new();
        let (_, col) = MASTCollection::collect(master, field_iter)?;
        masters.push(col);

        loop {
            let (_, master) = get_field(field_iter, MAST::static_type_name())?;
            match master {
                Some(master) => {
                    let (_, col) = MASTCollection::collect(master, field_iter)?;
                    masters.push(col);
                }
                _ => break,
            }
        }

        Ok((&[], Self { masters }))
    }
}
impl_static_type_named!(MasterCollection<'_>, MASTCollection::static_type_name());
impl DataSize for MasterCollection<'_> {
    fn data_size(&self) -> usize {
        self.masters.data_size()
    }
}
impl Writable for MasterCollection<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.masters.write_to(w)
    }
}

make_single_value_field!(
    [Debug, Clone, Eq, PartialEq],
    MAST,
    /// Master filename
    filename,
    NullTerminatedString,
    'data
);
impl<'data> FromField<'data> for MAST<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, filename) = NullTerminatedString::parse(field.data)?;
        Ok((data, MAST { filename }))
    }
}

make_single_value_field!([Debug, Clone, Eq, PartialEq], DATA, value, u64);
impl FromField<'_> for DATA {
    fn from_field(field: GeneralField<'_>) -> PResult<DATA, FromFieldError> {
        // TODO: verify that was all
        let (data, value) = u64::parse(field.data)?;
        Ok((data, DATA { value }))
    }
}

make_single_value_field!(
    [Clone, Eq, PartialEq],
    ONAM,
    /// Overidden forms
    /// Only appears in ESM flagged files which override their masters' cell children
    /// Will only list formids of cell children: (ACHR, LAND, NVM, PGR, PHZD, REFR)
    overrides,
    Vec<FormId>
);
impl FromField<'_> for ONAM {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (field_data, overrides) = many(field.data, FormId::parse)?;
        Ok((field_data, ONAM { overrides }))
    }
}
impl std::fmt::Debug for ONAM {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = fmt.debug_struct("ONAM");
        fmt_data(&mut res, "overrides", self.overrides.as_slice(), 10);
        res.finish()
    }
}

make_single_value_field!([Debug, Clone, Eq, PartialEq], INTV, value, u32);
impl FromField<'_> for INTV {
    fn from_field(field: GeneralField<'_>) -> PResult<INTV, FromFieldError> {
        // TODO: verify that was all
        let (data, value) = u32::parse(field.data)?;
        Ok((data, INTV { value }))
    }
}

make_single_value_field!([Debug, Clone, Eq, PartialEq], INCC, value, u32);
impl FromField<'_> for INCC {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        // TODO: verify that was all
        let (data, value) = u32::parse(field.data)?;
        Ok((data, INCC { value }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;
    #[test]
    fn test_hedr() {
        let hedr = HEDR {
            version: 1.7,
            record_count: 2,
            next_object_id: 120,
        };
        assert_size_output!(hedr);
    }

    #[test]
    fn test_onam() {
        let onam = ONAM {
            overrides: vec![FormId::new(0x4292), FormId::new(0x664a)],
        };
        assert_size_output!(onam);
    }

    #[test]
    fn test_tes4() {
        // TODO: it'd be better to have more fields active to be a better test
        let tes4 = TES4Record {
            common: CommonRecordInfo::test_default(),
            header_index: 0,
            author_index: None,
            description_index: None,
            master_collection_index: None,
            overrides_index: None,
            internal_version_index: None,
            unknown_incc_index: None,
            fields: vec![TES4Field::HEDR(HEDR {
                version: 1.7,
                record_count: 2,
                next_object_id: 120,
            })],
        };
        assert_size_output!(tes4);
    }
}
