use super::{
    common::{
        get_field, CommonRecordInfo, FormId, FromRecord, FromRecordError, GeneralRecord, Index,
        NullTerminatedString, TypeNamed,
    },
    fields::{
        common::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE},
        ctda, edid, full, kwda, modl, obnd,
    },
};
use crate::{
    collect_one, dispatch_all, make_formid_field, make_single_value_field,
    parse::{le_f32, le_u32, PResult},
    util::{DataSize, StaticDataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use derive_more::From;
use std::io::Write;

#[derive(Debug, Clone)]
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
impl<'data> ALCHRecord<'data> {
    pub fn fields_size(&self) -> usize {
        self.fields.iter().fold(0, |acc, x| acc + x.data_size())
    }
}
impl<'data> TypeNamed<'static> for ALCHRecord<'data> {
    fn type_name(&self) -> &'static BStr {
        b"ALCH".as_bstr()
    }
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
                b"FULL" => collect_one!(full::FULL, field => fields; full_name_index),
                b"KSIZ" => {
                    let (_, ksiz) = kwda::KSIZ::from_field(field)?;
                    let (_, col) = kwda::KWDACollection::collect(ksiz, &mut field_iter)?;
                    keyword_collection_index = Some(fields.len());
                    fields.push(ALCHField::KWDACollection(col));
                }
                b"MODL" => {
                    let (_, modl) = modl::MODL::from_field(field)?;
                    let (_, col) = modl::MODLCollection::collect(modl, &mut field_iter)?;
                    model_collection_index = Some(fields.len());
                    fields.push(ALCHField::MODLCollection(col));
                }
                b"ICON" => collect_one!(ICON, field => fields; icon_index),
                b"MICO" => collect_one!(MICO, field => fields; message_icon_index),
                b"YNAM" => collect_one!(YNAM, field => fields; pickup_sound_index),
                b"ZNAM" => collect_one!(ZNAM, field => fields; drop_sound_index),
                b"DATA" => collect_one!(DATA, field => fields; weight_index),
                b"ENIT" => {
                    let (_, enit) = ENIT::from_field(field)?;
                    let (_, col) = EnchantedEffectCollection::collect(enit, &mut field_iter)?;
                    enchanted_effect_collection_index = Some(fields.len());
                    fields.push(ALCHField::EnchantedEffectCollection(col));
                }
                _ => fields.push(ALCHField::Unknown(field)),
            }
        }

        let editor_id_index = editor_id_index.expect("Expected EDID field");
        let object_bounds_index = object_bounds_index.expect("Expected OBND field");
        let weight_index = weight_index.expect("Expected DATA field");
        let enchanted_effect_collection_index =
            enchanted_effect_collection_index.expect("Expected ENIT field and following fields");

        Ok((
            &[],
            ALCHRecord {
                common: record.common_info,
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
impl<'data> DataSize for ALCHRecord<'data> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
			4 + // data len
			self.common.data_size() +
			self.fields_size()
    }
}
impl<'data> Writable for ALCHRecord<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.type_name().write_to(w)?;
        // TODO: assert size fits within
        (self.fields_size() as u32).write_to(w)?;
        self.common.write_to(w)?;
        for field in self.fields.iter() {
            field.write_to(w)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, From)]
pub enum ALCHField<'data> {
    EDID(edid::EDID<'data>),
    OBND(obnd::OBND),
    FULL(full::FULL),
    // TODO: note: UESP says that there may be more than one KWDA entry? I didn't see that in a quick skim through a handful of skyrim entries
    KWDACollection(kwda::KWDACollection),
    MODLCollection(modl::MODLCollection<'data>),
    ICON(ICON<'data>),
    MICO(MICO<'data>),
    YNAM(YNAM),
    ZNAM(ZNAM),
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
    /// Inventory icon filename
    [Debug, Clone],
    ICON,
    filename,
    NullTerminatedString,
    'data
);
impl<'data> FromField<'data> for ICON<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, filename) = NullTerminatedString::parse(field.data)?;
        Ok((data, Self { filename }))
    }
}

make_single_value_field!(
    /// Message icon filename
    [Debug, Clone],
    MICO,
    filename,
    NullTerminatedString,
    'data
);
impl<'data> FromField<'data> for MICO<'data> {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, filename) = NullTerminatedString::parse(field.data)?;
        Ok((data, Self { filename }))
    }
}

make_formid_field!(
    /// Pickup ->SNDR
    YNAM
);

make_formid_field!(
    /// Drop ->SNDR
    ZNAM
);

make_single_value_field!(
    /// Message icon filename
    [Debug, Clone],
    DATA,
    weight,
    f32
);
impl<'data> FromField<'data> for DATA {
    fn from_field(field: GeneralField<'data>) -> PResult<Self, FromFieldError> {
        let (data, weight) = le_f32(field.data)?;
        Ok((data, Self { weight }))
    }
}

#[derive(Debug, Clone)]
pub struct ENIT {
    /// ?
    potion_value: u32,
    flags: ENITFlags,
    addiction: FormId,
    addiction_chance: u32,
    /// ->SNDR
    use_sound: FormId,
}
impl FromField<'_> for ENIT {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, potion_value) = le_u32(field.data)?;
        let (data, flags) = ENITFlags::parse(data)?;
        let (data, addiction) = FormId::parse(data)?;
        let (data, addiction_chance) = le_u32(data)?;
        let (data, use_sound) = FormId::parse(data)?;
        Ok((
            data,
            Self {
                potion_value,
                flags,
                addiction,
                addiction_chance,
                use_sound,
            },
        ))
    }
}
impl TypeNamed<'static> for ENIT {
    fn type_name(&self) -> &'static BStr {
        b"ENIT".as_bstr()
    }
}
impl StaticDataSize for ENIT {
    fn static_data_size() -> usize {
        FIELDH_SIZE +
        	u32::static_data_size() + // potion value
			ENITFlags::static_data_size() +
			FormId::static_data_size() + // addiction
			u32::static_data_size() + // addiction chance
			FormId::static_data_size() // use sound
    }
}
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
    pub fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = le_u32(data)?;
        Ok((data, Self { flags }))
    }

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
impl StaticDataSize for ENITFlags {
    fn static_data_size() -> usize {
        u32::static_data_size()
    }
}
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
    magnitude: f32,
    area_of_effect: u32,
    // TODO: make duration an enum of Instant, and Time/Value/whatever
    /// 0 = instant
    duration: u32,
}
// calculate cost of an effect as: effect_base_cost * (magnitude * duration / 10) ** 1.1
// duration=0 uses it as 10
// magnitude < 1 becomes 1
impl FromField<'_> for EFIT {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, magnitude) = le_f32(field.data)?;
        let (data, area_of_effect) = le_u32(data)?;
        let (data, duration) = le_u32(data)?;
        Ok((
            data,
            Self {
                magnitude,
                area_of_effect,
                duration,
            },
        ))
    }
}
impl TypeNamed<'static> for EFIT {
    fn type_name(&self) -> &'static BStr {
        b"EFIT".as_bstr()
    }
}
impl StaticDataSize for EFIT {
    fn static_data_size() -> usize {
        FIELDH_SIZE + f32::static_data_size() + u32::static_data_size() + u32::static_data_size()
    }
}
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

#[derive(Debug, Clone)]
pub struct EnchantedEffectCollection {
    enchanted_item: ENIT,
    effect_id: EFID,
    item: EFIT,
    conditions: Vec<ctda::CTDA>,
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
impl TypeNamed<'static> for EnchantedEffectCollection {
    fn type_name(&self) -> &'static BStr {
        self.effect_id.type_name()
    }
}
impl DataSize for EnchantedEffectCollection {
    fn data_size(&self) -> usize {
        self.effect_id.data_size() + self.item.data_size() + self.conditions.data_size()
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
