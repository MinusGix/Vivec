use super::{
    common::{
        get_field, CommonRecordInfo, FormId, FromRecord, FromRecordError, GeneralRecord, Index,
        StaticTypeNamed, TypeNamed,
    },
    fields::{
        common::{item, object, write_field_header, FromFieldError, GeneralField, FIELDH_SIZE},
        ctda, edid, kwda, modl, obnd,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_from_field, impl_static_data_size,
    impl_static_type_named, make_formid_field, make_single_value_field,
    parse::{PResult, Parse},
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use derive_more::From;
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct ALCHRecord<'data> {
    pub common: CommonRecordInfo,
    /// EDID
    pub editor_id_index: Index,
    /// OBND
    pub object_bounds_index: Index,
    /// FULL. In game name
    pub full_name_index: Option<Index>,
    /// KWDACollection
    pub keyword_collection_index: Option<Index>,
    /// MODLCollection
    pub model_collection_index: Option<Index>,
    /// ICON
    pub icon_index: Option<Index>,
    /// MICO
    pub message_icon_index: Option<Index>,
    /// YNAM
    pub pickup_sound_index: Option<Index>,
    /// ZNAM
    pub drop_sound_index: Option<Index>,
    /// DATA
    pub weight_index: Index,
    /// EnchantedEffectCollection
    pub enchanted_effect_collection_index: Index,

    pub fields: Vec<ALCHField<'data>>,
}
impl<'data> FromRecord<'data> for ALCHRecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut editor_id_index = None;
        let mut object_bounds_index = None;
        let mut full_name_index = None;
        let mut keyword_collection_index = None;
        let mut model_collection_index = None;
        let mut icon_index = None;
        let mut message_icon_index = None;
        let mut pickup_sound_index = None;
        let mut drop_sound_index = None;
        let mut weight_index = None;
        let mut enchanted_effect_collection_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();

        while let Some(field) = field_iter.next() {
            match field.type_name.as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; editor_id_index),
                b"OBND" => collect_one!(obnd::OBND, field => fields; object_bounds_index),
                b"FULL" => collect_one!(object::FULL, field => fields; full_name_index),
                b"KSIZ" => {
                    collect_one_collection!(kwda::KSIZ, kwda::KWDACollection; field, field_iter => fields; keyword_collection_index)
                }
                b"MODL" => {
                    collect_one_collection!(modl::MODL, modl::MODLCollection; field, field_iter => fields; model_collection_index)
                }
                b"ICON" => collect_one!(item::ICON, field => fields; icon_index),
                b"MICO" => collect_one!(item::MICO, field => fields; message_icon_index),
                b"YNAM" => collect_one!(item::YNAM, field => fields; pickup_sound_index),
                b"ZNAM" => collect_one!(item::ZNAM, field => fields; drop_sound_index),
                b"DATA" => collect_one!(DATA, field => fields; weight_index),
                b"ENIT" => {
                    collect_one_collection!(ENIT, EnchantedEffectCollection; field, field_iter => fields; enchanted_effect_collection_index)
                }
                _ => fields.push(ALCHField::Unknown(field)),
            }
        }

        let editor_id_index = editor_id_index
            .ok_or_else(|| FromRecordError::ExpectedField(edid::EDID::static_type_name()))?;
        let object_bounds_index = object_bounds_index
            .ok_or_else(|| FromRecordError::ExpectedField(obnd::OBND::static_type_name()))?;
        let weight_index =
            weight_index.ok_or_else(|| FromRecordError::ExpectedField(DATA::static_type_name()))?;
        let enchanted_effect_collection_index =
            enchanted_effect_collection_index.ok_or_else(|| {
                FromRecordError::ExpectedField(EnchantedEffectCollection::static_type_name())
            })?;

        Ok((
            &[],
            ALCHRecord {
                common: record.common,
                editor_id_index,
                object_bounds_index,
                full_name_index,
                keyword_collection_index,
                model_collection_index,
                icon_index,
                message_icon_index,
                pickup_sound_index,
                drop_sound_index,
                weight_index,
                enchanted_effect_collection_index,
                fields,
            },
        ))
    }
}
impl_static_type_named!(ALCHRecord<'_>, b"ALCH");
impl<'data> DataSize for ALCHRecord<'data> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
			4 + // data len
            self.common.data_size() +
            self.fields.data_size()
    }
}
impl<'data> Writable for ALCHRecord<'data> {
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

#[derive(Debug, Clone, PartialEq, From)]
pub enum ALCHField<'data> {
    EDID(edid::EDID<'data>),
    OBND(obnd::OBND),
    FULL(object::FULL),
    // TODO: note: UESP says that there may be more than one KWDA entry? I didn't see that in a quick skim through a handful of skyrim entries
    KWDACollection(kwda::KWDACollection),
    MODLCollection(modl::MODLCollection<'data>),
    ICON(item::ICON<'data>),
    MICO(item::MICO<'data>),
    YNAM(item::YNAM),
    ZNAM(item::ZNAM),
    DATA(DATA),
    EnchantedEffectCollection(EnchantedEffectCollection),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for ALCHField<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(
            ALCHField,
            self,
            [
                EDID,
                OBND,
                FULL,
                KWDACollection,
                MODLCollection,
                ICON,
                MICO,
                YNAM,
                ZNAM,
                DATA,
                EnchantedEffectCollection,
                Unknown
            ],
            x,
            { x.type_name() }
        )
    }
}
impl<'data> DataSize for ALCHField<'data> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            ALCHField,
            self,
            [
                EDID,
                OBND,
                FULL,
                KWDACollection,
                MODLCollection,
                ICON,
                MICO,
                YNAM,
                ZNAM,
                DATA,
                EnchantedEffectCollection,
                Unknown
            ],
            x,
            { x.data_size() }
        )
    }
}
impl<'data> Writable for ALCHField<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            ALCHField,
            self,
            [
                EDID,
                OBND,
                FULL,
                KWDACollection,
                MODLCollection,
                ICON,
                MICO,
                YNAM,
                ZNAM,
                DATA,
                EnchantedEffectCollection,
                Unknown
            ],
            x,
            { x.write_to(w) }
        )
    }
}

make_single_value_field!(
    /// Message icon filename
    [Debug, Copy, Clone, PartialEq, PartialOrd],
    DATA,
    weight,
    f32
);
impl_from_field!(DATA, [weight: f32]);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ENIT {
    /// ?
    pub potion_value: u32,
    pub flags: ENITFlags,
    pub addiction: FormId,
    pub addiction_chance: u32,
    /// ->SNDR
    pub use_sound: FormId,
}
impl_from_field!(
    ENIT,
    [
        potion_value: u32,
        flags: ENITFlags,
        addiction: FormId,
        addiction_chance: u32,
        use_sound: FormId
    ]
);
impl_static_type_named!(ENIT, b"ENIT");
impl_static_data_size!(
    ENIT,
    FIELDH_SIZE +
    u32::static_data_size() + // potion value
	ENITFlags::static_data_size() +
	FormId::static_data_size() + // addiction
	u32::static_data_size() + // addiction chance
	FormId::static_data_size() // use sound
);
impl Writable for ENIT {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.potion_value.write_to(w)?;
        self.flags.write_to(w)?;
        self.addiction.write_to(w)?;
        self.addiction_chance.write_to(w)?;
        self.use_sound.write_to(w)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ENITFlags {
    /// 0x1: Manual Calc
    /// 0x2: Food
    /// 0x10000: Medicine
    /// 0x20000: Poison
    pub flags: u32,
}
impl ENITFlags {
    pub fn manual_calc(&self) -> bool {
        (self.flags & 0b1) != 0
    }

    pub fn food(&self) -> bool {
        (self.flags & 0b10) != 0
    }

    pub fn medicine(&self) -> bool {
        (self.flags & 0x10000) != 0
    }

    pub fn poison(&self) -> bool {
        (self.flags & 0x20000) != 0
    }
}
impl Parse<'_> for ENITFlags {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = u32::parse(data)?;
        Ok((data, Self { flags }))
    }
}
impl_static_data_size!(ENITFlags, u32::static_data_size());
impl Writable for ENITFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

make_formid_field!(
    /// ->MGEF
    EFID
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EFIT {
    pub magnitude: f32,
    pub area_of_effect: u32,
    // TODO: make duration an enum of Instant, and Time/Value/whatever
    /// 0 = instant
    pub duration: u32,
}
// calculate cost of an effect as: effect_base_cost * (magnitude * duration / 10) ** 1.1
// duration=0 uses it as 10
// magnitude < 1 becomes 1
impl_from_field!(EFIT, [magnitude: f32, area_of_effect: u32, duration: u32]);
impl_static_type_named!(EFIT, b"EFIT");
impl_static_data_size!(
    EFIT,
    FIELDH_SIZE + f32::static_data_size() + u32::static_data_size() + u32::static_data_size()
);
impl Writable for EFIT {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.magnitude.write_to(w)?;
        self.area_of_effect.write_to(w)?;
        self.duration.write_to(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnchantedEffectCollection {
    pub enchanted_item: ENIT,
    pub effect_id: EFID,
    pub item: EFIT,
    pub conditions: Vec<ctda::CTDA>,
}
impl EnchantedEffectCollection {
    pub fn collect<'data, I>(
        enchanted_item: ENIT,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<Self, FromFieldError<'data>>
    where
        I: std::iter::Iterator<Item = GeneralField<'data>>,
    {
        let (_, effect_id) = get_field(field_iter, b"EFID".as_bstr())?;
        let effect_id = match effect_id {
            Some(effect_id) => effect_id,
            None => return Err(FromFieldError::ExpectedSpecificField(b"EFID".as_bstr())),
        };
        let (_, item) = get_field(field_iter, b"EFIT".as_bstr())?;
        let item = match item {
            Some(item) => item,
            None => return Err(FromFieldError::ExpectedSpecificField(b"EFIT".as_bstr())),
        };
        let mut conditions = Vec::new();
        loop {
            let (_, condition) = get_field(field_iter, b"CTDA".as_bstr())?;
            match condition {
                Some(condition) => conditions.push(condition),
                None => break,
            };
        }

        Ok((
            &[],
            Self {
                enchanted_item,
                effect_id,
                item,
                conditions,
            },
        ))
    }
}
impl_static_type_named!(EnchantedEffectCollection, ENIT::static_type_name());
impl DataSize for EnchantedEffectCollection {
    fn data_size(&self) -> usize {
        self.enchanted_item.data_size()
            + self.effect_id.data_size()
            + self.item.data_size()
            + self.conditions.data_size()
    }
}
impl Writable for EnchantedEffectCollection {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.enchanted_item.write_to(w)?;
        self.effect_id.write_to(w)?;
        self.item.write_to(w)?;
        self.conditions.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_size_output, records::common::NullTerminatedString, util::Position3};

    #[test]
    fn test_data() {
        let weight = 4.29;
        let data = DATA { weight };
        let body = assert_size_output!(data);
        assert_eq!(&body[..4], b"DATA");
        assert_eq!(body[4..6], 4u16.to_le_bytes());
        assert_eq!(body[6..], (weight as f32).to_le_bytes());
    }

    #[test]
    fn test_enit() {
        let enit = ENIT {
            potion_value: 0xaabbccdd,
            flags: ENITFlags { flags: 0x0 },
            addiction: FormId::new(0x1054aa66),
            addiction_chance: 0x1,
            use_sound: FormId::new(0x0),
        };
        let body = assert_size_output!(enit);
        assert_eq!(&body[..4], b"ENIT");
        assert_eq!(body[4..6], ((4 + 4 + 4 + 4 + 4) as u16).to_le_bytes());
        assert_eq!(
            &body[6..],
            [
                0xdd, 0xcc, 0xbb, 0xaa, 0x0, 0x0, 0x0, 0x0, 0x66, 0xaa, 0x54, 0x10, 0x1, 0x0, 0x0,
                0x0, 0x0, 0x0, 0x0, 0x0
            ]
        );
    }

    #[test]
    fn test_efit() {
        let magnitude = 4.29;
        let area_of_effect = 5529582;
        let duration = 5;
        let efit = EFIT {
            magnitude,
            area_of_effect,
            duration,
        };
        let body = assert_size_output!(efit);
        assert_eq!(&body[..4], b"EFIT");
        assert_eq!(body[4..6], ((4 + 4 + 4) as u16).to_le_bytes());
        assert_eq!(&body[6..10], magnitude.to_le_bytes());
        assert_eq!(&body[10..14], area_of_effect.to_le_bytes());
        assert_eq!(&body[14..18], duration.to_le_bytes());
    }

    #[test]
    fn test_alch_record() {
        let alch = ALCHRecord {
            common: CommonRecordInfo::test_default(),
            editor_id_index: 0,
            object_bounds_index: 1,
            full_name_index: None,
            keyword_collection_index: None,
            model_collection_index: None,
            icon_index: None,
            message_icon_index: None,
            pickup_sound_index: None,
            drop_sound_index: None,
            weight_index: 2,
            enchanted_effect_collection_index: 3,
            fields: vec![
                ALCHField::EDID(edid::EDID {
                    id: NullTerminatedString::new(b"Testing".as_bstr()),
                }),
                ALCHField::OBND(obnd::OBND {
                    p1: Position3::new(5, 10, 40),
                    p2: Position3::new(9, 30, 80),
                }),
                ALCHField::DATA(DATA { weight: 4.29 }),
                ALCHField::EnchantedEffectCollection(EnchantedEffectCollection {
                    enchanted_item: ENIT {
                        potion_value: 400,
                        flags: ENITFlags { flags: 0 },
                        addiction: FormId::new(0),
                        addiction_chance: 0,
                        use_sound: FormId::new(0),
                    },
                    effect_id: EFID::new(FormId::new(0)),
                    item: EFIT {
                        magnitude: 1.0,
                        area_of_effect: 10,
                        duration: 0,
                    },
                    conditions: vec![ctda::CTDA {
                        op_data: ctda::OperatorData {
                            operator: ctda::Operator::Equal,
                            flags: ctda::Flags::from_byte(0),
                        },
                        unknown: [4, 5, 6],
                        comp_value: ctda::ComparisonValue::Float(4.3),
                        function_index: 0,
                        padding: 0,
                        parameters: ctda::Parameters::Normal {
                            first: 0x0,
                            second: 0x1,
                        },
                        run_on: ctda::RunOn::Target,
                        reference: FormId::new(0),
                        unknown2: -1,
                    }],
                }),
            ],
        };
        assert_size_output!(alch);
    }
}
