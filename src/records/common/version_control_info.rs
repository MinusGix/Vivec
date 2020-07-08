use crate::util::{StaticDataSize, Writable};
use nom::{bytes::complete::take, IResult};

/// Version Control User ID
pub type VUID = u8;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct VersionControlInfo {
    /// Day of the month
    pub day: u8,
    /// starts with 0 = December 2002
    pub month: u8,
    /// User id that last checked this out
    pub last_user_id: VUID,
    /// User-id that currently has this checked out
    pub current_user_id: VUID,
}
impl VersionControlInfo {
    pub fn new(
        day: u8,
        month: u8,
        last_user_id: VUID,
        current_user_id: VUID,
    ) -> VersionControlInfo {
        VersionControlInfo {
            day,
            month,
            last_user_id,
            current_user_id,
        }
    }

    pub fn parse(data: &[u8]) -> IResult<&[u8], VersionControlInfo> {
        let (data, day) = take(1usize)(data)?;
        let (data, month) = take(1usize)(data)?;
        let (data, last_user_id) = take(1usize)(data)?;
        let (data, current_user_id) = take(1usize)(data)?;
        Ok((
            data,
            VersionControlInfo::new(day[0], month[0], last_user_id[0], current_user_id[0]),
        ))
    }
}
impl Writable for VersionControlInfo {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.day.write_to(w)?;
        self.month.write_to(w)?;
        self.last_user_id.write_to(w)?;
        self.current_user_id.write_to(w)
    }
}
impl StaticDataSize for VersionControlInfo {
    fn static_data_size() -> usize {
        u8::static_data_size() // day
            + u8::static_data_size() // month
            + VUID::static_data_size() // last_user_id
            + VUID::static_data_size() // current_user_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;
    #[test]
    fn test_vci() {
        let v = VersionControlInfo {
            day: 5,
            month: 7,
            last_user_id: 0xaa,
            current_user_id: 0x30,
        };
        let data = assert_size_output!(v);
        assert_eq!(data[0], 0x05);
        assert_eq!(data[1], 0x07);
        assert_eq!(data[2], 0xaa);
        assert_eq!(data[3], 0x30);
    }
}