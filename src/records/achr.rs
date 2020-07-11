use super::{
    common::{self, CommonRecordInfo, GeneralRecord, Index},
    fields::{
        common::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE},
        edid, vmad,
    },
};
use crate::{
    collect_many, collect_one, dispatch_all, make_empty_field, make_formid_field,
    make_single_value_field,
    parse::{le_f32, le_u32, take, PResult, ParseError},
    util::{byte, DataSize, Position3, StaticDataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use common::{FormId, FromRecord, FromRecordError, TypeNamed};
use derive_more::From;
use std::io::Write;

// TODO: this uses up a good amount of memory to hold all these indices. We could turn most of these into functions, and simply verify at parse time that there isn't multiple.

/// Holds information about actors
/// It is a specific NPC at a certain location, possibly at a time (possibly triggered by scripts)
/// doing things.
#[derive(Debug, Clone)]
pub struct ACHRRecord<'data> {
    pub common: CommonRecordInfo,
    /// EDID
    pub editor_id_index: Option<Index>,
    /// VMAD
    script_index: Option<Index>,
    /// NAME. formid of base NPC_
    base_npc_index: Index,
    /// XEZN. Encounter Zone. Formid to ECZN
    encounter_zone_index: Option<Index>,

    // These four are part of patrol data, which is uncommon.
    /// XPRD. float
    patrol_idle_index: Option<Index>,
    /// XPPA. 0-length.
    /// Maybe some form of marker?
    unknown_xppa_index: Option<Index>,
    /// INAM. formid
    unknown_inam_index: Option<Index>,
    /// PDTO.
    topic_data_index: Option<Index>,

    /// XRGD. Unknown if this is actually ragdoll info. UESP theorizes it is.
    ragdoll_index: Option<Index>,
    /// XRGB
    unknown_xrgb: Option<Index>,
    /// XLCM
    leveled_creature_data: Option<Index>,
    /// XAPD
    activation_parent_flags_index: Option<Index>,
    /// XAPR
    activate_parent_index: Option<Index>,
    /// XLRT* formids to LCRT
    location_ref_type_indices: Vec<Index>,
    /// XHOR. Rare
    horse_id_index: Option<Index>,
    /// XESP
    enable_parent_index: Option<Index>,
    /// XOWN
    owner_index: Option<Index>,
    /// XLCN
    location_index: Option<Index>,
    /// XLKR. maybe right name?
    location_route_index: Option<Index>,
    /// XIS2. Not found in esms, zero length, present if "Ignored By Sandbox" checked
    unknown_xis2_index: Option<Index>,
    /// XLRL. Not found in esms. Added by CK 1.8 when edited.
    unknown_xlrl_index: Option<Index>,
    /// XSCL
    scale_index: Option<Index>,
    /// DATA
    coords_index: Index,

    fields: Vec<ACHRField<'data>>,
}
impl<'data> TypeNamed<'static> for ACHRRecord<'data> {
    fn type_name(&self) -> &'static BStr {
        b"ACHR".as_bstr()
    }
}
impl<'data> FromRecord<'data> for ACHRRecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError> {
        let mut editor_id_index: Option<Index> = None;
        let mut script_index: Option<Index> = None;
        let mut base_npc_index: Option<Index> = None; // has to have value
        let mut encounter_zone_index: Option<Index> = None;
        let mut patrol_idle_index: Option<Index> = None;
        let mut unknown_xppa_index: Option<Index> = None;
        let mut unknown_inam_index: Option<Index> = None;
        let mut topic_data_index: Option<Index> = None;
        let mut ragdoll_index: Option<Index> = None;
        let mut unknown_xrgb: Option<Index> = None;
        let mut leveled_creature_data: Option<Index> = None;
        let mut activation_parent_flags_index: Option<Index> = None;
        let mut activate_parent_index: Option<Index> = None;
        let mut location_ref_type_indices: Vec<Index> = Vec::new();
        let mut horse_id_index: Option<Index> = None;
        let mut enable_parent_index: Option<Index> = None;
        let mut owner_index: Option<Index> = None;
        let mut location_index: Option<Index> = None;
        let mut location_route_index: Option<Index> = None;
        let mut unknown_xis2_index: Option<Index> = None;
        let mut unknown_xlrl_index: Option<Index> = None;
        let mut scale_index: Option<Index> = None;
        let mut coords_index: Option<Index> = None; // has to have value

        let mut fields = Vec::new();

        for field in record.fields {
            match field.type_name.as_ref() {
                b"EDID" => collect_one!(EDID, field => fields; editor_id_index),
                b"VMAD" => {
                    collect_one!(vmad::VMAD<'data, vmad::NoFragments>, field => fields; script_index)
                }
                b"NAME" => collect_one!(NAME, field => fields; base_npc_index),
                b"XEZN" => collect_one!(XEZN, field => fields; encounter_zone_index),
                b"XPRD" => collect_one!(XPRD, field => fields; patrol_idle_index),
                b"XPPA" => collect_one!(XPPA, field => fields; unknown_xppa_index),
                b"INAM" => collect_one!(INAM, field => fields; unknown_inam_index),
                b"PDTO" => collect_one!(PDTO, field => fields; topic_data_index),
                b"XRGD" => collect_one!(XRGD, field => fields; ragdoll_index),
                b"XRGB" => collect_one!(XRGB, field => fields; unknown_xrgb),
                b"XLCM" => collect_one!(XLCM, field => fields; leveled_creature_data),
                b"XAPD" => collect_one!(XAPD, field => fields; activation_parent_flags_index),
                b"XAPR" => collect_one!(XAPR, field => fields; activate_parent_index),
                b"XLRT" => collect_many!(XLRT, field => fields; location_ref_type_indices),
                b"XHOR" => collect_one!(XHOR, field => fields; horse_id_index),
                b"XESP" => collect_one!(XESP, field => fields; enable_parent_index),
                b"XOWN" => collect_one!(XOWN, field => fields; owner_index),
                b"XLCN" => collect_one!(XLCN, field => fields; location_index),
                b"XLKR" => collect_one!(XLKR, field => fields; location_route_index),
                b"XIS2" => collect_one!(XIS2, field => fields; unknown_xis2_index),
                b"XLRL" => collect_one!(XLRL, field => fields; unknown_xlrl_index),
                b"XSCL" => collect_one!(XSCL, field => fields; scale_index),
                b"DATA" => collect_one!(DATA, field => fields; coords_index),
                _ => fields.push(ACHRField::Unknown(field)),
            }
        }

        let base_npc_index = base_npc_index.expect("Expected base npc index in ACHR record.");
        let coords_index = coords_index.expect("Expected coordinates in ACHR record.");

        Ok((
            &[],
            ACHRRecord {
                common: record.common,
                editor_id_index,
                script_index,
                base_npc_index,
                encounter_zone_index,
                patrol_idle_index,
                unknown_xppa_index,
                unknown_inam_index,
                topic_data_index,
                ragdoll_index,
                unknown_xrgb,
                leveled_creature_data,
                activation_parent_flags_index,
                activate_parent_index,
                location_ref_type_indices,
                horse_id_index,
                enable_parent_index,
                owner_index,
                location_index,
                location_route_index,
                unknown_xis2_index,
                unknown_xlrl_index,
                scale_index,
                coords_index,
                fields,
            },
        ))
    }
}
impl<'data> ACHRRecord<'data> {
    pub fn fields_size(&self) -> usize {
        self.fields.iter().fold(0, |acc, x| acc + x.data_size())
    }
}
impl<'data> Writable for ACHRRecord<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert that size fits within a u32
        (self.fields_size() as u32).write_to(w)?;
        self.common.write_to(w)?;
        for field in self.fields.iter() {
            field.write_to(w)?;
        }
        Ok(())
    }
}
impl<'data> DataSize for ACHRRecord<'data> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
            4 + // data size
            self.common.data_size() +
            self.fields_size()
    }
}

#[derive(Debug, Clone, From)]
pub enum ACHRField<'data> {
    EDID(EDID<'data>),
    VMAD(vmad::VMAD<'data, vmad::NoFragments>),
    NAME(NAME),
    XEZN(XEZN),
    XPRD(XPRD),
    XPPA(XPPA),
    INAM(INAM),
    PDTO(PDTO<'data>),
    XRGD(XRGD<'data>),
    XRGB(XRGB),
    XLCM(XLCM),
    XAPD(XAPD),
    XAPR(XAPR),
    XLRT(XLRT),
    XHOR(XHOR),
    XESP(XESP),
    XOWN(XOWN),
    XLCN(XLCN),
    XLKR(XLKR),
    XIS2(XIS2),
    XLRL(XLRL),
    XSCL(XSCL),
    DATA(DATA),

    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for ACHRField<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(
            ACHRField,
            self,
            [
                EDID, VMAD, NAME, XEZN, XPRD, XPPA, INAM, PDTO, XRGD, XRGB, XLCM, XAPD, XAPR, XLRT,
                XHOR, XESP, XOWN, XLCN, XLKR, XIS2, XLRL, XSCL, DATA, Unknown
            ],
            x,
            { x.type_name() }
        )
    }
}
impl<'data> Writable for ACHRField<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            ACHRField,
            self,
            [
                EDID, VMAD, NAME, XEZN, XPRD, XPPA, INAM, PDTO, XRGD, XRGB, XLCM, XAPD, XAPR, XLRT,
                XHOR, XESP, XOWN, XLCN, XLKR, XIS2, XLRL, XSCL, DATA, Unknown
            ],
            x,
            { x.write_to(w) }
        )
    }
}
impl<'data> DataSize for ACHRField<'data> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            ACHRField,
            self,
            [
                EDID, VMAD, NAME, XEZN, XPRD, XPPA, INAM, PDTO, XRGD, XRGB, XLCM, XAPD, XAPR, XLRT,
                XHOR, XESP, XOWN, XLCN, XLKR, XIS2, XLRL, XSCL, DATA, Unknown
            ],
            x,
            { x.data_size() }
        )
    }
}

pub type EDID<'data> = edid::EDID<'data>;

make_formid_field!(NAME);
make_formid_field!(XEZN);

make_single_value_field!([Debug, Copy, Clone, PartialEq], XPRD, idle_time, f32);
impl FromField<'_> for XPRD {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, idle_time) = le_f32(field.data)?;
        Ok((data, XPRD { idle_time }))
    }
}

make_empty_field!(XPPA);

make_formid_field!(INAM);

make_single_value_field!([Debug, Clone], PDTO, topic_type, TopicType, 'data);
impl<'data> FromField<'data> for PDTO<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, topic_type) = le_u32(field.data)?;
        let (data, topic_type) = TopicType::parse(data, topic_type)?;
        Ok((data, PDTO { topic_type }))
    }
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TopicType<'data> {
    /// 0
    Ref(FormId),
    // TODO: should this be an array of 4 bchars?
    /// 4 bcharacters
    /// 1
    Subtype(&'data BStr),
}
impl<'data> TopicType<'data> {
    pub fn parse(data: &'data [u8], topic_type: u32) -> PResult<Self> {
        match topic_type {
            0 => {
                let (data, formid) = FormId::parse(data)?;
                Ok((data, TopicType::Ref(formid)))
            }
            1 => {
                let (data, text) = take(data, 4)?;
                let text = text.as_bstr();
                Ok((data, TopicType::Subtype(text)))
            }
            _ => Err(ParseError::InvalidEnumerationValue),
        }
    }

    /// Returns the value that signifies it's type
    pub fn code(&self) -> u32 {
        match *self {
            TopicType::Ref(_) => 0,
            TopicType::Subtype(_) => 1,
        }
    }

    /// Returns it's value in bytes
    pub fn label(&self) -> [u8; 4] {
        match *self {
            TopicType::Ref(formid) => formid.as_bytes(),
            TopicType::Subtype(string) => byte::as_4_bytes(string.as_bytes()),
        }
    }
}
impl<'data> StaticDataSize for TopicType<'data> {
    fn static_data_size() -> usize {
        u32::static_data_size() + // type integer
            FormId::static_data_size() // u32 size (formid and 4 char bstr)
    }
}
impl<'data> Writable for TopicType<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.code().write_to(w)?;
        // TODO: assert that subtype is only 4 chars
        match self {
            TopicType::Ref(f) => f.write_to(w),
            TopicType::Subtype(s) => s.write_to(w),
        }
    }
}

make_single_value_field!(
    [Debug, Clone],
    XRGD,
    /// TODO: figure out how this is structured
    data,
    refer [u8],
    'data
);
impl<'data> FromField<'data> for XRGD<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        Ok((&[], Self { data: field.data }))
    }
}

#[derive(Debug, Clone)]
pub struct XRGB {
    // TODO: figure out what this is
    // It's named XRGB, and (maybe) 3 floats, so it's potentially a color, but for what? (check if it always fits in 0.0-1.0)
    pub data: [f32; 3],
}
impl TypeNamed<'static> for XRGB {
    fn type_name(&self) -> &'static BStr {
        b"XRGB".as_bstr()
    }
}
impl FromField<'_> for XRGB {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, f1) = le_f32(field.data)?;
        let (data, f2) = le_f32(data)?;
        let (data, f3) = le_f32(data)?;
        Ok((data, XRGB { data: [f1, f2, f3] }))
    }
}
impl StaticDataSize for XRGB {
    fn static_data_size() -> usize {
        FIELDH_SIZE + (f32::static_data_size() * 3)
    }
}
impl Writable for XRGB {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.data[0].write_to(w)?;
        self.data[1].write_to(w)?;
        self.data[2].write_to(w)
    }
}

make_single_value_field!(
    /// Leveled creature data
    [Debug, Copy, Clone, Eq, PartialEq],
    XLCM,
    level_mod,
    LevelModifier
);
impl FromField<'_> for XLCM {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, modifier) = le_u32(field.data)?;
        let modifier = match LevelModifier::from_u32(modifier) {
            Some(x) => x,
            None => return Err(ParseError::InvalidEnumerationValue.into()),
        };
        Ok((
            data,
            XLCM {
                level_mod: modifier,
            },
        ))
    }
}

/// Repr u32
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LevelModifier {
    /// 0
    Easy,
    /// 1
    Medium,
    /// 2
    Hard,
    /// 3
    VeryHard,
    // TODO: there is a None field, what is it's value?
}
impl LevelModifier {
    pub fn from_u32(modifier: u32) -> Option<LevelModifier> {
        match modifier {
            0 => Some(LevelModifier::Easy),
            1 => Some(LevelModifier::Medium),
            2 => Some(LevelModifier::Hard),
            3 => Some(LevelModifier::VeryHard),
            _ => None,
        }
    }

    pub fn code(&self) -> u32 {
        match self {
            LevelModifier::Easy => 0,
            LevelModifier::Medium => 1,
            LevelModifier::Hard => 2,
            LevelModifier::VeryHard => 3,
        }
    }
}
impl StaticDataSize for LevelModifier {
    fn static_data_size() -> usize {
        u32::static_data_size()
    }
}
impl Writable for LevelModifier {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.code().write_to(w)
    }
}

make_single_value_field!([Debug, Copy, Clone, Eq, PartialEq], XAPD, flags, XAPDFlags);
impl FromField<'_> for XAPD {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, flags) = take(field.data, 1)?;
        Ok((
            data,
            XAPD {
                flags: XAPDFlags::new(flags[0]),
            },
        ))
    }
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct XAPDFlags {
    /// 0b1: parent activate only
    pub flags: u8,
}
impl XAPDFlags {
    pub fn new(flags: u8) -> XAPDFlags {
        XAPDFlags { flags }
    }

    pub fn is_parent_activate_only(&self) -> bool {
        (self.flags & 0b1) != 0
    }
}
impl StaticDataSize for XAPDFlags {
    fn static_data_size() -> usize {
        u8::static_data_size()
    }
}
impl Writable for XAPDFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

/// activate parent
#[derive(Debug, Clone)]
pub struct XAPR {
    /// -> REFR, which is usually a STAT
    formid: FormId,
    delay: f32,
}
impl TypeNamed<'static> for XAPR {
    fn type_name(&self) -> &'static BStr {
        b"XAPR".as_bstr()
    }
}
impl FromField<'_> for XAPR {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, formid) = FormId::parse(field.data)?;
        let (data, delay) = le_f32(data)?;
        Ok((data, XAPR { formid, delay }))
    }
}
impl StaticDataSize for XAPR {
    fn static_data_size() -> usize {
        FIELDH_SIZE +
            FormId::static_data_size() + // formid
            f32::static_data_size() // delay
    }
}
impl Writable for XAPR {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.formid.write_to(w)?;
        self.delay.write_to(w)
    }
}

make_formid_field!(
    /// ->LCRT
    XLRT
);

make_formid_field!(
    /// Horse
    XHOR
);

#[derive(Debug, Clone)]
pub struct XESP {
    /// Parent reference. (Object to take enable state from)
    parent: FormId,
    flags: XESPFlags,
}
impl TypeNamed<'static> for XESP {
    fn type_name(&self) -> &'static BStr {
        b"XESP".as_bstr()
    }
}
impl FromField<'_> for XESP {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, parent) = FormId::parse(field.data)?;
        let (data, flags) = le_u32(data)?;
        let flags = XESPFlags::new(flags);
        Ok((data, XESP { parent, flags }))
    }
}
impl StaticDataSize for XESP {
    fn static_data_size() -> usize {
        FIELDH_SIZE +
            FormId::static_data_size() + // parent
            XESPFlags::static_data_size() // flags
    }
}
impl Writable for XESP {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.parent.write_to(w)?;
        self.flags.write_to(w)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct XESPFlags {
    /// 0b01: Set enable state to parent's opposite
    /// 0b10: pop in
    pub flags: u32,
}
impl XESPFlags {
    pub fn new(flags: u32) -> Self {
        Self { flags }
    }

    pub fn is_set_enable_state_opposite(&self) -> bool {
        (self.flags & 0b1) != 0
    }

    pub fn is_pop_in(&self) -> bool {
        (self.flags & 0b10) != 0
    }
}
impl StaticDataSize for XESPFlags {
    fn static_data_size() -> usize {
        u32::static_data_size()
    }
}
impl Writable for XESPFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

make_formid_field!(
    /// Usually ->FACT, but horse/dog has ->NPC_
    XOWN
);

make_formid_field!(
    /// -> LCTN
    XLCN
);

#[derive(Debug, Clone)]
pub struct XLKR {
    /// 0 or ->KYWD (usually LinkCarryStart/LinkCarryEnd)
    keyword: FormId,
    /// ->REFR to STAT or FURN
    /// TODO: better name: target?
    reference: FormId,
}
impl TypeNamed<'static> for XLKR {
    fn type_name(&self) -> &'static BStr {
        b"XLKR".as_bstr()
    }
}
impl FromField<'_> for XLKR {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, keyword) = FormId::parse(field.data)?;
        let (data, reference) = FormId::parse(data)?;
        Ok((data, XLKR { keyword, reference }))
    }
}
impl StaticDataSize for XLKR {
    fn static_data_size() -> usize {
        FIELDH_SIZE +
            FormId::static_data_size() + // keyword
            FormId::static_data_size() // reference
    }
}
impl Writable for XLKR {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.keyword.write_to(w)?;
        self.reference.write_to(w)
    }
}

make_empty_field!(
    /// Not found in esms, zero length, present if "Ignored by Sandbox is checked"
    XIS2
);

make_formid_field!(XLRL);

make_single_value_field!([Debug, Copy, Clone, PartialEq], XSCL, scale, f32);
impl FromField<'_> for XSCL {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, scale) = le_f32(field.data)?;
        Ok((data, XSCL { scale }))
    }
}

#[derive(Debug, Clone)]
pub struct DATA {
    /// TODO: is this correct name?
    position: Position3<f32>,
    /// TODO: is this correct name?
    /// in radians
    rotation: Position3<f32>,
}
impl TypeNamed<'static> for DATA {
    fn type_name(&self) -> &'static BStr {
        b"DATA".as_bstr()
    }
}
impl FromField<'_> for DATA {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, x) = le_f32(field.data)?;
        let (data, y) = le_f32(data)?;
        let (data, z) = le_f32(data)?;
        let (data, rx) = le_f32(data)?;
        let (data, ry) = le_f32(data)?;
        let (data, rz) = le_f32(data)?;
        Ok((
            data,
            DATA {
                position: Position3::new(x, y, z),
                rotation: Position3::new(rx, ry, rz),
            },
        ))
    }
}
impl StaticDataSize for DATA {
    fn static_data_size() -> usize {
        FIELDH_SIZE +
            Position3::<f32>::static_data_size() + // position
            Position3::<f32>::static_data_size() // rotation
    }
}
impl Writable for DATA {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.position.write_to(w)?;
        self.rotation.write_to(w)
    }
}
