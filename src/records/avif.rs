use super::{
    common::{
        get_field, CollectionList, CommonRecordInfo, ConversionError, FromRecord, FromRecordError,
        GeneralRecord, NullTerminatedString, StaticTypeNamed, TypeNamed,
    },
    fields::{
        common::{
            item, object, write_field_header, CollectField, FromField, FromFieldError,
            GeneralField, FIELDH_SIZE,
        },
        edid,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_from_field, impl_static_data_size,
    impl_static_type_named, make_field_getter, make_formid_field, make_single_value_field,
    parse::{PResult, Parse},
    util::{DataSize, Writable},
};
use derive_more::From;
use std::{
    convert::{TryFrom, TryInto},
    io::Write,
};

#[derive(Debug, Clone, PartialEq)]
pub struct AVIFRecord<'data> {
    pub common: CommonRecordInfo,
    pub fields: Vec<AVIFField<'data>>,
}
impl<'data> AVIFRecord<'data> {
    make_field_getter!(
        editor_id_index,
        editor_id,
        editor_id_mut,
        AVIFField::EDID,
        edid::EDID<'data>
    );

    make_field_getter!(
        optional: name_index,
        name,
        name_mut,
        AVIFField::FULL,
        object::FULL
    );

    make_field_getter!(
        description_index,
        description,
        description_mut,
        AVIFField::DESC,
        item::DESC
    );

    make_field_getter!(
        optional: abbreviation_index,
        abbreviation,
        abbreviation_mut,
        AVIFField::ANAM,
        ANAM<'data>
    );

    make_field_getter!(data_index, data, data_mut, AVIFField::CNAM, CNAM);

    // TODO: better name
    make_field_getter!(
        optional: av_skill_index,
        av_skill,
        av_skill_mut,
        AVIFField::AVSK,
        AVSK
    );

    // TODO: should we create this anyway so that there's always an empty list of perks since it doesn't affect writing?
    make_field_getter!(
        optional: perks_index,
        perks,
        perks_mut,
        AVIFField::PerkList,
        PerkList<'data>
    );
}
impl<'data> FromRecord<'data> for AVIFRecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut edid_index = None;
        let mut full_index = None;
        let mut desc_index = None;
        let mut anam_index = None;
        let mut cnam_index = None;
        let mut avsk_index = None;
        let mut perks_list_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();

        while let Some(field) = field_iter.next() {
            match field.type_name().as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; edid_index),
                b"FULL" => collect_one!(object::FULL, field => fields; full_index),
                b"DESC" => collect_one!(item::DESC, field => fields; desc_index),
                b"ANAM" => collect_one!(ANAM, field => fields; anam_index),
                b"CNAM" => collect_one!(CNAM, field => fields; cnam_index),
                b"AVSK" => collect_one!(AVSK, field => fields; avsk_index),
                b"PNAM" => {
                    collect_one_collection!(PNAM, PerkList; field, field_iter => fields; perks_list_index)
                }
                _ => fields.push(field.into()),
            }
        }

        if edid_index.is_none() {
            Err(FromRecordError::ExpectedField(
                edid::EDID::static_type_name(),
            ))
        } else if desc_index.is_none() {
            Err(FromRecordError::ExpectedField(
                item::DESC::static_type_name(),
            ))
        } else if cnam_index.is_none() {
            Err(FromRecordError::ExpectedField(CNAM::static_type_name()))
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
impl_static_type_named!(AVIFRecord<'_>, b"AVIF");
impl DataSize for AVIFRecord<'_> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
        4 + // data size len
        self.common.data_size() +
        self.fields.data_size()
    }
}
impl Writable for AVIFRecord<'_> {
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
pub enum AVIFField<'data> {
    EDID(edid::EDID<'data>),
    FULL(object::FULL),
    DESC(item::DESC),
    ANAM(ANAM<'data>),
    CNAM(CNAM),
    AVSK(AVSK),
    PerkList(PerkList<'data>),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for AVIFField<'data> {
    fn type_name(&self) -> &'data bstr::BStr {
        dispatch_all!(
            AVIFField,
            self,
            [EDID, FULL, DESC, ANAM, CNAM, AVSK, PerkList, Unknown],
            x,
            { x.type_name() }
        )
    }
}
impl DataSize for AVIFField<'_> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            AVIFField,
            self,
            [EDID, FULL, DESC, ANAM, CNAM, AVSK, PerkList, Unknown],
            x,
            { x.data_size() }
        )
    }
}
impl Writable for AVIFField<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            AVIFField,
            self,
            [EDID, FULL, DESC, ANAM, CNAM, AVSK, PerkList, Unknown],
            x,
            { x.write_to(w) }
        )
    }
}

make_single_value_field!(
    /// Abbreviation
    [Debug, Clone, Eq, PartialEq],
    ANAM,
    abbreviation,
    NullTerminatedString,
    'data
);
impl_from_field!(ANAM, 'data, [abbreviation: NullTerminatedString]);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum SkillCategory {
    None = 0,
    Combat = 1,
    Magic = 2,
    Stealth = 3,
}
impl SkillCategory {
    pub fn code(&self) -> u32 {
        *self as u32
    }
}
impl Parse<'_> for SkillCategory {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, value) = u32::parse(data)?;
        Ok((data, value.try_into()?))
    }
}
impl_static_data_size!(SkillCategory, u32::static_data_size());
impl Writable for SkillCategory {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.code().write_to(w)
    }
}
impl TryFrom<u32> for SkillCategory {
    type Error = ConversionError<u32>;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => SkillCategory::None,
            1 => SkillCategory::Combat,
            2 => SkillCategory::Magic,
            3 => SkillCategory::Stealth,
            _ => return Err(ConversionError::InvalidEnumerationValue(value)),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CNAM {
    SkillCategory(SkillCategory),
    /// UESP things it's what is inside AVSK field
    Unknown(u32),
}
impl FromField<'_> for CNAM {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, value) = u32::parse(field.data)?;
        let cnam = match SkillCategory::try_from(value) {
            Ok(value) => CNAM::SkillCategory(value),
            Err(_) => CNAM::Unknown(value),
        };
        Ok((data, cnam))
    }
}
impl_static_type_named!(CNAM, b"CNAM");
impl_static_data_size!(CNAM, FIELDH_SIZE + u32::static_data_size());
impl Writable for CNAM {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        match self {
            CNAM::SkillCategory(skill) => skill.write_to(w),
            CNAM::Unknown(x) => x.write_to(w),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AVSK {
    pub skill_use_multiplier: f32,
    pub skill_use_offset: f32,
    pub skill_improve_multiplier: f32,
    pub skill_improve_offset: f32,
}
impl_from_field!(
    AVSK,
    [
        skill_use_multiplier: f32,
        skill_use_offset: f32,
        skill_improve_multiplier: f32,
        skill_improve_offset: f32
    ]
);
impl_static_type_named!(AVSK, b"AVSK");
impl_static_data_size!(AVSK, FIELDH_SIZE + (f32::static_data_size() * 4));
impl Writable for AVSK {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.skill_use_multiplier.write_to(w)?;
        self.skill_use_offset.write_to(w)?;
        self.skill_improve_multiplier.write_to(w)?;
        self.skill_improve_offset.write_to(w)
    }
}

pub type PerkList<'unused> = CollectionList<'unused, Perk>;
#[derive(Debug, Clone, PartialEq)]
pub struct Perk {
    pub perk: PNAM,
    pub flag: FNAM,
    pub x: XNAM,
    pub y: YNAM,
    pub horizontal: HNAM,
    pub vertical: VNAM,
    pub skill: SNAM,
    pub connecting: Vec<PerkCNAM>,
    pub id: INAM,
}
impl<'data> CollectField<'data, PNAM> for Perk {
    fn collect<I>(
        perk: PNAM,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<Self, FromFieldError<'data>>
    where
        I: std::iter::Iterator<Item = GeneralField<'data>>,
    {
        let (_, flag) = get_field(field_iter, FNAM::static_type_name())?;
        let flag =
            flag.ok_or_else(|| FromFieldError::ExpectedSpecificField(FNAM::static_type_name()))?;
        let (_, x) = get_field(field_iter, XNAM::static_type_name())?;
        let x = x.ok_or_else(|| FromFieldError::ExpectedSpecificField(XNAM::static_type_name()))?;
        let (_, y) = get_field(field_iter, YNAM::static_type_name())?;
        let y = y.ok_or_else(|| FromFieldError::ExpectedSpecificField(YNAM::static_type_name()))?;
        let (_, horizontal) = get_field(field_iter, HNAM::static_type_name())?;
        let horizontal = horizontal
            .ok_or_else(|| FromFieldError::ExpectedSpecificField(HNAM::static_type_name()))?;
        let (_, vertical) = get_field(field_iter, VNAM::static_type_name())?;
        let vertical = vertical
            .ok_or_else(|| FromFieldError::ExpectedSpecificField(VNAM::static_type_name()))?;
        let (_, skill) = get_field(field_iter, SNAM::static_type_name())?;
        let skill =
            skill.ok_or_else(|| FromFieldError::ExpectedSpecificField(SNAM::static_type_name()))?;

        let mut connecting = Vec::new();
        loop {
            let (_, connected) = get_field(field_iter, CNAM::static_type_name())?;
            match connected {
                Some(connected) => connecting.push(connected),
                None => break,
            }
        }

        let (_, id) = get_field(field_iter, INAM::static_type_name())?;
        let id =
            id.ok_or_else(|| FromFieldError::ExpectedSpecificField(INAM::static_type_name()))?;

        Ok((
            &[],
            Self {
                perk,
                flag,
                x,
                y,
                horizontal,
                vertical,
                skill,
                connecting,
                id,
            },
        ))
    }
}
impl_static_type_named!(Perk, PNAM::static_type_name());
impl DataSize for Perk {
    fn data_size(&self) -> usize {
        self.perk.data_size()
            + self.flag.data_size()
            + self.x.data_size()
            + self.y.data_size()
            + self.horizontal.data_size()
            + self.vertical.data_size()
            + self.skill.data_size()
            + self.connecting.data_size()
            + self.id.data_size()
    }
}
impl Writable for Perk {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.perk.write_to(w)?;
        self.flag.write_to(w)?;
        self.x.write_to(w)?;
        self.y.write_to(w)?;
        self.horizontal.write_to(w)?;
        self.vertical.write_to(w)?;
        self.skill.write_to(w)?;
        self.connecting.write_to(w)?;
        self.id.write_to(w)
    }
}

// TODO: make th is be an option (and special handle writing)?
make_formid_field!(
    // ->PERK, or 0 for the first
    PNAM
);
make_single_value_field!(
    /// UESP isn't really sure what it is.
    /// Most common values are 0 or 1
    /// but first peek of a tree usually has large values.
    [Debug, Copy, Clone, Eq, PartialEq],
    FNAM,
    flag,
    u32
);
impl_from_field!(FNAM, [flag: u32]);

make_single_value_field!(
    /// X coord within perk grid
    [Debug, Copy, Clone, Eq, PartialEq],
    XNAM,
    x_coord,
    i32
);
impl_from_field!(XNAM, [x_coord: i32]);
make_single_value_field!(
    /// Y coord within perk grid
    [Debug, Copy, Clone, Eq, PartialEq],
    YNAM,
    y_coord,
    i32
);
impl_from_field!(YNAM, [y_coord: i32]);

make_single_value_field!(
    /// Horizontal position of the skill within xnam/ynam grid. (Offset?)
    [Debug, Copy, Clone, PartialEq],
    HNAM,
    horiz_position,
    f32
);
impl_from_field!(HNAM, [horiz_position: f32]);
make_single_value_field!(
    /// Vertical position of the skill within xnam/ynam grid. (Offset?)
    [Debug, Copy, Clone, PartialEq],
    VNAM,
    vert_position,
    f32
);
impl_from_field!(VNAM, [vert_position: f32]);

make_formid_field!(
    /// ->AVIF, usually same as parent. Present even if CNAM is not
    SNAM
);

// TODO: it would be better for this to not have an insane submodule. Either expand the macro manually, or make the macro support custom typename.
mod sub {
    use crate::{impl_from_field, make_single_value_field};

    make_single_value_field!(
        [Debug, Copy, Clone, Eq, PartialEq],
        CNAM,
        /// ->INAM of destination perk for each line coming from box.
        id,
        u32
    );
    impl_from_field!(CNAM, [id: u32]);
}
use sub::CNAM as PerkCNAM;

make_single_value_field!(
    [Debug, Copy, Clone, Eq, PartialEq],
    INAM,
    /// Unique id for perk box. Doesn't have to be sequential.
    id,
    u32
);
impl_from_field!(INAM, [id: u32]);
