use super::fields::common::{CollectField, FieldName, FromField, FromFieldError, GeneralField};
use crate::{
    impl_static_data_size,
    parse::{many, take, PResult, Parse, ParseError},
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use std::{fmt::Debug, io::Write};

pub type Index = usize;
/// Always four characters
pub type RecordName<'data> = &'data BStr;

pub mod formid;
pub mod full_string;
pub mod lstring;
pub mod null_terminated_string;
pub mod version_control_info;
pub mod windows1252_string;

pub use formid::*;
pub use null_terminated_string::*;
pub use version_control_info::*;
pub use windows1252_string::*;

/// collect_one(FieldType, field_variable => field_vector; index_option)
#[macro_export]
macro_rules! collect_one {
    ($s:ty, $field:expr => $fields:expr; $o:expr) => {{
        use $crate::records::fields::common::FromField;
        if $o.is_some() {
            use bstr::ByteSlice;
            return Err($crate::records::common::FromRecordError::DuplicateField(
                stringify!($s).as_bytes().as_bstr(),
            ));
        }

        let (_, result) = <$s>::from_field($field)?;
        $o = Some($fields.len());
        $fields.push(result.into());
    }};
}

/// collect_one_collection!(OpeningFieldType, CollectionType; field_variable, field_iterator => field_vector; index_option);
#[macro_export]
macro_rules! collect_one_collection {
    ($of:ty, $cf:ty; $field:expr, $field_iter:expr => $fields:expr; $o:expr; $collect_name:ident) => {{
        use $crate::records::fields::common::FromField;
        if $o.is_some() {
            use bstr::ByteSlice;
            return Err($crate::records::common::FromRecordError::DuplicateField(
                stringify!($of).as_bytes().as_bstr(),
            ));
        }

        let (_, opening_field) = <$of>::from_field($field)?;
        let (_, collection) = <$cf>::$collect_name(opening_field, &mut $field_iter)?;
        $o = Some($fields.len());
        $fields.push(collection.into());
    }};
    ($of:ty, $cf:ty; $field:expr, $field_iter:expr => $fields:expr; $o:expr) => {{
        use $crate::records::fields::common::CollectField;
        collect_one_collection!($of, $cf; $field, $field_iter => $fields; $o; collect);
    }};
}

/// collect_many(field_type, field_variable => field_vector; index_vector)
#[macro_export]
macro_rules! collect_many {
    ($s:ty, $field:expr => $fields:expr; $o:expr) => {{
        let (_, result) = <$s>::from_field($field)?;
        $o.push($fields.len());
        $fields.push(result.into());
    }};
}

pub type BStrw<'data> = std::borrow::Cow<'data, BStr>;

// ==== Records ====

pub mod record_flag {
    /// Is Master (ESM) file
    /// TES4,
    pub const MASTER: u32 = 0x1;

    /// Is it deleted
    /// All?
    pub const DELETED: u32 = 0x20;

    /// Constant (??)
    /// ??? All, except REFR?
    pub const CONSTANT: u32 = 0x40;
    /// Hidden from local map
    /// (uesp says this (and CONSTANT?) need confirmation and it's related to shields)
    /// REFR,
    pub const REFR: u32 = 0x40;

    /// UESP: 'This will make Skyrim load the .STRINGS, .DLSTRINGS, and .ILSTRING
    /// associated with the mod'.
    /// If not set, lstrings are treated as zstrings
    /// TES4,
    pub const LOCALIZED: u32 = 0x80;

    /// Must update anims
    /// All, except REFR?
    pub const MUST_UPDATE_ANIMS: u32 = 0x100;
    /// Inaccessible
    /// REFR,
    pub const INACCESSIBLE: u32 = 0x100;

    /// Light master (ESL) file Data file.
    /// Fallout 4 only?
    /// Light master files allow more plugins to be loaded than the 8 bit plugin id limit of 255.
    /// TES4,
    pub const LIGHT_MASTER: u32 = 0x200;
    /// Hidden from local map
    /// REFR,
    pub const LOCAL_MAP_HIDDEN: u32 = 0x200;
    /// Starts dead
    /// ACHR,
    pub const STARTS_DEAD: u32 = 0x200;
    /// Motion Blur Casts Shadows
    /// REFR, (??? what, multiple entries for same record?)
    pub const MOTION_BLUR_CAST_SHADOWS: u32 = 0x200;

    /// Quest item
    /// TODO: this probably is not for all (maybe whatever ITEM is?)
    /// All???
    pub const QUEST_ITEM: u32 = 0x400;
    /// Persistent reference
    /// TODO: this is probably not for all (maybe REFR?)
    pub const PERSISTENT_REFERENCE: u32 = 0x400;
    /// Displays in main menu
    /// LSCR,
    pub const DISPLAY_MAIN_MENU: u32 = 0x400;

    /// Initially disabled
    /// All?
    pub const INITIALLY_DISABLED: u32 = 0x800;

    /// Ignored
    /// All?
    pub const IGNORED: u32 = 0x1000;

    // TODO: note: this seems to be missing 0x2000, and 0x4000?

    /// Visible when distant
    /// All? (probably not?)
    pub const VISIBLE_DISTANT: u32 = 0x8000;

    /// Random Animation start
    /// ACTI,
    pub const RANDOM_ANIMATION_START: u32 = 0x10000;

    /// Dangerous
    /// Can't be set without ignore object interaction
    /// ACTI,
    pub const DANGEROUS: u32 = 0x20000;

    /// Off limits
    /// Interior cell??
    pub const OFF_LIMITS: u32 = 0x20000;

    /// Data is compressed
    /// All,
    pub const COMPRESSED: u32 = 0x40000;

    /// Can't wait
    /// TODO: this is probably not for all, probably only cells or something
    /// All?
    pub const NO_WAITING: u32 = 0x80000;

    /// Ignore object interaction
    /// Sets [Dangerous] automatically
    /// ACTI,
    pub const IGNORE_OBJECT_INTERACTION: u32 = 0x10_0000;

    /// TODO: note this seems to be missing 0x20_000 and 0x40_000

    /// Is Marker
    /// All??
    pub const MARKER: u32 = 0x80_000;

    /// Obstacle
    /// ACTI,
    pub const OBSTACLE: u32 = 0x2_0000000;
    /// No AI Acquire
    /// REFR,
    pub const NO_AI_ACQUIRE: u32 = 0x2_0000000;

    /// Navmesh gen: filter
    /// All???
    pub const NAVMESH_GEN_FILTER: u32 = 0x4_000000;

    /// Navmesh gen: bounding box
    /// All???
    pub const NAVMESH_GEN_BOUNDING_BOX: u32 = 0x8_000000;

    // Note: I wouldn't be surprised if these have some navmesh, since this is a somewhat weird hole for navmesh

    /// Must exit to talk
    /// FURN,
    pub const MUST_EXIT_TO_TALK: u32 = 0x10_000000;
    /// Reflected by auto water
    /// REFR,
    pub const REFLECT_AUTO_WATER: u32 = 0x10_000000;

    /// Child can use
    /// FURN, IDLM
    pub const CHILD_CAN_USE: u32 = 0x20_000000;
    /// Don't havok settle
    /// REFR
    pub const NO_HAVOK_SETTLE: u32 = 0x20_000000;

    /// Navmesh gen: ground
    /// All???
    pub const NAVMESH_GEN_GROUND: u32 = 0x40_000000;
    /// No Respawn
    /// REFR
    pub const NO_RESPAWN: u32 = 0x40_000000;

    /// Multi bound
    /// REFR
    pub const MULTIBOUND: u32 = 0x80_000000;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RecordFlags {
    pub flags: u32,
}
impl RecordFlags {
    pub fn new(flags: u32) -> RecordFlags {
        RecordFlags { flags }
    }

    /// Use values from the record_flag module (namespace)
    pub fn is(&self, flag: u32) -> bool {
        (self.flags & flag) != 0
    }
}
impl_static_data_size!(RecordFlags, u32::static_data_size());
impl Writable for RecordFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

/// Information that tends to be common amongst records
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommonRecordInfo {
    pub flags: RecordFlags,
    // TODO: is this a formid?
    /// Record form identifier
    pub id: u32,
    pub version_control_info: VersionControlInfo,
    /// Internal version of record
    pub version: u16,
    /// Values range from 0-15, according to UESP
    pub unknown: u16,
}
impl CommonRecordInfo {
    pub fn new(
        flags: RecordFlags,
        id: u32,
        version_control_info: VersionControlInfo,
        version: u16,
        unknown: u16,
    ) -> CommonRecordInfo {
        CommonRecordInfo {
            flags,
            id,
            version_control_info,
            version,
            unknown,
        }
    }
    /// Extracts the common record information from that record
    pub fn from_field(record: &GeneralRecord<'_>) -> CommonRecordInfo {
        record.common.clone()
    }

    #[cfg(test)]
    pub fn test_default() -> CommonRecordInfo {
        CommonRecordInfo {
            flags: RecordFlags::new(0x9942649a),
            id: 0x420,
            version_control_info: VersionControlInfo {
                day: 9,
                month: 7,
                last_user_id: 42,
                current_user_id: 0,
            },
            version: 4,
            unknown: 1,
        }
    }
}
impl_static_data_size!(
    CommonRecordInfo,
    RecordFlags::static_data_size()
        + u32::static_data_size()
        + VersionControlInfo::static_data_size()
        + u16::static_data_size()
        + u16::static_data_size() // unknown
);
impl Writable for CommonRecordInfo {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)?;
        self.id.write_to(w)?;
        self.version_control_info.write_to(w)?;
        self.version.write_to(w)?;
        self.unknown.write_to(w)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GeneralRecord<'data> {
    pub type_name: RecordName<'data>,
    pub common: CommonRecordInfo,
    /// The fields
    /// Stored in data
    pub fields: Vec<GeneralField<'data>>,
}
impl<'data> Parse<'data> for GeneralRecord<'data> {
    fn parse(data: &'data [u8]) -> PResult<GeneralRecord<'data>> {
        let (data, type_name) = take(data, 4)?;
        let type_name = type_name.as_bstr();

        let (data, record_data_size) = u32::parse(data)?;
        let (data, flags) = u32::parse(data)?;
        let (data, id) = u32::parse(data)?;
        let (data, version_control_info) = VersionControlInfo::parse(data)?;
        // IDEA: Perhaps the version is a four bit integer, and so unkown is simpler the lower bits of it?
        let (data, version) = u16::parse(data)?;
        let (data, unknown) = u16::parse(data)?;

        // TODO: verify it's all been used
        let (data, record_data) = take(data, record_data_size as usize)?;
        let (_, fields) = many(record_data, GeneralField::parse)?;

        Ok((
            data,
            GeneralRecord {
                type_name,
                common: CommonRecordInfo::new(
                    RecordFlags::new(flags),
                    id,
                    version_control_info,
                    version,
                    unknown,
                ),
                fields,
            },
        ))
    }
}
impl<'data> TypeNamed<'data> for GeneralRecord<'data> {
    fn type_name(&self) -> &'data BStr {
        self.type_name
    }
}
impl<'data> DataSize for GeneralRecord<'data> {
    fn data_size(&self) -> usize {
        self.type_name.len() +
            4 + // data_size
            self.common.data_size() +
            self.fields.data_size()
    }
}
impl<'data> Writable for GeneralRecord<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert fields_size is u32
        (self.fields.data_size() as u32).write_to(w)?;
        self.common.write_to(w)?;
        self.fields.write_to(w)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ConversionError<T> {
    InvalidEnumerationValue(T),
}

impl<T> From<ConversionError<T>> for ParseError<'_> {
    fn from(v: ConversionError<T>) -> Self {
        match v {
            ConversionError::InvalidEnumerationValue(_) => ParseError::InvalidEnumerationValue,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FromRecordError<'data> {
    /// An unexpected end of fields
    UnexpectedEnd,
    /// Expected a certain type of field
    ExpectedField(FieldName<'data>),
    /// Expected field, but got
    ExpectedFieldGot {
        expected: FieldName<'data>,
        found: FieldName<'data>,
    },
    /// Found a field, which we didn't expect to find at all
    UnexpectedField(FieldName<'data>),
    /// Found a field which was a duplicate. Note: This doesn't mean they're the same, but we only expected to find one.
    DuplicateField(FieldName<'data>),
    FromField(FromFieldError<'data>),
    ParseError(ParseError<'data>),
}
impl<'data> From<FromFieldError<'data>> for FromRecordError<'data> {
    fn from(err: FromFieldError<'data>) -> Self {
        Self::FromField(err)
    }
}
impl<'data> From<ParseError<'data>> for FromRecordError<'data> {
    fn from(err: ParseError<'data>) -> Self {
        Self::ParseError(err)
    }
}

pub trait FromRecord<'data>: Sized {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>>;
}

pub trait TypeNamed<'aleph>: Sized {
    fn type_name(&self) -> &'aleph BStr;
}
pub trait StaticTypeNamed: Sized {
    fn static_type_name() -> &'static BStr;
}
#[macro_export]
macro_rules! impl_static_type_named {
    ($t:ty, $e:expr) => {
        impl $crate::records::common::StaticTypeNamed for $t {
            fn static_type_name() -> &'static bstr::BStr {
                use bstr::ByteSlice;
                $e.as_bstr()
            }
        }
    };
}
impl<T> TypeNamed<'static> for T
where
    T: StaticTypeNamed,
{
    fn type_name(&self) -> &'static BStr {
        T::static_type_name()
    }
}

pub fn get_field<'aleph, 'bet, I, F>(
    field_iter: &mut std::iter::Peekable<I>,
    expected_field_name: &'bet BStr,
) -> PResult<'aleph, Option<F>, FromFieldError<'aleph>>
where
    I: std::iter::Iterator<Item = GeneralField<'aleph>>,
    F: FromField<'aleph>,
{
    let next_field: Option<&GeneralField<'aleph>> = field_iter.peek();
    // TODO: hardcoding field name = bad
    if next_field
        .map(|x| x.type_name())
        .filter(|x| *x == expected_field_name)
        .is_none()
    {
        Ok((&[], None))
    } else {
        let field: GeneralField<'aleph> = field_iter.next().unwrap();
        assert_eq!(field.type_name(), expected_field_name);
        let (_, field): (_, F) = F::from_field(field)?;
        Ok((&[], Some(field)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldList<'data, T: StaticTypeNamed + DataSize> {
    list: Vec<T>,
    // TODO: is this good a way to do this?
    _marker: std::marker::PhantomData<&'data [u8]>,
}
// Implementation for fields
impl<'data, T> CollectField<'data, T> for FieldList<'data, T>
where
    T: StaticTypeNamed + DataSize + FromField<'data>,
{
    fn collect<I>(
        first: T,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<'data, Self, FromFieldError<'data>>
    where
        I: std::iter::Iterator<Item = GeneralField<'data>>,
    {
        let mut list = vec![first];
        loop {
            let (_, entry) = get_field(field_iter, T::static_type_name())?;
            match entry {
                Some(entry) => list.push(entry),
                None => break,
            };
        }
        Ok((
            &[],
            Self {
                list,
                _marker: std::marker::PhantomData,
            },
        ))
    }
}

impl<'data, T> StaticTypeNamed for FieldList<'data, T>
where
    T: StaticTypeNamed + DataSize,
{
    fn static_type_name() -> &'static BStr {
        T::static_type_name()
    }
}
impl<'data, T> DataSize for FieldList<'data, T>
where
    T: StaticTypeNamed + DataSize,
{
    fn data_size(&self) -> usize {
        self.list.data_size()
    }
}
impl<'data, T> Writable for FieldList<'data, T>
where
    T: Writable + StaticTypeNamed + DataSize,
{
    fn write_to<U>(&self, w: &mut U) -> std::io::Result<()>
    where
        U: Write,
    {
        self.list.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionList<'data, T: StaticTypeNamed + DataSize> {
    list: Vec<T>,
    // TODO: is this a good way to do this
    _marker: std::marker::PhantomData<&'data [u8]>,
}
impl<'data, T, F> CollectField<'data, F> for CollectionList<'data, T>
where
    T: StaticTypeNamed + DataSize + CollectField<'data, F>,
    F: StaticTypeNamed + DataSize + FromField<'data>,
{
    fn collect<I>(
        first: F,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<'data, Self, FromFieldError<'data>>
    where
        I: std::iter::Iterator<Item = GeneralField<'data>>,
    {
        let (_, first) = T::collect(first, field_iter)?;

        let mut list: Vec<T> = vec![first];
        loop {
            let (_, field): (_, Option<F>) = get_field(field_iter, F::static_type_name())?;
            let field = match field {
                Some(field) => field,
                None => break,
            };

            let (_, entry) = T::collect(field, field_iter)?;
            list.push(entry);
        }

        Ok((
            &[],
            Self {
                list,
                _marker: std::marker::PhantomData,
            },
        ))
    }
}
impl<'data, T> StaticTypeNamed for CollectionList<'data, T>
where
    T: StaticTypeNamed + DataSize,
{
    fn static_type_name() -> &'static BStr {
        T::static_type_name()
    }
}
impl<'data, T> DataSize for CollectionList<'data, T>
where
    T: StaticTypeNamed + DataSize,
{
    fn data_size(&self) -> usize {
        self.list.data_size()
    }
}
impl<'data, T> Writable for CollectionList<'data, T>
where
    T: Writable + StaticTypeNamed + DataSize,
{
    fn write_to<W>(&self, w: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        self.list.write_to(w)
    }
}

/// make_field_getter!(editor_id_index, editor_id, editor_id_mut, ARTOField::EDID, edid::EDID<'data>); inside of impl
///   to find a field and panic if it doesn't exist, and makes getters for non-mut and mut version
/// make_field_getter!(optional: model_index, model, model_mut, ARTOField::MODLCollection, modl::MODLCollection<'data>);
///   to find a field and return None if it does not exist. Type will automatically be wrapped in Option
/// requires self.fields property
#[macro_export]
macro_rules! make_field_getter {
    ($index_name:ident, $name:ident, $name_mut:ident, $field_variant:path, $field_type:ty) => {
        pub fn $index_name(&self) -> $crate::records::common::Index {
            self.fields
                .iter()
                .position(|x| matches!(x, $field_variant(_)))
                .expect("ILE: Expected specific field")
        }

        pub fn $name(&self) -> &$field_type {
            match &self.fields[self.$index_name()] {
                $field_variant(x) => x,
                _ => panic!("ILE: Unreachable"),
            }
        }

        pub fn $name_mut(&mut self) -> &mut $field_type {
            let index = self.$index_name();
            match &mut self.fields[index] {
                $field_variant(x) => x,
                _ => panic!("ILE: Unreachable"),
            }
        }
    };

    (optional: $index_name:ident, $name:ident, $name_mut:ident, $field_variant:path, $field_type:ty) => {
        pub fn $index_name(&self) -> Option<$crate::records::common::Index> {
            self.fields
                .iter()
                .position(|x| matches!(x, $field_variant(_)))
        }

        pub fn $name(&self) -> Option<&$field_type> {
            self.$index_name().map(|i| match &self.fields[i] {
                $field_variant(x) => x,
                _ => panic!("ILE: Unreachable"),
            })
        }

        pub fn $name_mut(&mut self) -> Option<&mut $field_type> {
            self.$index_name().map(move |i| match &mut self.fields[i] {
                $field_variant(x) => x,
                _ => panic!("ILE: Unreachable"),
            })
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;
    #[test]
    fn test_record_flags() {
        let r = RecordFlags::new(0x9942649a);
        let data = assert_size_output!(r);
        assert_eq!(data[0], 0x9a);
        assert_eq!(data[1], 0x64);
        assert_eq!(data[2], 0x42);
        assert_eq!(data[3], 0x99);
    }
    #[test]
    fn test_cri() {
        let c = CommonRecordInfo::test_default();
        assert_size_output!(c);
    }
}
