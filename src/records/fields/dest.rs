use super::common::{write_field_header, FromFieldError, GeneralField, FIELDH_SIZE};
use crate::{
    impl_from_field, impl_static_data_size, impl_static_type_named, make_empty_field,
    make_model_fields, make_single_value_field,
    parse::{take, PResult, Parse},
    records::common::{get_field, FormId, StaticTypeNamed},
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use std::io::Write;

/// Destruction data
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DEST {
    pub health: u32,
    // TODO: is this talking about the other fields that follow DEST entries?
    /// Number of destruction sections that follow
    pub count: u8,
    /// 0b1: VATs enabled
    pub flags: u8,
    pub unknown: u16,
}
impl_from_field!(DEST, [health: u32, count: u8, flags: u8, unknown: u16]);
impl_static_type_named!(DEST, b"DEST");
impl_static_data_size!(
    DEST,
    FIELDH_SIZE + u32::static_data_size() + (u8::static_data_size() * 4)
);
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

#[derive(Debug, Clone, Eq, PartialEq)]
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
impl_from_field!(
    DSTD,
    [
        health_percent: u16,
        damage_stage: u8,
        flags: DSTDFlags,
        self_damage_rate: u32,
        explosion_id: FormId,
        debris_id: FormId,
        debris_count: u32
    ]
);
impl_static_type_named!(DSTD, b"DSTD");
impl_static_data_size!(
    DSTD,
    FIELDH_SIZE +
    u16::static_data_size() + // health_percent
	u8::static_data_size() + // damage_stage
    DSTDFlags::static_data_size() + // flags
    u32::static_data_size() + // self damage rate
	FormId::static_data_size() + // explosion id
	FormId::static_data_size() + // debris id
	u32::static_data_size() // debris count
);
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
impl Parse<'_> for DSTDFlags {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, flags) = take(data, 1)?;
        Ok((data, Self { flags: flags[0] }))
    }
}
impl_static_data_size!(DSTDFlags, u8::static_data_size());
impl Writable for DSTDFlags {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.flags.write_to(w)
    }
}

make_model_fields!(DMDL; DMDT; DMDS; DMDLCollection);

#[derive(Debug, Clone, PartialEq)]
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
            let (_, dstd) = get_field(field_iter, DSTD::static_type_name())?;
            let dstd = dstd
                .ok_or_else(|| FromFieldError::ExpectedSpecificField(DSTD::static_type_name()))?;
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
impl_static_type_named!(DESTCollection<'_>, DEST::static_type_name());
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
#[derive(Debug, Clone, PartialEq)]
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

        let (_, dmdl) = get_field(field_iter, DMDL::static_type_name())?;
        let dmdl = if let Some(dmdl) = dmdl {
            let (_, dmdl) = DMDLCollection::collect(dmdl, field_iter)?;
            Some(dmdl)
        } else {
            None
        };

        let (_, dstf) = get_field(field_iter, DSTF::static_type_name())?;
        let dstf = match dstf {
            Some(dstf) => dstf,
            None => {
                return Err(FromFieldError::ExpectedSpecificField(
                    DSTF::static_type_name(),
                ))
            }
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
impl_static_type_named!(DSTDCollection<'_>, DSTD::static_type_name());
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
