use super::fields::common::{FromField, GeneralField};
use crate::util::{byte, DataSize, StaticDataSize, Writable};
use bstr::{BStr, ByteSlice};
use nom::{
    bytes::complete::take,
    multi::many0,
    number::complete::{le_u16, le_u32},
    IResult,
};
use std::io::Write;

pub type Index = usize;
/// Always four characters
pub type RecordName<'data> = &'data BStr;

mod formid;
pub mod lstring;
mod null_terminated_string;
mod version_control_info;
mod windows1252_string;

pub use formid::*;
pub use null_terminated_string::*;
pub use version_control_info::*;
pub use windows1252_string::*;

/// collect_one(FieldType, field_variable => field_vector; index_option)
#[macro_export]
macro_rules! collect_one {
    ($s:ty, $field:expr => $fields:expr; $o:expr) => {{
        if $o.is_some() {
            panic!("Unexpected [name] field when already found [name] in TES4 record!");
        }

        let (_, result) = <$s>::from_field($field)?;
        $o = Some($fields.len());
        $fields.push(result.into());
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
impl Writable for RecordFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}
impl StaticDataSize for RecordFlags {
    fn static_data_size() -> usize {
        u32::static_data_size()
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
        record.common_info.clone()
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
impl StaticDataSize for CommonRecordInfo {
    fn static_data_size() -> usize {
        RecordFlags::static_data_size()
            + u32::static_data_size() // id
            + VersionControlInfo::static_data_size()
            + u16::static_data_size() // version
            + u16::static_data_size() // unknown
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GeneralRecord<'data> {
    pub type_name: RecordName<'data>,
    pub common_info: CommonRecordInfo,
    /// The fields
    /// Stored in data
    pub fields: Vec<GeneralField<'data>>,
}
impl<'data> TypeNamed<'data> for GeneralRecord<'data> {
    fn type_name(&self) -> &'data BStr {
        self.type_name
    }
}
impl<'data> GeneralRecord<'data> {
    pub fn parse(data: &'data [u8]) -> IResult<&[u8], GeneralRecord<'data>> {
        let (data, type_name) = take(4usize)(data)?;
        let type_name = type_name.as_bstr();

        let (data, record_data_size) = le_u32(data)?;
        let (data, flags) = le_u32(data)?;
        let (data, id) = le_u32(data)?;
        let (data, version_control_info) = VersionControlInfo::parse(data)?;
        // IDEA: Perhaps the version is a four bit integer, and so unkown is simpler the lower bits of it?
        let (data, version) = le_u16(data)?;
        let (data, unknown) = le_u16(data)?;

        // TODO: verify it's all been used
        let (data, record_data) = take(record_data_size)(data)?;
        let (_, fields) = many0(GeneralField::parse)(record_data)?;

        Ok((
            data,
            GeneralRecord {
                type_name,
                common_info: CommonRecordInfo::new(
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

    pub fn fields_size(&self) -> usize {
        self.fields.iter().fold(0, |acc, x| acc + x.data_size())
    }
}
impl<'data> Writable for GeneralRecord<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert fields_size is u32
        (self.fields_size() as u32).write_to(w)?;
        self.common_info.write_to(w)?;
        for field in self.fields.iter() {
            field.write_to(w)?;
        }
        Ok(())
    }
}
impl<'data> DataSize for GeneralRecord<'data> {
    fn data_size(&self) -> usize {
        self.type_name.len() +
            4 + // data_size
            self.common_info.data_size() +
            self.fields_size()
    }
}

pub trait FromRecord<'data>: Sized {
    fn from_record(record: GeneralRecord<'data>) -> IResult<&[u8], Self>;
}

pub trait TypeNamed<'aleph>: Sized {
    fn type_name(&self) -> &'aleph BStr;
}

pub fn get_field<'data, I, F>(
    field_iter: &mut std::iter::Peekable<I>,
    expected_field_name: &BStr,
) -> nom::IResult<&'data [u8], Option<F>>
where
    I: Iterator<Item = GeneralField<'data>>,
    F: FromField<'data>,
{
    let next_field: Option<&GeneralField<'data>> = field_iter.peek();
    // TODO: hardcoding field name = bad
    if next_field
        .map(|x| x.type_name())
        .filter(|x| *x == expected_field_name)
        .is_none()
    {
        Ok((&[], None))
    } else {
        let field: GeneralField<'data> = field_iter.next().unwrap();
        assert_eq!(field.type_name(), expected_field_name);
        let (_, field): (_, F) = F::from_field(field)?;
        Ok((&[], Some(field)))
    }
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
