use super::{
    common::{self, CommonRecordInfo, GeneralRecord, Index},
    fields::{
        common::{write_field_header, GeneralField, FIELDH_SIZE},
        edid, modl, obnd,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_from_field, impl_static_data_size,
    impl_static_type_named, make_formid_field, make_single_value_field,
    parse::{PResult, Parse},
    util::{DataSize, Writable},
};
use bstr::BStr;
use common::{FromRecord, FromRecordError, TypeNamed};
use derive_more::From;
use std::io::Write;

/// Contains information on addon nodes
/// appear to be generic visual attachments for any object
/// Note: there can be an Empty-Record of this
#[derive(Debug, Clone, PartialEq)]
pub struct ADDNRecord<'data> {
    pub common: CommonRecordInfo,
    /// EDID
    pub editor_id_index: Option<Index>,
    /// OBND
    pub object_bounds_index: Option<Index>,
    /// MODL + MODT + MODS(?)
    pub model_collection_index: Option<Index>,
    /// MODT
    //pub model_data_index: Option<Index>,
    /// DATA
    pub addon_node_index_index: Option<Index>,
    /// SNAM. Formid
    pub ambient_sound_index: Option<Index>,
    /// DNAM
    pub flags_index: Option<Index>,

    pub fields: Vec<ADDNField<'data>>,
}
impl<'data> FromRecord<'data> for ADDNRecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError> {
        let mut edid_index = None;
        let mut obnd_index = None;
        let mut modl_collection_index = None;
        let mut data_index = None;
        let mut snam_index = None;
        let mut dnam_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();

        while let Some(field) = field_iter.next() {
            match field.type_name.as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; edid_index),
                b"OBND" => collect_one!(obnd::OBND, field => fields; obnd_index),
                b"MODL" => {
                    collect_one_collection!(modl::MODL, modl::MODLCollection; field, field_iter => fields; modl_collection_index)
                }
                //b"MODT" => collect_one!(modl::MODT, field => fields; modt_index),
                b"DATA" => collect_one!(DATA, field => fields; data_index),
                b"SNAM" => collect_one!(SNAM, field => fields; snam_index),
                b"DNAM" => collect_one!(DNAM, field => fields; dnam_index),
                _ => fields.push(ADDNField::Unknown(field)),
            }
        }

        Ok((
            &[],
            ADDNRecord {
                common: record.common,
                editor_id_index: edid_index,
                object_bounds_index: obnd_index,
                model_collection_index: modl_collection_index,
                addon_node_index_index: data_index,
                ambient_sound_index: snam_index,
                flags_index: dnam_index,
                fields,
            },
        ))
    }
}
impl_static_type_named!(ADDNRecord<'_>, b"ADDN");
impl<'data> DataSize for ADDNRecord<'data> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
            4 + // data len
            self.common.data_size() +
            self.fields.data_size()
    }
}
impl<'data> Writable for ADDNRecord<'data> {
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
pub enum ADDNField<'data> {
    EDID(edid::EDID<'data>),
    OBND(obnd::OBND),
    MODLCollection(modl::MODLCollection<'data>),
    DATA(DATA),
    SNAM(SNAM),
    DNAM(DNAM),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for ADDNField<'data> {
    fn type_name(&self) -> &'data BStr {
        dispatch_all!(
            ADDNField,
            self,
            [EDID, OBND, MODLCollection, DATA, SNAM, DNAM, Unknown],
            x,
            { x.type_name() }
        )
    }
}
impl<'data> DataSize for ADDNField<'data> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            ADDNField,
            self,
            [EDID, OBND, MODLCollection, DATA, SNAM, DNAM, Unknown],
            x,
            { x.data_size() }
        )
    }
}
impl<'data> Writable for ADDNField<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            ADDNField,
            self,
            [EDID, OBND, MODLCollection, DATA, SNAM, DNAM, Unknown],
            x,
            { x.write_to(w) }
        )
    }
}

make_single_value_field!(
    [Debug, Copy, Clone, Eq, PartialEq],
    DATA,
    /// Unique integer within ADDN
    /// may be used instead of FormId for reference
    addon_node_index,
    u32
);
impl_from_field!(DATA, [addon_node_index: u32]);

make_formid_field!(
    /// FormId for a SOUN record
    SNAM
);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DNAM {
    /// According to UESP. Always 0 in original files
    pub master_particle_system_cap: u16,
    // TODO: make this it's own flag structure
    /// According to uesp:
    /// 0x1: unknown, always set in original files
    /// 0x2: Always loaded - Camera? dust spray/blood spray/fire impact (but not forst)
    pub flags: u16,
}
impl DNAM {
    pub fn new(master_particle_system_cap: u16, flags: u16) -> DNAM {
        DNAM {
            master_particle_system_cap,
            flags,
        }
    }
}
impl_from_field!(DNAM, [master_particle_system_cap: u16, flags: u16]);
impl_static_type_named!(DNAM, b"DNAM");
impl_static_data_size!(
    DNAM,
    FIELDH_SIZE +
    u16::static_data_size() + // master particle system cap
    u16::static_data_size() // flags
);
impl Writable for DNAM {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.master_particle_system_cap.write_to(w)?;
        self.flags.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;

    #[test]
    fn data_test() {
        let data = DATA {
            addon_node_index: 42,
        };
        assert_size_output!(data);
    }

    #[test]
    fn dnam_test() {
        let dnam = DNAM {
            master_particle_system_cap: 0,
            flags: 0x1,
        };
        assert_size_output!(dnam);
    }
}
