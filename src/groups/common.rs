use crate::{
    parse::{le_i32, le_u32, tag, take, PResult, ParseError},
    records::common::{
        FormId, FromRecord, FromRecordError, GeneralRecord, RecordName, TypeNamed,
        VersionControlInfo,
    },
    util::{byte, DataSize, Position, StaticDataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use derive_more::From;
use std::io::Write;

pub const GROUPH_SIZE: usize = 24;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CommonGroupInfo {
    version_control_info: VersionControlInfo,
    /// Values are different than in records
    /// All top-level groups have a 0 here, except for CELL, which can have 1 in some addons.
    /// Topic children have single-byte values or 0xcc..
    /// Interior cells have 0 or 0cc.. Some addons have value of 1
    /// Some cell/world related groups have a wde array of small to large values that are not form ids. May be 0xCC..
    unknown: u32,
}
impl Writable for CommonGroupInfo {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.version_control_info.write_to(w)?;
        self.unknown.write_to(w)
    }
}

#[derive(Debug, Clone)]
pub struct GeneralGroup<'data> {
    pub group_type: GroupType<'data>,
    pub common: CommonGroupInfo,
    /// Records and subgroups
    pub data: &'data [u8],
}
impl<'data> GeneralGroup<'data> {
    pub fn parse(data: &'data [u8]) -> PResult<Self> {
        let (data, _) = tag(data, b"GRUP")?;
        // Size of the entire group, including group header..
        let (data, group_size) = le_u32(data)?;
        let (data, group_type) = GroupType::parse(data)?;
        let (data, version_control_info) = VersionControlInfo::parse(data)?;
        let (data, unknown) = le_u32(data)?;
        let (data, group_data) = take(data, group_size as usize - GROUPH_SIZE)?;

        Ok((
            data,
            Self {
                group_type,
                common: CommonGroupInfo {
                    version_control_info,
                    unknown,
                },
                data: group_data,
            },
        ))
    }
}
impl<'data> DataSize for GeneralGroup<'data> {
    fn data_size(&self) -> usize {
        // same as value of group_size field, due to that containing header size
        GROUPH_SIZE + self.data.len()
    }
}
pub fn write_group_header<T: DataSize, W: Write>(group: &T, w: &mut W) -> std::io::Result<()> {
    b"GRUP".as_bstr().write_to(w)?;
    // TODO: assert that data size fits within u32
    // data size is equivalent to group size in file format
    (group.data_size() as u32).write_to(w)?;

    Ok(())
}
impl<'data> Writable for GeneralGroup<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_group_header(self, w)?;
        self.group_type.write_to(w)?;
        self.common.write_to(w)?;
        self.data.write_to(w)
    }
}
pub trait FromGeneralGroup<'data> {
    /// Panics on conversion failure
    fn from_general_group(group: GeneralGroup<'data>) -> Self;
}

#[derive(Debug, Clone)]
pub struct TopGroup<'data> {
    pub label: RecordName<'data>,
    pub common: CommonGroupInfo,
    pub data: &'data [u8],
}
impl<'data> FromGeneralGroup<'data> for TopGroup<'data> {
    fn from_general_group(group: GeneralGroup<'data>) -> Self {
        if let GroupType::Top(label) = group.group_type {
            Self {
                label,
                common: group.common,
                data: group.data,
            }
        } else {
            panic!(
                "Incorrect group type, expected Top, got: {:?}",
                group.group_type
            );
        }
    }
}
impl<'data> TypeNamed<'data> for TopGroup<'data> {
    fn type_name(&self) -> &'data BStr {
        self.label
    }
}
impl<'data> DataSize for TopGroup<'data> {
    fn data_size(&self) -> usize {
        GROUPH_SIZE + self.data.len()
    }
}
impl<'data> Writable for TopGroup<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_group_header(self, w)?;
        GroupType::Top(self.label).write_to(w)?;
        self.common.write_to(w)?;
        self.data.write_to(w)
    }
}

#[derive(Debug, Clone, From)]
pub enum FromTopGroupError<'data> {
    /// TODO: note that any parse errors inside records will be under FromRecordError.. this is fine I guess?
    RecordError(FromRecordError<'data>),
    ParseError(ParseError<'data>),
}

pub trait FromTopGroup<'data>: Sized {
    fn from_top_group(group: TopGroup<'data>) -> PResult<Self, FromTopGroupError<'data>>;
}

// TODO: this label storing behavior doesn't match What Record does
/// The GroupType. Holds the type and the label, since the lable depends on the group-type for meaning
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GroupType<'data> {
    /// Record type; 0
    /// Always 4 characters
    Top(RecordName<'data>),
    /// Parent (WRLD); 1
    WorldChildren(FormId),
    /// Block Number; 2
    InteriorCellBlock(u32),
    /// Sub-block number; 3
    InteriorSubCellBlock(u32),
    /// Grid Y, X; 4
    ExteriorCellBlock(Position<u16>),
    /// Grid Y, X; 5
    ExteriorCellSubBlock(Position<u16>),
    /// Parent (CELL); 6
    CellChildren(FormId),
    /// Parent (DIAL); 7
    TopicChildren(FormId),
    /// Parent (CELL); 8
    CellPersistentChildren(FormId),
    /// Parent (CELL); 9
    CellTemporaryChildren(FormId),

    /// An unknown entry
    Unknown { group_type: i32, label: [u8; 4] },
}
impl<'data> GroupType<'data> {
    pub fn parse(data: &'data [u8]) -> PResult<Self> {
        let (data, label) = take(data, 4)?;
        let (data, group_type) = le_i32(data)?;
        Ok((data, GroupType::from_info(group_type, label)))
    }

    // TODO: this needs testing
    // TODO: check endianess!!!
    // TODO: the uesp docs say that group_type is a int32 rather than a uint32, why? it doesn't have any negative values
    pub fn from_info(group_type: i32, data: &'data [u8]) -> GroupType<'data> {
        use GroupType as GT;
        let value = &data[0..4];
        match group_type {
            0 => GT::Top(value.as_bstr()),
            1 => GT::WorldChildren(FormId::from_bytes(byte::as_4_bytes(value))),
            2 => GT::InteriorCellBlock(u32::from_le_bytes(byte::as_4_bytes(value))),
            3 => GT::InteriorSubCellBlock(u32::from_le_bytes(byte::as_4_bytes(value))),
            4 => GT::ExteriorCellBlock(Position::new(
                // it's reversed
                u16::from_le_bytes([value[2], value[3]]),
                u16::from_le_bytes([value[0], value[1]]),
            )),
            5 => GT::ExteriorCellSubBlock(Position::new(
                // reversed
                u16::from_le_bytes([value[2], value[3]]),
                u16::from_le_bytes([value[0], value[1]]),
            )),
            6 => GT::CellChildren(FormId::from_bytes(byte::as_4_bytes(value))),
            7 => GT::TopicChildren(FormId::from_bytes(byte::as_4_bytes(value))),
            8 => GT::CellPersistentChildren(FormId::from_bytes(byte::as_4_bytes(value))),
            9 => GT::CellTemporaryChildren(FormId::from_bytes(byte::as_4_bytes(value))),
            _ => GT::Unknown {
                group_type,
                label: byte::as_4_bytes(value),
            },
        }
    }

    pub fn get_label(&self) -> [u8; 4] {
        match self {
            GroupType::Top(label) => {
                let b = label.as_bytes();
                [b[0], b[1], b[2], b[3]]
            }
            GroupType::WorldChildren(id)
            | GroupType::CellChildren(id)
            | GroupType::TopicChildren(id)
            | GroupType::CellPersistentChildren(id)
            | GroupType::CellTemporaryChildren(id) => id.id.to_le_bytes(),
            GroupType::InteriorCellBlock(n) | GroupType::InteriorSubCellBlock(n) => n.to_le_bytes(),
            GroupType::ExteriorCellBlock(pos) | GroupType::ExteriorCellSubBlock(pos) => {
                let x = pos.x.to_le_bytes();
                let y = pos.y.to_le_bytes();

                // They are stored in reverse order
                [y[0], y[1], x[0], x[1]]
            }
            GroupType::Unknown {
                group_type: _,
                label,
            } => *label,
        }
    }

    pub fn get_name(&self) -> i32 {
        match self {
            GroupType::Top(_) => 0,
            GroupType::WorldChildren(_) => 1,
            GroupType::InteriorCellBlock(_) => 2,
            GroupType::InteriorSubCellBlock(_) => 3,
            GroupType::ExteriorCellBlock(_) => 4,
            GroupType::ExteriorCellSubBlock(_) => 5,
            GroupType::CellChildren(_) => 6,
            GroupType::TopicChildren(_) => 7,
            GroupType::CellPersistentChildren(_) => 8,
            GroupType::CellTemporaryChildren(_) => 9,
            GroupType::Unknown {
                group_type,
                label: _,
            } => *group_type,
        }
    }
}
impl<'data> StaticDataSize for GroupType<'data> {
    fn static_data_size() -> usize {
        (u8::static_data_size() * 4) + // label
			i32::static_data_size() // group type enum value
    }
}
impl<'data> Writable for GroupType<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(&self.get_label())?;
        self.get_name().write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;

    #[test]
    fn test_grouptype() {
        let g = GroupType::Top(b"GMST".as_bstr());
        assert_eq!(g.data_size(), 8);
        let data = assert_size_output!(g);
        assert_eq!(data[0], b'G');
        assert_eq!(data[1], b'M');
        assert_eq!(data[2], b'S');
        assert_eq!(data[3], b'T');
        assert_eq!(data[4], 0x00);
        assert_eq!(data[5], 0x00);
        assert_eq!(data[6], 0x00);
        assert_eq!(data[7], 0x00);
    }
}

pub fn convert_all_records_into<'data, T>(
    records: Vec<GeneralRecord<'data>>,
) -> Result<Vec<T>, FromTopGroupError>
where
    T: FromRecord<'data>,
{
    let records: Result<Vec<T>, FromTopGroupError> = records
        .into_iter()
        .map(T::from_record)
        .map(|x| x.map(|x| x.1))
        .map(|x| x.map_err(|e| e.into()))
        .collect();

    records
}

// This would be easier if I could concatenate identifiers, but it's simply anyway
#[macro_export]
macro_rules! make_simple_top_group {
    ($(#[$outer:meta])* $group_name:ident, $name:ident, $record_name:ident, $life:lifetime) => {
		$(#[$outer])*
        #[derive(Debug, Clone)]
        pub struct $group_name<$life> {
            pub common: $crate::groups::common::CommonGroupInfo,
            pub records: Vec<$record_name<$life>>,
        }
        impl<$life> $crate::FromTopGroup<$life> for $group_name<$life> {
            fn from_top_group(group: $crate::groups::common::TopGroup<$life>) -> crate::parse::PResult<Self, crate::groups::common::FromTopGroupError> {
                let (data, records) = crate::parse::many(group.data, $crate::records::common::GeneralRecord::parse)?;
                if !data.is_empty() {
                    return Err(crate::parse::ParseError::ExpectedEOF.into());
                }

                let records = $crate::groups::common::convert_all_records_into(records)?;

                Ok((
                    data,
                    Self {
                        common: group.common,
                        records,
                    },
                ))
            }
        }
        impl<$life> $crate::records::common::TypeNamed<'static> for $group_name<$life> {
            fn type_name(&self) -> &'static bstr::BStr {
				use bstr::ByteSlice;
                stringify!($name).as_bytes().as_bstr()
            }
        }
        impl<$life> $crate::util::DataSize for $group_name<$life> {
            fn data_size(&self) -> usize {
                $crate::groups::common::GROUPH_SIZE + self.records.data_size()
            }
        }
        impl<$life> $crate::util::Writable for $group_name<$life> {
            fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
            where
                T: std::io::Write,
            {
				use $crate::records::common::TypeNamed;
                $crate::groups::common::write_group_header(self, w)?;
                $crate::groups::common::GroupType::Top(self.type_name()).write_to(w)?;
                self.common.write_to(w)?;
                self.records.write_to(w)
            }
        }
    };
}
