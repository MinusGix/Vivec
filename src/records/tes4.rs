use super::{
    common::{
        get_field, CommonRecordInfo, FormId, FromRecord, FromRecordError, GeneralRecord, Index,
        NullTerminatedString, StaticTypeNamed, TypeNamed,
    },
    fields::common::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE},
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_from_field, impl_static_data_size,
    impl_static_type_named, make_field_getter, make_single_value_field,
    parse::{many, PResult, Parse},
    util::{fmt_data, DataSize, Writable},
};
use bstr::BStr;
use derive_more::From;
use std::io::Write;

/// Header record for mod file
#[derive(Debug, Clone, PartialEq)]
pub struct TES4Record<'data> {
    pub common: CommonRecordInfo,
    pub fields: Vec<TES4Field<'data>>,
}
impl<'data> TES4Record<'data> {
    make_field_getter!(header_index, header, header_mut, TES4Field::HEDR, HEDR);

    make_field_getter!(
        optional: author_index,
        author,
        author_mut,
        TES4Field::CNAM,
        CNAM<'data>
    );

    make_field_getter!(
        optional: description_index,
        description,
        description_mut,
        TES4Field::SNAM,
        SNAM<'data>
    );

    // TODO: should this just exist always, but with zero entries? would have to decide where to put it..
    make_field_getter!(
        optional: masters_index,
        masters,
        masters_mut,
        TES4Field::MasterCollection,
        MasterCollection<'data>
    );

    make_field_getter!(
        optional: overrides_index,
        overrides,
        overrides_mut,
        TES4Field::ONAM,
        ONAM
    );

    make_field_getter!(
        optional: internal_version_index,
        internal_version,
        internal_version_mut,
        TES4Field::INTV,
        INTV
    );
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

        if hedr_index.is_none() {
            Err(FromRecordError::ExpectedField(HEDR::static_type_name()))
        } else {
            Ok((
                &[],
                TES4Record {
                    common: record.common.clone(),
                    fields,
                },
            ))
        }
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

#[derive(Debug, Clone, PartialEq, From)]
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
impl_from_field!(HEDR, [version: f32, record_count: u32, next_object_id: u32]);
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
impl_from_field!(CNAM, 'data, [author: NullTerminatedString]);

make_single_value_field!([Debug, Clone, Eq, PartialEq], SNAM,
    /// max-size: 512 bytes (including null!)
    description,
    NullTerminatedString,
    'data
);
impl_from_field!(SNAM, 'data, [description: NullTerminatedString]);

/// Holds a MAST,DATA pair
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
impl_from_field!(MAST, 'data, [filename: NullTerminatedString]);

make_single_value_field!([Debug, Clone, Eq, PartialEq], DATA, value, u64);
impl_from_field!(DATA, [value: u64]);

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
impl_from_field!(INTV, [value: u32]);

make_single_value_field!([Debug, Clone, Eq, PartialEq], INCC, value, u32);
impl_from_field!(INCC, [value: u32]);

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
            fields: vec![TES4Field::HEDR(HEDR {
                version: 1.7,
                record_count: 2,
                next_object_id: 120,
            })],
        };
        assert_size_output!(tes4);
    }
}
