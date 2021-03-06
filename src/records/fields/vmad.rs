use super::common::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE};
use crate::{
    dispatch_all, impl_static_data_size,
    parse::{count, many, take, PResult, Parse, ParseError},
    records::common::{ConversionError, FormId, StaticTypeNamed, Windows1252String16},
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use std::{convert::TryFrom, io::Write};

/// A trait for fragment data, since the interpretation of Fragments (and if they exist at all) is dependent on the parent Record
pub trait ParseFragments<'data>: Sized + DataSize + Writable {
    fn parse_fragments(data: &'data [u8]) -> PResult<Self>;
}

/// Contains Papyrus script data
#[derive(Debug, Clone, PartialEq)]
pub struct VMAD<'data, Fragment: ParseFragments<'data>> {
    pub primary: VMADPrimarySection<'data, Fragment>,
}
impl<'data, Fragment> FromField<'data> for VMAD<'data, Fragment>
where
    Fragment: ParseFragments<'data>,
{
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, primary) = VMADPrimarySection::parse(field.data)?;
        Ok((data, VMAD { primary }))
    }
}
impl<'data, Fragment> StaticTypeNamed for VMAD<'data, Fragment>
where
    Fragment: ParseFragments<'data>,
{
    fn static_type_name() -> &'static BStr {
        b"VMAD".as_bstr()
    }
}
impl<'data, Fragment> DataSize for VMAD<'data, Fragment>
where
    Fragment: ParseFragments<'data>,
{
    fn data_size(&self) -> usize {
        FIELDH_SIZE + self.primary.data_size()
    }
}
impl<'data, Fragment> Writable for VMAD<'data, Fragment>
where
    Fragment: ParseFragments<'data>,
{
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.primary.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VMADPrimarySection<'data, Fragment: ParseFragments<'data>> {
    // TODO: unlikely to be signed...
    pub version: i16,
    /// UESP: always seems to be 1 or 2, affects how object-type properties are read
    pub object_format: VMADObjectFormat,
    /// Information on each of the scripts
    pub scripts: Vec<VMADScript<'data>>,
    /// Script fragments
    pub fragments: Vec<Fragment>,
}
impl<'data, Fragment> Parse<'data> for VMADPrimarySection<'data, Fragment>
where
    Fragment: ParseFragments<'data>,
{
    fn parse(data: &'data [u8]) -> PResult<Self> {
        let (data, version) = i16::parse(data)?;
        let (data, object_format) = VMADObjectFormat::parse(data)?;
        let (data, script_count) = u16::parse(data)?;
        // since it's script count rather than the size of the data that is scripts, that makes life slightly harder
        let (data, scripts) = count(
            data,
            |x| VMADScript::parse(x, object_format),
            script_count as usize,
        )?;
        // We only want to try parsing the rest as fragments if there isn't anything left.
        // many0 would still try calling the function, even if there is no data left, which is not what I want.
        let fragments = if !data.is_empty() {
            many(data, Fragment::parse_fragments)?.1
        } else {
            Vec::new()
        };

        Ok((
            data,
            VMADPrimarySection {
                version,
                object_format,
                scripts,
                fragments,
            },
        ))
    }
}
impl<'data, Fragment> DataSize for VMADPrimarySection<'data, Fragment>
where
    Fragment: ParseFragments<'data>,
{
    fn data_size(&self) -> usize {
        use crate::util::StaticDataSize;
        self.version.data_size()
            + self.object_format.data_size()
            + u16::static_data_size() // scripts count size
            + self.scripts.data_size()
            + self.fragments.data_size()
    }
}
impl<'data, Fragment> Writable for VMADPrimarySection<'data, Fragment>
where
    Fragment: ParseFragments<'data>,
{
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.version.write_to(w)?;
        self.object_format.write_to(w)?;
        // TODO: assert that it fits
        (self.scripts.len() as u16).write_to(w)?;
        // FIXME: I HATE THIS BLOODY AAAAAAGH. Essentially, VMADPropertyObject depends upon the VMADObjectFormat
        // stored up here for how it should be read/written (I hate that as well), which means we need to pass it along
        // this breaks us out of implementing Writable for everything, since we need an extra parameter
        // and it forces us to pass it along everywhere, which infects others parts. Also breaks dispatch_all!.
        for script in self.scripts.iter() {
            script.write_to(w, self.object_format)?;
        }
        self.fragments.write_to(w)
    }
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(i16)]
pub enum VMADObjectFormat {
    /// [formid:4][alias:2][unused:2]
    IDLead = 1,
    /// [unused:2][alias:2][formid:4]
    IDEnd = 2,
}
type VMADObjectFormatConversionError = ConversionError<u16>;
impl VMADObjectFormat {
    pub fn code(&self) -> i16 {
        *self as i16
    }
}
impl Parse<'_> for VMADObjectFormat {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, value) = u16::parse(data)?;
        let object_format = VMADObjectFormat::try_from(value)?;
        Ok((data, object_format))
    }
}
impl TryFrom<u16> for VMADObjectFormat {
    type Error = VMADObjectFormatConversionError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(VMADObjectFormat::IDLead),
            2 => Ok(VMADObjectFormat::IDEnd),
            x => Err(Self::Error::InvalidEnumerationValue(x)),
        }
    }
}
impl_static_data_size!(VMADObjectFormat, u16::static_data_size());
impl Writable for VMADObjectFormat {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.code().write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VMADScript<'data> {
    /// The name of the script, without an extension
    pub name: Windows1252String16<'data>,
    /// FIXME: Check what the UESP docs mean when it says 'only present if', is it just zeroed out before, or was it previously non existant
    /// Only present if version >= 4, defaults to 0 for earlier versions
    /// 0 = local script
    /// 1 = inherited and properties have been altered
    /// 3 = inherited and then removed
    pub status: u8,
    /// The properties
    pub properties: Vec<VMADProperty<'data>>,
}
impl<'data> VMADScript<'data> {
    pub fn parse(data: &'data [u8], object_format: VMADObjectFormat) -> PResult<Self> {
        let (data, name) = Windows1252String16::parse(data)?;
        let (data, status) = take(data, 1usize)?;
        let (data, property_count) = u16::parse(data)?;
        let (data, properties) = count(
            data,
            |x| VMADProperty::parse(x, object_format),
            property_count as usize,
        )?;
        Ok((
            data,
            VMADScript {
                name,
                status: status[0],
                properties,
            },
        ))
    }

    fn write_to<T>(&self, w: &mut T, object_format: VMADObjectFormat) -> std::io::Result<()>
    where
        T: Write,
    {
        self.name.write_to(w)?;
        self.status.write_to(w)?;
        // TODO: assert that is within range
        (self.properties.len() as u16).write_to(w)?;
        for property in self.properties.iter() {
            property.write_to(w, object_format)?;
        }
        Ok(())
    }
}
impl<'data> DataSize for VMADScript<'data> {
    fn data_size(&self) -> usize {
        self.name.data_size() +
            self.status.data_size() +
            2 + // properties u16 len
            self.properties.data_size()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VMADPropertyData<'data> {
    /// UESP: "Object types are used to assign formid values to props, in particular for quest aliases, but also for a range of other cases that use formids.
    /// it's length is always 8 bytes, but how the bytes are decoded depends upon object format":
    /// object_format=1: [formid: u32][alias: u16][zeros: u16]
    /// object_format=2: [zeros: u16][alias: u16][formid: u32]
    /// Aliasid is -1 (255??) whenever formid does not point to a quest (the formid is then directly assigned to the property).
    /// if it is not -1, then it provides the quest alias used to assign the value
    /// 1
    Object(VMADPropertyObject),
    /// 2
    Windows1252String16(Windows1252String16<'data>),
    /// 3
    Int32(i32),
    /// 4
    Float(f32),
    /// u8
    /// 5
    Boolean(bool),

    // Only supported if version >= 5
    /// 11
    ObjectArray(Vec<VMADPropertyObject>),
    /// 12
    Windows1252String16Array(Vec<Windows1252String16<'data>>),
    /// 13
    Int32Array(Vec<i32>),
    /// 14
    FloatArray(Vec<f32>),
    /// 15
    BooleanArray(Vec<bool>),
}
impl<'data> VMADPropertyData<'data> {
    pub fn parse(
        data: &'data [u8],
        object_format: VMADObjectFormat,
        property_type: u8,
    ) -> PResult<VMADPropertyData<'data>> {
        match property_type {
            1 => {
                let (data, value) = VMADPropertyObject::parse(data, object_format)?;
                Ok((data, VMADPropertyData::Object(value)))
            }
            2 => {
                let (data, value) = Windows1252String16::parse(data)?;
                Ok((data, VMADPropertyData::Windows1252String16(value)))
            }
            3 => {
                let (data, value) = i32::parse(data)?;
                Ok((data, VMADPropertyData::Int32(value)))
            }
            4 => {
                let (data, value) = f32::parse(data)?;
                Ok((data, VMADPropertyData::Float(value)))
            }
            5 => {
                let (data, value) = take(data, 1usize)?;
                let value = value[0] != 0;
                Ok((data, VMADPropertyData::Boolean(value)))
            }

            // only supported if version >= 5
            11 => {
                let (data, amount) = u32::parse(data)?;
                // TODO: we could just `take` the amount of bytes, since the size is statically known
                let (data, items) = count(
                    data,
                    |x| VMADPropertyObject::parse(x, object_format),
                    amount as usize,
                )?;
                Ok((data, VMADPropertyData::ObjectArray(items)))
            }
            12 => {
                let (data, amount) = u32::parse(data)?;
                let (data, items) = count(data, Windows1252String16::parse, amount as usize)?;
                Ok((data, VMADPropertyData::Windows1252String16Array(items)))
            }
            13 => {
                let (data, amount) = u32::parse(data)?;
                let (data, items) = count(data, i32::parse, amount as usize)?;
                Ok((data, VMADPropertyData::Int32Array(items)))
            }
            14 => {
                let (data, amount) = u32::parse(data)?;
                let (data, items) = count(data, f32::parse, amount as usize)?;
                Ok((data, VMADPropertyData::FloatArray(items)))
            }
            15 => {
                let (data, amount) = u32::parse(data)?;
                // TODO: I hate it
                let (data, items) = count(
                    data,
                    |x: &[u8]| -> PResult<bool> {
                        let (data, value) = u8::parse(x)?;
                        Ok((data, value != 0))
                    },
                    amount as usize,
                )?;
                Ok((data, VMADPropertyData::BooleanArray(items)))
            }

            _ => Err(ParseError::InvalidEnumerationValue),
        }
    }

    pub fn is_type_valid_for_version(property_type: u8, version: u16) -> bool {
        match property_type {
            1 | 2 | 3 | 5 => true,
            11 | 12 | 13 | 14 | 15 => version >= 5,
            // TODO: should this do something different with a completely invalid property type?
            _ => false,
        }
    }

    /// Get the code (aka the value representing it in a file). This would be the 'property type' in the file
    pub fn code(&self) -> u8 {
        match self {
            VMADPropertyData::Object(_) => 1,
            VMADPropertyData::Windows1252String16(_) => 2,
            VMADPropertyData::Int32(_) => 3,
            VMADPropertyData::Float(_) => 4,
            VMADPropertyData::Boolean(_) => 5,

            VMADPropertyData::ObjectArray(_) => 11,
            VMADPropertyData::Windows1252String16Array(_) => 12,
            VMADPropertyData::Int32Array(_) => 13,
            VMADPropertyData::FloatArray(_) => 14,
            VMADPropertyData::BooleanArray(_) => 15,
        }
    }

    // There would be a u8 (status) between the type and the data, so we have to make it in separate steps :/

    pub fn write_type_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.code().write_to(w)
    }

    pub fn write_data_to<T>(
        &self,
        w: &mut T,
        object_format: VMADObjectFormat,
    ) -> std::io::Result<()>
    where
        T: Write,
    {
        match self {
            VMADPropertyData::Object(x) => x.write_to(w, object_format),
            VMADPropertyData::Windows1252String16(x) => x.write_to(w),
            VMADPropertyData::Int32(x) => x.write_to(w),
            VMADPropertyData::Float(x) => x.write_to(w),
            VMADPropertyData::Boolean(x) => x.write_to(w),
            VMADPropertyData::ObjectArray(x) => {
                for object in x {
                    object.write_to(w, object_format)?;
                }
                Ok(())
            }
            VMADPropertyData::Windows1252String16Array(x) => x.write_to(w),
            VMADPropertyData::Int32Array(x) => x.write_to(w),
            VMADPropertyData::FloatArray(x) => x.write_to(w),
            VMADPropertyData::BooleanArray(x) => x.write_to(w),
        }
    }
}
// DataSize isn't entirely meaningful for VMADPropertyData
impl<'data> DataSize for VMADPropertyData<'data> {
    fn data_size(&self) -> usize {
        self.code().data_size()
            + dispatch_all!(
                VMADPropertyData,
                self,
                [
                    Object,
                    Windows1252String16,
                    Int32,
                    Float,
                    Boolean,
                    ObjectArray,
                    Windows1252String16Array,
                    Int32Array,
                    FloatArray,
                    BooleanArray
                ],
                x,
                { x.data_size() }
            )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct VMADPropertyObject {
    pub formid: FormId,
    pub alias: u16,
    /// I store this just in case it is actually useful
    pub unused: u16,
}
impl VMADPropertyObject {
    pub fn parse(data: &[u8], object_format: VMADObjectFormat) -> PResult<VMADPropertyObject> {
        match object_format {
            // [formid:4][alias:2][zeros:2]
            VMADObjectFormat::IDLead => {
                let (data, formid) = FormId::parse(data)?;
                let (data, alias) = u16::parse(data)?;
                let (data, unused) = u16::parse(data)?;
                Ok((
                    data,
                    VMADPropertyObject {
                        formid,
                        alias,
                        unused,
                    },
                ))
            }
            // [zeros:2][alias:2][formid:4]
            VMADObjectFormat::IDEnd => {
                let (data, unused) = u16::parse(data)?;
                let (data, alias) = u16::parse(data)?;
                let (data, formid) = FormId::parse(data)?;
                Ok((
                    data,
                    VMADPropertyObject {
                        formid,
                        alias,
                        unused,
                    },
                ))
            }
        }
    }

    /// Fake Writable impl, since it needs extra info :/
    pub fn write_to<T>(&self, w: &mut T, object_format: VMADObjectFormat) -> std::io::Result<()>
    where
        T: Write,
    {
        match object_format {
            VMADObjectFormat::IDLead => {
                self.formid.write_to(w)?;
                self.alias.write_to(w)?;
                self.unused.write_to(w)
            }
            VMADObjectFormat::IDEnd => {
                self.unused.write_to(w)?;
                self.alias.write_to(w)?;
                self.formid.write_to(w)
            }
        }
    }
}
impl_static_data_size!(
    VMADPropertyObject,
    FormId::static_data_size() + // formid
    u16::static_data_size() + // alias
    u16::static_data_size() // unused
);

#[derive(Debug, Clone, PartialEq)]
pub struct VMADProperty<'data> {
    pub name: Windows1252String16<'data>,
    /// FIXME: UESP says only present if version >= 4
    /// Defaults to 1 for earlier than version 4
    /// 1 = property edited
    /// 3 = property removed
    pub status: u8,
    /// The representation depends on the type, which is just merged with the data here
    pub data: VMADPropertyData<'data>,
}
impl<'data> VMADProperty<'data> {
    pub fn parse(data: &'data [u8], object_format: VMADObjectFormat) -> PResult<Self> {
        let (data, name) = Windows1252String16::parse(data)?;
        let (data, property_type) = take(data, 1usize)?;
        let property_type = property_type[0];
        let (data, status) = take(data, 1usize)?;
        let status = status[0];
        let (data, property_data) = VMADPropertyData::parse(data, object_format, property_type)?;
        Ok((
            data,
            VMADProperty {
                name,
                status,
                data: property_data,
            },
        ))
    }

    pub fn write_to<T>(&self, w: &mut T, object_format: VMADObjectFormat) -> std::io::Result<()>
    where
        T: Write,
    {
        self.name.write_to(w)?;
        self.data.write_type_to(w)?;
        self.status.write_to(w)?;
        self.data.write_data_to(w, object_format)
    }
}
impl<'data> DataSize for VMADProperty<'data> {
    fn data_size(&self) -> usize {
        self.name.data_size() + self.status.data_size() + self.data.data_size()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoFragments {}
impl<'data> ParseFragments<'data> for NoFragments {
    fn parse_fragments(_data: &'data [u8]) -> PResult<Self> {
        Err(ParseError::ExpectedEOF)
    }
}
impl_static_data_size!(NoFragments, 0);
impl Writable for NoFragments {
    fn write_to<T>(&self, _w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        // TODO: should this fail since it shouldn't exist?
        Ok(())
    }
}

/// Stored by default in a TIF file, ex: TIF_[editorId]_[formId]
/// Since most INFO records do not have an editorID, it stores as TIF__[formId]
#[derive(Debug, Clone, PartialEq)]
pub struct INFORecordFragments<'data> {
    /// Always 2
    pub unknown: u8,
    /// script locations
    pub flags: INFORecordFragmentsFlags,
    /// Name of the script file containing the fragments, without extension
    pub filename: Windows1252String16<'data>,
    /// Information on each fragment
    /// size is the number of bit flags activated in flags (wew)
    pub fragments: Vec<FragmentInfo<'data>>,
}
impl<'data> ParseFragments<'data> for INFORecordFragments<'data> {
    fn parse_fragments(data: &'data [u8]) -> PResult<Self> {
        let (data, unknown) = take(data, 1usize)?;
        let unknown = unknown[0];
        assert_eq!(unknown, 2);
        let (data, flags) = INFORecordFragmentsFlags::parse(data)?;
        let (data, filename) = Windows1252String16::parse(data)?;
        // The amount of fragments is the amount of bits set in flags. Scary, but an interesting way to do it.
        let (data, fragments) = count(data, FragmentInfo::parse, flags.count_ones() as usize)?;
        Ok((
            data,
            INFORecordFragments {
                unknown,
                flags,
                filename,
                fragments,
            },
        ))
    }
}
impl<'data> DataSize for INFORecordFragments<'data> {
    fn data_size(&self) -> usize {
        self.unknown.data_size()
            + self.flags.data_size()
            + self.filename.data_size()
            + self.fragments.data_size()
    }
}
impl<'data> Writable for INFORecordFragments<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.unknown.write_to(w)?;
        self.flags.write_to(w)?;
        self.filename.write_to(w)?;
        self.fragments.write_to(w)
    }
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct INFORecordFragmentsFlags {
    /// 0x1: has begin script
    /// 0x2: has end script
    pub flags: u8,
}
impl INFORecordFragmentsFlags {
    pub fn new(flags: u8) -> Self {
        Self { flags }
    }

    // TODO: verify this
    pub fn has_begin_script(&self) -> bool {
        (self.flags & 0b1) != 0
    }

    // TODO: verify this
    pub fn has_end_script(&self) -> bool {
        (self.flags & 0b10) != 0
    }

    pub fn count_ones(&self) -> u8 {
        self.flags.count_ones() as u8
    }
}
impl Parse<'_> for INFORecordFragmentsFlags {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = take(data, 1usize)?;
        Ok((data, Self::new(flags[0])))
    }
}
impl_static_data_size!(INFORecordFragmentsFlags, u8::static_data_size());
impl Writable for INFORecordFragmentsFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FragmentInfo<'data> {
    pub unknown: u8,
    /// Name of script. Normally same as parent INFORecord.filename
    pub script_name: Windows1252String16<'data>,
    /// Name of function containing this fragment script. Usually, something like "Fragment_3"
    pub fragment_name: Windows1252String16<'data>,
}
impl<'data> Parse<'data> for FragmentInfo<'data> {
    fn parse(data: &'data [u8]) -> PResult<Self> {
        let (data, unknown) = take(data, 1usize)?;
        let unknown = unknown[0];
        let (data, script_name) = Windows1252String16::parse(data)?;
        let (data, fragment_name) = Windows1252String16::parse(data)?;
        Ok((
            data,
            FragmentInfo {
                unknown,
                script_name,
                fragment_name,
            },
        ))
    }
}
impl<'data> DataSize for FragmentInfo<'data> {
    fn data_size(&self) -> usize {
        self.unknown.data_size() + self.script_name.data_size() + self.fragment_name.data_size()
    }
}
impl<'data> Writable for FragmentInfo<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.unknown.write_to(w)?;
        self.script_name.write_to(w)?;
        self.fragment_name.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PACKRecordFragments<'data> {
    /// Always 2
    pub unknown: u8,
    /// script locations
    pub flags: PACKRecordFragmentsFlags,
    /// Name of script file containing fragments, without extension
    pub filename: Windows1252String16<'data>,
    /// Length is the number of bits set in flags
    /// When more than one is present, fragments are emitted in the order: On Begin, On End, On change
    pub fragments: Vec<FragmentInfo<'data>>,
}
impl<'data> ParseFragments<'data> for PACKRecordFragments<'data> {
    fn parse_fragments(data: &'data [u8]) -> PResult<Self> {
        let (data, unknown) = take(data, 1usize)?;
        let unknown = unknown[0];
        assert_eq!(unknown, 2);
        let (data, flags) = PACKRecordFragmentsFlags::parse(data)?;
        let (data, filename) = Windows1252String16::parse(data)?;
        let (data, fragments) = count(data, FragmentInfo::parse, flags.count_ones() as usize)?;
        Ok((
            data,
            PACKRecordFragments {
                unknown,
                flags,
                filename,
                fragments,
            },
        ))
    }
}
impl<'data> DataSize for PACKRecordFragments<'data> {
    fn data_size(&self) -> usize {
        self.unknown.data_size()
            + self.flags.data_size()
            + self.filename.data_size()
            + self.fragments.data_size()
    }
}
impl<'data> Writable for PACKRecordFragments<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.unknown.write_to(w)?;
        self.flags.write_to(w)?;
        self.filename.write_to(w)?;
        self.fragments.write_to(w)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PACKRecordFragmentsFlags {
    /// 0x1 = on begin
    /// 0x2 = on end
    /// 0x4 = on change
    pub flags: u8,
}
impl PACKRecordFragmentsFlags {
    pub fn new(flags: u8) -> Self {
        Self { flags }
    }

    pub fn has_on_begin(&self) -> bool {
        (self.flags & 0b1) != 0
    }

    pub fn has_on_end(&self) -> bool {
        (self.flags & 0b10) != 0
    }

    pub fn has_on_change(&self) -> bool {
        (self.flags & 0b100) != 0
    }

    pub fn count_ones(&self) -> u8 {
        self.flags.count_ones() as u8
    }
}
impl Parse<'_> for PACKRecordFragmentsFlags {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = take(data, 1usize)?;
        Ok((data, Self::new(flags[0])))
    }
}
impl_static_data_size!(PACKRecordFragmentsFlags, u8::static_data_size());
impl Writable for PACKRecordFragmentsFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PERKRecordFragments<'data> {
    /// always 2
    pub unknown: u8,
    /// Name of script file containing fragments, without extension
    pub filename: Windows1252String16<'data>,
    pub fragments: Vec<PERKRecordFragmentInfo<'data>>,
}
impl<'data> ParseFragments<'data> for PERKRecordFragments<'data> {
    fn parse_fragments(data: &'data [u8]) -> PResult<Self> {
        let (data, unknown) = take(data, 1usize)?;
        let unknown = unknown[0];
        let (data, filename) = Windows1252String16::parse(data)?;
        let (data, fragment_count) = u16::parse(data)?;
        let (data, fragments) =
            count(data, PERKRecordFragmentInfo::parse, fragment_count as usize)?;
        Ok((
            data,
            PERKRecordFragments {
                unknown,
                filename,
                fragments,
            },
        ))
    }
}
impl<'data> DataSize for PERKRecordFragments<'data> {
    fn data_size(&self) -> usize {
        self.unknown.data_size() + self.filename.data_size() + 2 + self.fragments.data_size()
    }
}
impl<'data> Writable for PERKRecordFragments<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.unknown.write_to(w)?;
        self.filename.write_to(w)?;
        // TODO: assert that it fits
        (self.fragments.len() as u16).write_to(w)?;
        self.fragments.write_to(w)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct PERKRecordFragmentInfo<'data> {
    /// Index into what??
    pub index: u16,
    pub unknown: u16,
    pub unknown2: u8,
    /// Typically same as parent INFORecord.filename
    pub script_name: Windows1252String16<'data>,
    /// Name of fragment. Usually a name like "Fragment_3"
    pub fragment_name: Windows1252String16<'data>,
}
impl<'data> Parse<'data> for PERKRecordFragmentInfo<'data> {
    fn parse(data: &'data [u8]) -> PResult<Self> {
        let (data, index) = u16::parse(data)?;
        let (data, unknown) = u16::parse(data)?;
        let (data, unknown2) = take(data, 1usize)?;
        let unknown2 = unknown2[0];
        let (data, script_name) = Windows1252String16::parse(data)?;
        let (data, fragment_name) = Windows1252String16::parse(data)?;
        Ok((
            data,
            PERKRecordFragmentInfo {
                index,
                unknown,
                unknown2,
                script_name,
                fragment_name,
            },
        ))
    }
}
impl<'data> DataSize for PERKRecordFragmentInfo<'data> {
    fn data_size(&self) -> usize {
        self.index.data_size()
            + self.unknown.data_size()
            + self.unknown2.data_size()
            + self.script_name.data_size()
            + self.fragment_name.data_size()
    }
}
impl<'data> Writable for PERKRecordFragmentInfo<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.index.write_to(w)?;
        self.unknown.write_to(w)?;
        self.script_name.write_to(w)?;
        self.fragment_name.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QUSTRecordFragments<'data> {
    /// always 2
    pub unknown: u8,
    /// Name of script file containing the fragments, without extension
    pub filename: Windows1252String16<'data>,
    pub fragments: Vec<QUSTRecordFragmentInfo<'data>>,
    /// Info on scripts attached to each alias
    pub aliases: Vec<FragmentAlias<'data>>,
}
impl<'data> ParseFragments<'data> for QUSTRecordFragments<'data> {
    fn parse_fragments(data: &'data [u8]) -> PResult<Self> {
        let (data, unknown) = take(data, 1usize)?;
        let unknown = unknown[0];
        assert_eq!(unknown, 2);
        let (data, fragment_count) = u16::parse(data)?;
        let (data, filename) = Windows1252String16::parse(data)?;
        let (data, fragments) =
            count(data, QUSTRecordFragmentInfo::parse, fragment_count as usize)?;
        let (data, alias_count) = u16::parse(data)?;
        let (data, aliases) = count(data, FragmentAlias::parse, alias_count as usize)?;
        Ok((
            data,
            QUSTRecordFragments {
                unknown,
                filename,
                fragments,
                aliases,
            },
        ))
    }
}
impl<'data> DataSize for QUSTRecordFragments<'data> {
    fn data_size(&self) -> usize {
        self.unknown.data_size()
            + 2 // fragment count len
            + self.filename.data_size()
            + self.fragments.data_size()
            + 2 // alias count len
            + self.aliases.data_size()
    }
}
impl<'data> Writable for QUSTRecordFragments<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.unknown.write_to(w)?;
        // TODO: assert that it fits
        (self.fragments.len() as u16).write_to(w)?;
        self.filename.write_to(w)?;
        self.fragments.write_to(w)?;
        // TODO: assert that it fits
        (self.aliases.len() as u16).write_to(w)?;
        self.aliases.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QUSTRecordFragmentInfo<'data> {
    /// Quest stage index (same as QUST INDX field) that this fragment is attached to
    pub index: u16,
    /// always 0
    pub unknown: u16,
    // TODO: is this really signed?
    /// Log entry within a stage this fragment is attached to
    pub log_entry: i32,
    /// always 1
    pub unknown2: u8,
    /// Name of script. Normally same as parent INFORecord.filename
    pub script_name: Windows1252String16<'data>,
    /// Name of function containing this fragment script
    pub fragment_name: Windows1252String16<'data>,
}
impl<'data> Parse<'data> for QUSTRecordFragmentInfo<'data> {
    fn parse(data: &'data [u8]) -> PResult<Self> {
        let (data, index) = u16::parse(data)?;
        let (data, unknown) = u16::parse(data)?;
        assert_eq!(unknown, 0);
        let (data, log_entry) = i32::parse(data)?;
        let (data, unknown2) = take(data, 1usize)?;
        let unknown2 = unknown2[0];
        let (data, script_name) = Windows1252String16::parse(data)?;
        let (data, fragment_name) = Windows1252String16::parse(data)?;
        Ok((
            data,
            Self {
                index,
                unknown,
                log_entry,
                unknown2,
                script_name,
                fragment_name,
            },
        ))
    }
}
impl<'data> DataSize for QUSTRecordFragmentInfo<'data> {
    fn data_size(&self) -> usize {
        self.index.data_size()
            + self.unknown.data_size()
            + self.log_entry.data_size()
            + self.unknown2.data_size()
            + self.script_name.data_size()
            + self.fragment_name.data_size()
    }
}
impl<'data> Writable for QUSTRecordFragmentInfo<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.index.write_to(w)?;
        self.unknown.write_to(w)?;
        self.log_entry.write_to(w)?;
        self.unknown2.write_to(w)?;
        self.script_name.write_to(w)?;
        self.fragment_name.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FragmentAlias<'data> {
    pub object: VMADPropertyObject,
    /// Always 4 or 5. always the same as primary script's version
    pub version: u16,
    /// Always 1 or 2. Always the same as primarily script's object_format
    pub object_format: VMADObjectFormat,
    // TODO: verify that it is supposed to be a VMADScript..
    pub scripts: Vec<VMADScript<'data>>,
}
impl<'data> Parse<'data> for FragmentAlias<'data> {
    // TODO: verify that version and object_format are equivalent to parents
    fn parse(data: &'data [u8]) -> PResult<Self> {
        // TODO: hardcoding byte size is bleh
        // We need the object format, which is stored later, so we simply consume the bytes needed for now
        let (data, object) = take(data, 4usize)?;

        let (data, version) = u16::parse(data)?;

        let (data, object_format) = VMADObjectFormat::parse(data)?;
        // We've gotten the object format, use it to parse the data
        let (_, object) = VMADPropertyObject::parse(object, object_format)?;

        let (data, script_count) = u16::parse(data)?;
        let (data, scripts) = count(
            data,
            |x| VMADScript::parse(x, object_format),
            script_count as usize,
        )?;
        Ok((
            data,
            Self {
                object,
                version,
                object_format,
                scripts,
            },
        ))
    }
}
impl<'data> DataSize for FragmentAlias<'data> {
    fn data_size(&self) -> usize {
        self.object.data_size() + self.version.data_size()
    }
}
impl<'data> Writable for FragmentAlias<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.object.write_to(w, self.object_format)?;
        self.version.write_to(w)?;
        self.object_format.write_to(w)?;
        // TODO: asssert that it fits within
        (self.scripts.len() as u16).write_to(w)?;
        for script in self.scripts.iter() {
            script.write_to(w, self.object_format)?;
        }
        Ok(())
    }
}

/// Stored in a SF file: "SF_[editorId]_[formId]"
#[derive(Debug, Clone, PartialEq)]
pub struct SCENRecordFragments<'data> {
    /// always 2
    pub unknown: u8,
    /// script locations
    /// 0x1 = has begin script
    /// 0x2 has end script
    pub flags: SCENRecordFragmentsFlags,
    /// name of the script file containing fragments, without extension
    pub filename: Windows1252String16<'data>,
    /// Info on begin/end fragments
    /// size is the number of bits set in [flags]
    /// when both are set, Begin fragment comes first
    pub begin_end: Vec<BEFragmentInfo<'data>>,
    /// Info on phase fragments
    pub phases: Vec<PhaseInfo<'data>>,
}
impl<'data> ParseFragments<'data> for SCENRecordFragments<'data> {
    fn parse_fragments(data: &'data [u8]) -> PResult<Self> {
        let (data, unknown) = take(data, 1usize)?;
        let unknown = unknown[0];
        assert_eq!(unknown, 2);
        let (data, flags) = SCENRecordFragmentsFlags::parse(data)?;
        let (data, filename) = Windows1252String16::parse(data)?;
        let (data, begin_end) = count(data, BEFragmentInfo::parse, flags.count_ones() as usize)?;
        let (data, phase_count) = u16::parse(data)?;
        let (data, phases) = count(data, PhaseInfo::parse, phase_count as usize)?;
        Ok((
            data,
            SCENRecordFragments {
                unknown,
                flags,
                filename,
                begin_end,
                phases,
            },
        ))
    }
}
impl<'data> DataSize for SCENRecordFragments<'data> {
    fn data_size(&self) -> usize {
        self.unknown.data_size()
            + self.flags.data_size()
            + self.filename.data_size()
            + self.begin_end.data_size()
            + 2 // phases count len
            + self.phases.data_size()
    }
}
impl<'data> Writable for SCENRecordFragments<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.unknown.write_to(w)?;
        self.flags.write_to(w)?;
        self.filename.write_to(w)?;
        self.begin_end.write_to(w)?;
        // TODO: assert that it fits within
        (self.phases.len() as u16).write_to(w)?;
        self.phases.write_to(w)
    }
}

// We just type alias it, since from what I know they're the same
pub type SCENRecordFragmentsFlags = INFORecordFragmentsFlags;

#[derive(Debug, Clone, PartialEq)]
pub struct BEFragmentInfo<'data> {
    pub unknown: u8,
    /// Tends to equal parent filename
    pub script_name: Windows1252String16<'data>,
    /// name of function containing this fragment script
    pub fragment_name: Windows1252String16<'data>,
}
impl<'data> Parse<'data> for BEFragmentInfo<'data> {
    fn parse(data: &'data [u8]) -> PResult<Self> {
        let (data, unknown) = take(data, 1usize)?;
        let unknown = unknown[0];
        let (data, script_name) = Windows1252String16::parse(data)?;
        let (data, fragment_name) = Windows1252String16::parse(data)?;
        Ok((
            data,
            BEFragmentInfo {
                unknown,
                script_name,
                fragment_name,
            },
        ))
    }
}
impl<'data> DataSize for BEFragmentInfo<'data> {
    fn data_size(&self) -> usize {
        self.unknown.data_size() + self.script_name.data_size() + self.fragment_name.data_size()
    }
}
impl<'data> Writable for BEFragmentInfo<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.unknown.write_to(w)?;
        self.script_name.write_to(w)?;
        self.fragment_name.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseInfo<'data> {
    pub unknown: u8,
    /// Phase number. In creation kit, it starts at 1, but in code it starts at 0.
    pub phase: u32,
    pub unknown2: u8,
    /// normally same as parent filename
    pub script_name: Windows1252String16<'data>,
    /// Name of function containing fragment script
    pub fragment_name: Windows1252String16<'data>,
}
impl<'data> Parse<'data> for PhaseInfo<'data> {
    fn parse(data: &'data [u8]) -> PResult<Self> {
        let (data, unknown) = take(data, 1usize)?;
        let unknown = unknown[0];
        let (data, phase) = u32::parse(data)?;
        let (data, unknown2) = take(data, 1usize)?;
        let unknown2 = unknown2[0];
        let (data, script_name) = Windows1252String16::parse(data)?;
        let (data, fragment_name) = Windows1252String16::parse(data)?;
        Ok((
            data,
            PhaseInfo {
                unknown,
                phase,
                unknown2,
                script_name,
                fragment_name,
            },
        ))
    }
}
impl<'data> DataSize for PhaseInfo<'data> {
    fn data_size(&self) -> usize {
        self.unknown.data_size()
            + self.phase.data_size()
            + self.unknown2.data_size()
            + self.script_name.data_size()
            + self.fragment_name.data_size()
    }
}
impl<'data> Writable for PhaseInfo<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.unknown.write_to(w)?;
        self.phase.write_to(w)?;
        self.unknown2.write_to(w)?;
        self.script_name.write_to(w)?;
        self.fragment_name.write_to(w)
    }
}
