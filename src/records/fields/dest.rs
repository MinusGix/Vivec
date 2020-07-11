use super::common::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE};
use crate::{
    make_empty_field, make_model_fields, make_single_value_field,
    parse::{le_u16, le_u32, take, PResult},
    records::common::{get_field, FormId, StaticTypeNamed, TypeNamed},
    util::{DataSize, StaticDataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use std::io::Write;

/// Destruction data
#[derive(Debug, Clone)]
pub struct DEST {
    pub health: u32,
    // TODO: is this talking about the other records that follow DEST entries?
    /// Number of destruction sections that follow
    pub count: u8,
    /// 0b1: VATs enabled
    pub flags: u8,
    pub unknown: u16,
}
impl FromField<'_> for DEST {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, health) = le_u32(field.data)?;
        let (data, count) = take(data, 1usize)?;
        let (data, flag) = take(data, 1usize)?;
        let (data, unknown) = le_u16(data)?;
        Ok((
            data,
            DEST {
                health,
                count: count[0],
                flags: flag[0],
                unknown,
            },
        ))
    }
}
impl StaticTypeNamed<'static> for DEST {
    fn static_type_name() -> &'static BStr {
        b"DEST".as_bstr()
    }
}
impl StaticDataSize for DEST {
    fn static_data_size() -> usize {
        FIELDH_SIZE + u32::static_data_size() + (u8::static_data_size() * 4)
    }
}
impl Writable for DEST {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.health.write_to(w)?;
        self.count.write_to(w)?;
        self.flags.write_to(w)?;
        self.unknown.write_to(w)
    }
}

// I believe these tend to be right after DEST, and repeating in order

#[derive(Debug, Clone)]
pub struct DSTD {
    // TODO: in what manner is this a percent??
    pub health_percent: u16,
    // TODO: make an enumeration for this?
    pub damage_stage: u8,
    pub flags: DSTDFlags,
    pub self_damage_rate: u32,
    pub explosion_id: FormId,
    pub debris_id: FormId,
    pub debris_count: u32,
}
impl FromField<'_> for DSTD {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, health_percent) = le_u16(field.data)?;
        let (data, damage_stage) = take(data, 1usize)?;
        let damage_stage = damage_stage[0];
        let (data, flags) = DSTDFlags::parse(data)?;
        let (data, self_damage_rate) = le_u32(data)?;
        let (data, explosion_id) = FormId::parse(data)?;
        let (data, debris_id) = FormId::parse(data)?;
        let (data, debris_count) = le_u32(data)?;
        Ok((
            data,
            Self {
                health_percent,
                damage_stage,
                flags,
                self_damage_rate,
                explosion_id,
                debris_id,
                debris_count,
            },
        ))
    }
}
impl StaticTypeNamed<'static> for DSTD {
    fn static_type_name() -> &'static BStr {
        b"DSTD".as_bstr()
    }
}
impl StaticDataSize for DSTD {
    fn static_data_size() -> usize {
        FIELDH_SIZE +
        u16::static_data_size() + // health_percent
			u8::static_data_size() + // damage_stage
			DSTDFlags::static_data_size() + // flags
			u32::static_data_size() + // self damage rate
			FormId::static_data_size() + // explosion id
			FormId::static_data_size() + // debris id
			u32::static_data_size() // debris count
    }
}
impl Writable for DSTD {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.health_percent.write_to(w)?;
        self.damage_stage.write_to(w)?;
        self.flags.write_to(w)?;
        self.self_damage_rate.write_to(w)?;
        self.explosion_id.write_to(w)?;
        self.debris_id.write_to(w)?;
        self.debris_count.write_to(w)
    }
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DSTDFlags {
    /// 0b1: cap damage
    /// 0b10: disable object
    /// 0b100: destroy object
    /// 0b1000: ignore external damage
    pub flags: u8,
}
impl DSTDFlags {
    pub fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = take(data, 1)?;
        Ok((data, Self { flags: flags[0] }))
    }

    pub fn cap_damage(&self) -> bool {
        (self.flags & 0b1) != 0
    }

    pub fn disable_object(&self) -> bool {
        (self.flags & 0b10) != 0
    }

    pub fn destroy_object(&self) -> bool {
        (self.flags & 0b100) != 0
    }

    pub fn ignore_external_damage(&self) -> bool {
        (self.flags & 0b1000) != 0
    }
}
impl StaticDataSize for DSTDFlags {
    fn static_data_size() -> usize {
        u8::static_data_size()
    }
}
impl Writable for DSTDFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

make_model_fields!(DMDL; DMDT; DMDS; DMDLCollection);

#[derive(Debug, Clone)]
pub struct DESTCollection<'data> {
    destruction: DEST,
    stage_data: Vec<DSTDCollection<'data>>,
}
impl<'data> DESTCollection<'data> {
    pub fn collect<I>(
        destruction: DEST,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<Self, FromFieldError<'data>>
    where
        I: std::iter::Iterator<Item = GeneralField<'data>>,
    {
        let mut stage_data = Vec::new();
        for _ in 0..destruction.count {
            let (_, dstd) = get_field(field_iter, b"DSTD".as_bstr())?;
            let dstd =
                dstd.ok_or_else(|| FromFieldError::ExpectedSpecificField(b"DSTD".as_bstr()))?;
            let (_, collection) = DSTDCollection::collect(dstd, field_iter)?;
            stage_data.push(collection);
        }

        Ok((
            &[],
            Self {
                destruction,
                stage_data,
            },
        ))
    }
}
impl<'data> StaticTypeNamed<'static> for DESTCollection<'data> {
    fn static_type_name() -> &'static BStr {
        DEST::static_type_name()
    }
}
impl<'data> DataSize for DESTCollection<'data> {
    fn data_size(&self) -> usize {
        self.destruction.data_size() + self.stage_data.data_size()
    }
}
impl<'data> Writable for DESTCollection<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.destruction.write_to(w)?;
        self.stage_data.write_to(w)
    }
}
#[derive(Debug, Clone)]
pub struct DSTDCollection<'data> {
    stage: DSTD,
    model: Option<DMDLCollection<'data>>,
    end: DSTF,
}
impl<'data> DSTDCollection<'data> {
    pub fn collect<I>(
        stage: DSTD,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<Self, FromFieldError<'data>>
    where
        I: std::iter::Iterator<Item = GeneralField<'data>>,
    {
        // TODO: hardcoded names are bad.

        let (_, dmdl) = get_field(field_iter, b"DMDL".as_bstr())?;
        let dmdl = if let Some(dmdl) = dmdl {
            let (_, dmdl) = DMDLCollection::collect(dmdl, field_iter)?;
            Some(dmdl)
        } else {
            None
        };

        let (_, dstf) = get_field(field_iter, b"DSTF".as_bstr())?;
        let dstf = match dstf {
            Some(dstf) => dstf,
            None => return Err(FromFieldError::ExpectedSpecificField(b"DSTF".as_bstr())),
        };

        Ok((
            &[],
            Self {
                stage,
                model: dmdl,
                end: dstf,
            },
        ))
    }
}
impl<'data> StaticTypeNamed<'static> for DSTDCollection<'data> {
    fn static_type_name() -> &'static BStr {
        DSTD::static_type_name()
    }
}
impl<'data> DataSize for DSTDCollection<'data> {
    fn data_size(&self) -> usize {
        self.stage.data_size() + self.model.data_size() + self.end.data_size()
    }
}
impl<'data> Writable for DSTDCollection<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.stage.write_to(w)?;
        if let Some(model) = &self.model {
            model.write_to(w)?;
        }
        self.end.write_to(w)
    }
}

make_empty_field!(DSTF);
