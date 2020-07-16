use super::{
    common::{
        full_string, lstring::LString, CommonRecordInfo, FormId, FromRecord, FromRecordError,
        GeneralRecord, Index, StaticTypeNamed, TypeNamed,
    },
    fields::{
        common::{
            item::{ICON, MICO, YNAM, ZNAM},
            write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE,
        },
        dest, edid, full, kwda, modl, obnd,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_static_data_size,
    impl_static_type_named, make_single_value_field,
    parse::{PResult, Parse, ParseError},
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use derive_more::From;
use full_string::FullString;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct AMMORecord<'data> {
    pub common: CommonRecordInfo,

    /// EDID
    pub editor_id_index: Index,
    /// OBND
    pub object_bounds_index: Index,
    /// FULL
    // Item name (localized string)
    pub item_name_index: Option<Index>,
    /// MODLCollection
    /// World model
    pub model_collection_index: Option<Index>,
    /// ICON
    /// Inventory image
    pub inventory_image_index: Option<Index>,
    /// MICO
    pub message_image_index: Option<Index>,
    /// DESTCollection
    pub destruction_collection_index: Option<Index>,
    /// YNAM
    pub pickup_sound_index: Option<Index>,
    /// ZNAM
    pub drop_sound_index: Option<Index>,
    /// DESC
    pub description_index: Option<Index>,
    /// KWDACollection
    pub keyword_collection_index: Option<Index>,
    /// DATA
    pub data_index: Option<Index>,
    /// ONAM
    pub short_name_index: Option<Index>,

    pub fields: Vec<AMMOField<'data>>,
}
impl<'data> FromRecord<'data> for AMMORecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut editor_id_index = None;
        let mut object_bounds_index = None;
        let mut item_name_index = None;
        let mut model_collection_index = None;
        let mut inventory_image_index = None;
        let mut message_image_index = None;
        let mut destruction_collection_index = None;
        let mut pickup_sound_index = None;
        let mut drop_sound_index = None;
        let mut description_index = None;
        let mut keyword_collection_index = None;
        let mut data_index = None;
        let mut short_name_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();

        while let Some(field) = field_iter.next() {
            match field.type_name.as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; editor_id_index),
                b"OBND" => collect_one!(obnd::OBND, field => fields; object_bounds_index),
                b"FULL" => collect_one!(full::FULL, field => fields; item_name_index),
                b"MODL" => {
                    collect_one_collection!(modl::MODL, modl::MODLCollection; field, field_iter => fields; model_collection_index)
                }
                b"ICON" => collect_one!(ICON, field => fields; inventory_image_index),
                b"MICO" => collect_one!(MICO, field => fields; message_image_index),
                b"DEST" => {
                    collect_one_collection!(dest::DEST, dest::DESTCollection; field, field_iter => fields; destruction_collection_index)
                }
                b"YNAM" => collect_one!(YNAM, field => fields; pickup_sound_index),
                b"ZNAM" => collect_one!(ZNAM, field => fields; drop_sound_index),
                b"DESC" => collect_one!(DESC, field => fields; description_index),
                b"KSIZ" => {
                    collect_one_collection!(kwda::KSIZ, kwda::KWDACollection; field, field_iter => fields; keyword_collection_index)
                }
                b"DATA" => collect_one!(DATA, field => fields; data_index),
                b"ONAM" => collect_one!(ONAM, field => fields; short_name_index),
                _ => fields.push(AMMOField::Unknown(field)),
            }
        }

        let editor_id_index = editor_id_index
            .ok_or_else(|| FromRecordError::ExpectedField(edid::EDID::static_type_name()))?;
        let object_bounds_index = object_bounds_index
            .ok_or_else(|| FromRecordError::ExpectedField(obnd::OBND::static_type_name()))?;

        Ok((
            &[],
            Self {
                common: record.common,
                editor_id_index,
                object_bounds_index,
                item_name_index,
                model_collection_index,
                inventory_image_index,
                message_image_index,
                destruction_collection_index,
                pickup_sound_index,
                drop_sound_index,
                description_index,
                keyword_collection_index,
                data_index,
                short_name_index,
                fields,
            },
        ))
    }
}
impl_static_type_named!(AMMORecord<'_>, b"AMMO");
impl<'data> DataSize for AMMORecord<'data> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
			4 + // data len
			self.common.data_size() +
			self.fields.data_size()
    }
}
impl<'data> Writable for AMMORecord<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert size fits within
        (self.fields.data_size() as u32).write_to(w)?;
        self.common.write_to(w)?;
        self.fields.write_to(w)
    }
}

#[derive(Debug, Clone, From)]
pub enum AMMOField<'data> {
    EDID(edid::EDID<'data>),
    OBND(obnd::OBND),
    FULL(full::FULL),
    MODLCollection(modl::MODLCollection<'data>),
    ICON(ICON<'data>),
    MICO(MICO<'data>),
    DESTCollection(dest::DESTCollection<'data>),
    YNAM(YNAM),
    ZNAM(ZNAM),
    DESC(DESC),
    KWDACollection(kwda::KWDACollection),
    DATA(DATA),
    ONAM(ONAM<'data>),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for AMMOField<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(
            AMMOField,
            self,
            [
                EDID,
                OBND,
                FULL,
                MODLCollection,
                ICON,
                MICO,
                DESTCollection,
                YNAM,
                ZNAM,
                DESC,
                KWDACollection,
                DATA,
                ONAM,
                Unknown
            ],
            x,
            { x.type_name() }
        )
    }
}
impl<'data> DataSize for AMMOField<'data> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            AMMOField,
            self,
            [
                EDID,
                OBND,
                FULL,
                MODLCollection,
                ICON,
                MICO,
                DESTCollection,
                YNAM,
                ZNAM,
                DESC,
                KWDACollection,
                DATA,
                ONAM,
                Unknown
            ],
            x,
            { x.data_size() }
        )
    }
}
impl<'data> Writable for AMMOField<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            AMMOField,
            self,
            [
                EDID,
                OBND,
                FULL,
                MODLCollection,
                ICON,
                MICO,
                DESTCollection,
                YNAM,
                ZNAM,
                DESC,
                KWDACollection,
                DATA,
                ONAM,
                Unknown
            ],
            x,
            { x.write_to(w) }
        )
    }
}

make_single_value_field!(
    /// Description
    [Debug, Copy, Clone, Eq, PartialEq],
    DESC,
    description,
    LString
);
impl FromField<'_> for DESC {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, description) = LString::parse(field.data)?;
        Ok((data, Self { description }))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DATALegendaryEdition {
    pub projectile_id: FormId,
    pub flags: DATAFlags,
    pub damage: f32,
    pub value: u32,
}
impl DATALegendaryEdition {
    pub fn parse(data: &[u8]) -> PResult<Self> {
        let (data, projectile_id) = FormId::parse(data)?;
        let (data, flags) = DATAFlags::parse(data)?;
        let (data, damage) = f32::parse(data)?;
        let (data, value) = u32::parse(data)?;
        Ok((
            data,
            Self {
                projectile_id,
                flags,
                damage,
                value,
            },
        ))
    }
}
impl_static_data_size!(
    DATALegendaryEdition,
    FormId::static_data_size() + // projectile id
    DATAFlags::static_data_size() + // flags
    f32::static_data_size() + // damage
    u32::static_data_size() // value
);
impl Writable for DATALegendaryEdition {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.projectile_id.write_to(w)?;
        self.flags.write_to(w)?;
        self.damage.write_to(w)?;
        self.value.write_to(w)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DATASpecialEdition {
    pub le: DATALegendaryEdition,
    pub weight: f32,
}
impl DATASpecialEdition {
    pub fn parse(data: &[u8]) -> PResult<Self> {
        let (data, le) = DATALegendaryEdition::parse(data)?;
        let (data, weight) = f32::parse(data)?;
        Ok((data, Self { le, weight }))
    }
}
impl_static_data_size!(
    DATASpecialEdition,
    DATALegendaryEdition::static_data_size() + f32::static_data_size()
);
impl Writable for DATASpecialEdition {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.le.write_to(w)?;
        self.weight.write_to(w)
    }
}

#[derive(Debug, Clone)]
pub enum DATA {
    /// Legendary edition version, 16 byte struct
    LE(DATALegendaryEdition),
    /// Special Edition, 20 byte struct
    SE(DATASpecialEdition),
}
impl FromField<'_> for DATA {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        if field.data.len() == 16 {
            let (data, le) = DATALegendaryEdition::parse(field.data)?;
            Ok((data, DATA::LE(le)))
        } else if field.data.len() == 20 {
            let (data, se) = DATASpecialEdition::parse(field.data)?;
            Ok((data, DATA::SE(se)))
        } else {
            Err(FromFieldError::ParseError(ParseError::InvalidByteCount {
                // expected: 16 | 20
                found: field.data.len(),
            }))
        }
    }
}
impl_static_type_named!(DATA, b"DATA");
impl DataSize for DATA {
    fn data_size(&self) -> usize {
        FIELDH_SIZE
            + match self {
                DATA::LE(x) => x.data_size(),
                DATA::SE(x) => x.data_size(),
            }
    }
}
impl Writable for DATA {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        match self {
            DATA::LE(x) => x.write_to(w),
            DATA::SE(x) => x.write_to(w),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DATAFlags {
    /// 0b001: Ignores normal weapon resistance
    /// 0b010: Non-playable
    /// 0b100: Non-Bolt (Arrow/Bolt)
    pub flags: u32,
}
impl DATAFlags {
    pub fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = u32::parse(data)?;
        Ok((data, Self { flags }))
    }

    pub fn ignores_weapon_resistance(&self) -> bool {
        (self.flags & 0b1) != 0
    }

    pub fn non_playable(&self) -> bool {
        (self.flags & 0b10) != 0
    }

    // TODO: I imagine these aren't completely correct for the fallout games..
    // TODO: I *think* this is correct, but i'm not 100%
    pub fn is_arrow(&self) -> bool {
        (self.flags & 0b100) != 0
    }

    pub fn is_bolt(&self) -> bool {
        (self.flags & 0b100) == 0
    }
}
impl_static_data_size!(DATAFlags, u32::static_data_size());
impl Writable for DATAFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

make_single_value_field!(
    /// UESP says this exists, but I haven't seen it in either Skyrim.esm or Dawnguard.esm
    [Debug, Clone], ONAM, short_name, FullString, 'data);
impl<'data> FromField<'data> for ONAM<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<'data, Self, FromFieldError> {
        let (data, short_name) = FullString::parse(field.data)?;
        Ok((data, Self { short_name }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;

    #[test]
    fn test_data() {
        let le = DATALegendaryEdition {
            projectile_id: FormId::new(0x4295aa52),
            flags: DATAFlags { flags: 0x0 },
            damage: 1.0,
            value: 500,
        };
        assert_size_output!(le);
        let data_le = DATA::LE(le.clone());
        assert_size_output!(data_le);

        let data_se = DATA::SE(DATASpecialEdition { le, weight: 0.1 });
        assert_size_output!(data_se);
    }
}
