use super::{
    common::{
        lstring::LString, CommonRecordInfo, FromRecord, FromRecordError, GeneralRecord,
        StaticTypeNamed, TypeNamed,
    },
    fields::{
        common::{item, object, write_field_header, GeneralField, FIELDH_SIZE},
        dest, edid, kwda, modl, obnd, vmad,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_from_field, impl_static_data_size,
    impl_static_type_named, make_formid_field, make_single_value_field,
    parse::{PResult, Parse},
    util::{DataSize, Writable},
};
use derive_more::From;
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct BOOKRecord<'data> {
    pub common: CommonRecordInfo,
    pub fields: Vec<BOOKField<'data>>,
}
impl<'data> FromRecord<'data> for BOOKRecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut edid_index = None;
        let mut vmad_index = None;
        let mut obnd_index = None;
        let mut full_index = None;
        let mut modl_collection_index = None;
        let mut icon_index = None;
        let mut mico_index = None;
        let mut desc_index = None;
        let mut dest_collection_index = None;
        let mut ynam_index = None;
        let mut znam_index = None;
        let mut kwda_collection_index = None;
        let mut data_index = None;
        let mut inam_index = None;
        let mut cnam_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();

        while let Some(field) = field_iter.next() {
            match field.type_name().as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; edid_index),
                b"VMAD" => {
                    collect_one!(vmad::VMAD<'data, vmad::NoFragments>, field => fields; vmad_index)
                }
                b"OBND" => collect_one!(obnd::OBND, field => fields; obnd_index),
                b"FULL" => collect_one!(object::FULL, field => fields; full_index),
                b"MODL" => {
                    collect_one_collection!(modl::MODL, modl::MODLCollection; field, field_iter => fields; modl_collection_index)
                }
                b"ICON" => collect_one!(item::ICON, field => fields; icon_index),
                b"MICO" => collect_one!(item::MICO, field => fields; mico_index),
                b"DESC" => collect_one!(item::DESC, field => fields; desc_index),
                b"DEST" => {
                    collect_one_collection!(dest::DEST, dest::DESTCollection; field, field_iter => fields; dest_collection_index)
                }
                b"YNAM" => collect_one!(item::YNAM, field => fields; ynam_index),
                b"ZNAM" => collect_one!(item::ZNAM, field => fields; znam_index),
                b"KSIZ" => {
                    collect_one_collection!(kwda::KSIZ, kwda::KWDACollection; field, field_iter => fields; kwda_collection_index)
                }
                b"DATA" => collect_one!(DATA, field => fields; data_index),
                b"INAM" => collect_one!(INAM, field => fields; inam_index),
                b"CNAM" => collect_one!(CNAM, field => fields; cnam_index),
                _ => fields.push(field.into()),
            }
        }

        if edid_index.is_none() {
            Err(FromRecordError::ExpectedField(
                edid::EDID::static_type_name(),
            ))
        } else if obnd_index.is_none() {
            Err(FromRecordError::ExpectedField(
                obnd::OBND::static_type_name(),
            ))
        } else if desc_index.is_none() {
            Err(FromRecordError::ExpectedField(
                item::DESC::static_type_name(),
            ))
        } else if data_index.is_none() {
            Err(FromRecordError::ExpectedField(DATA::static_type_name()))
        } else {
            Ok((
                &[],
                Self {
                    common: record.common,
                    fields,
                },
            ))
        }
    }
}
impl_static_type_named!(BOOKRecord<'_>, b"BOOK");
impl DataSize for BOOKRecord<'_> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
			4 + // data len
            self.common.data_size() +
            self.fields.data_size()
    }
}
impl Writable for BOOKRecord<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert size fits within
        (self.fields.data_size() as u32).write_to(w)?;
        self.common.write_to(w)?;
        self.fields.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq, From)]
pub enum BOOKField<'data> {
    EDID(edid::EDID<'data>),
    VMAD(vmad::VMAD<'data, vmad::NoFragments>),
    OBND(obnd::OBND),
    FULL(object::FULL),
    MODLCollection(modl::MODLCollection<'data>),
    ICON(item::ICON<'data>),
    MICO(item::MICO<'data>),
    DESC(item::DESC),
    DESTCollection(dest::DESTCollection<'data>),
    YNAM(item::YNAM),
    ZNAM(item::ZNAM),
    KWDACollection(kwda::KWDACollection),
    DATA(DATA),
    INAM(INAM),
    CNAM(CNAM),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for BOOKField<'data> {
    fn type_name(&self) -> &'data bstr::BStr {
        dispatch_all!(
            BOOKField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                MODLCollection,
                ICON,
                MICO,
                DESC,
                DESTCollection,
                YNAM,
                ZNAM,
                KWDACollection,
                DATA,
                INAM,
                CNAM,
                Unknown
            ],
            x,
            { x.type_name() }
        )
    }
}
impl DataSize for BOOKField<'_> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            BOOKField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                MODLCollection,
                ICON,
                MICO,
                DESC,
                DESTCollection,
                YNAM,
                ZNAM,
                KWDACollection,
                DATA,
                INAM,
                CNAM,
                Unknown
            ],
            x,
            { x.data_size() }
        )
    }
}
impl Writable for BOOKField<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        dispatch_all!(
            BOOKField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                MODLCollection,
                ICON,
                MICO,
                DESC,
                DESTCollection,
                YNAM,
                ZNAM,
                KWDACollection,
                DATA,
                INAM,
                CNAM,
                Unknown
            ],
            x,
            { x.write_to(w) }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DATA {
    flags: DATAFlags,
    /// always 0 since SSE
    /// 0 - book/tome
    /// 255 - note/scroll (uesp signifies that it's a guess)
    /// Note: I'm curious what Fallouts various book-esque things are like, or are they just not classified as books?
    b_type: u8,
    /// Potentially padding
    unknown: u16,
    // TODO: flags and teaches are partially intertwined depending on flags value..
    teaches: u32,
    value: item::Gold,
    weight: item::Weight,
}
impl_from_field!(
    DATA,
    [
        flags: DATAFlags,
        b_type: u8,
        unknown: u16,
        teaches: u32,
        value: item::Gold,
        weight: item::Weight
    ]
);
impl_static_type_named!(DATA, b"DATA");
impl_static_data_size!(
    DATA,
    FIELDH_SIZE
        + DATAFlags::static_data_size()
        + u8::static_data_size()
        + u16::static_data_size()
        + u32::static_data_size()
        + item::Gold::static_data_size()
        + item::Weight::static_data_size()
);
impl Writable for DATA {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.flags.write_to(w)?;
        self.b_type.write_to(w)?;
        self.unknown.write_to(w)?;
        self.teaches.write_to(w)?;
        self.value.write_to(w)?;
        self.weight.write_to(w)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DATAFlags {
    /// 0b0001: Teaches Skill
    /// 0b0010: Can't be taken
    /// 0b0100: Teaches spell
    /// 0b1000: Read. UESP guesses that this is what is set in the save file if the book has been read.
    pub flags: u8,
}
impl Parse<'_> for DATAFlags {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = u8::parse(data)?;
        Ok((data, Self { flags }))
    }
}
impl_static_data_size!(DATAFlags, u8::static_data_size());
impl Writable for DATAFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

make_formid_field!(
    /// ->STAT (?)
    INAM
);

make_single_value_field!(
    /// Description.
    [Debug, Copy, Clone, Eq, PartialEq],
    CNAM,
    description,
    LString
);
impl_from_field!(CNAM, [description: LString]);
