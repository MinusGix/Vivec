use super::{
    common::{
        full_string::FullString, get_field, CommonRecordInfo, FieldList, FormId, FromRecord,
        FromRecordError, GeneralRecord, NullTerminatedString, StaticTypeNamed, TypeNamed,
    },
    fields::{
        common::{item, object, CollectField, FromFieldError, GeneralField},
        dest, edid, kwda, modl, obnd, vmad,
    },
};
use crate::{
    collect_one, collect_one_collection, dispatch_all, impl_from_field, impl_static_type_named,
    make_field_getter, make_formid_field, make_model_fields, make_single_value_field,
    parse::{PResult, Parse},
    util::{DataSize, StaticDataSize, Writable},
};
use derive_more::From;
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct ARMORecord<'data> {
    common: CommonRecordInfo,
    fields: Vec<ARMOField<'data>>,
}
impl<'data> ARMORecord<'data> {
    make_field_getter!(
        editor_id_index,
        editor_id,
        editor_id_mut,
        ARMOField::EDID,
        edid::EDID<'data>
    );

    make_field_getter!(
        optional: script_index,
        script,
        script_mut,
        ARMOField::VMAD,
        vmad::VMAD<'data, vmad::NoFragments>
    );

    make_field_getter!(
        optional: object_bounds_index,
        object_bounds,
        object_bounds_mut,
        ARMOField::OBND,
        obnd::OBND
    );

    make_field_getter!(
        optional: enchantment_index,
        enchantment,
        enchantment_mut,
        ARMOField::Enchantment,
        Enchantment
    );

    make_field_getter!(
        optional: model_index,
        model,
        model_mut,
        ARMOField::MODLCollection,
        modl::MODLCollection<'data>
    );

    make_field_getter!(
        optional: inventory_mod2_index,
        inventory_mod2,
        inventory_mod2_mut,
        ARMOField::InventoryMO2LCollection,
        InventoryMO2LCollection<'data>
    );
    make_field_getter!(
        optional: inventory_mod4_index,
        inventory_mod4,
        inventory_mod4_mut,
        ARMOField::InventoryMO4LCollection,
        InventoryMO4LCollection<'data>
    );

    // TODO: make getter for BODT|BOD2

    make_field_getter!(
        optional: destruction_index,
        destruction,
        destruction_mut,
        ARMOField::DESTCollection,
        dest::DESTCollection<'data>
    );

    make_field_getter!(
        optional: pickup_sound_index,
        pickup_sound,
        pickup_sound_mut,
        ARMOField::YNAM,
        item::YNAM
    );
    make_field_getter!(
        optional: drop_sound_index,
        drop_sound,
        drop_sound_mut,
        ARMOField::ZNAM,
        item::ZNAM
    );

    make_field_getter!(
        optional: ragdoll_index,
        ragdoll,
        ragdoll_mut,
        ARMOField::BMCT,
        BMCT<'data>
    );

    make_field_getter!(
        optional: equip_slot_index,
        equip_slot,
        equip_slot_mut,
        ARMOField::ETYP,
        ETYP
    );

    // TODO: perhaps group bash impact with bash material?
    make_field_getter!(optional: bash_index, bash, bash_mut, ARMOField::BIDS, BIDS);

    make_field_getter!(
        optional: bash_material_index,
        bash_material,
        bash_material_mut,
        ARMOField::BAMT,
        BAMT
    );

    make_field_getter!(race_index, race, race_mut, ARMOField::RNAM, RNAM);

    make_field_getter!(
        optional: keywords_index,
        keywords,
        keywords_mut,
        ARMOField::KWDACollection,
        kwda::KWDACollection
    );

    make_field_getter!(
        optional: description_index,
        description,
        description_mut,
        ARMOField::DESC,
        item::DESC
    );

    make_field_getter!(
        optional: armatures_index,
        armatures,
        armatures_mut,
        ARMOField::MODLList,
        MODLList<'data>
    );

    make_field_getter!(data_index, data, data_mut, ARMOField::DATA, item::DATA);

    make_field_getter!(
        optional: template_index,
        template,
        template_mut,
        ARMOField::TNAM,
        TNAM
    );
}
impl<'data> FromRecord<'data> for ARMORecord<'data> {
    fn from_record(record: GeneralRecord<'data>) -> PResult<Self, FromRecordError<'data>> {
        let mut edid_index = None;
        let mut vmad_index = None;
        let mut obnd_index = None;
        let mut full_index = None;
        let mut enchantment_index = None;
        let mut modl_collection_index = None;
        let mut inventory_mod2_index = None;
        let mut inventory_mod4_index = None;
        let mut bodt_index = None;
        let mut bod2_index = None;
        let mut dest_collection_index = None;
        let mut ynam_index = None;
        let mut znam_index = None;
        let mut bmct_index = None;
        let mut etyp_index = None;
        let mut bids_index = None;
        let mut bamt_index = None;
        let mut rnam_index = None;
        let mut kwda_collection_index = None;
        let mut desc_index = None;
        let mut modl_list_index = None;
        let mut data_index = None;
        let mut dnam_index = None;
        let mut tnam_index = None;

        let mut fields = Vec::new();
        let mut field_iter = record.fields.into_iter().peekable();

        while let Some(field) = field_iter.next() {
            match field.type_name().as_ref() {
                b"EDID" => collect_one!(edid::EDID, field => fields; edid_index),
                b"VMAD" => {
                    collect_one!(vmad::VMAD<'data, vmad::NoFragments>, field => fields; vmad_index)
                }
                b"OBND" => collect_one!(obnd::OBND, field => fields; obnd_index),
                b"FULL" => collect_one!(object::FULL, field => fields; full_index),
                b"EITM" => {
                    collect_one_collection!(EITM, Enchantment; field, field_iter => fields; enchantment_index)
                }
                b"MODL" => {
                    // special handling because there's two fields with the same name in this... Honestly.
                    if field.data.len() == FormId::static_data_size() {
                        collect_one_collection!(MODL, MODLList; field, field_iter => fields; modl_list_index)
                    } else {
                        collect_one_collection!(modl::MODL, modl::MODLCollection; field, field_iter => fields; modl_collection_index)
                    }
                }
                b"MOD2" => {
                    collect_one_collection!(MOD2, InventoryMO2LCollection; field, field_iter => fields; inventory_mod2_index)
                }
                b"MOD4" => {
                    collect_one_collection!(MOD4, InventoryMO4LCollection; field, field_iter => fields; inventory_mod4_index)
                }
                b"BODT" => collect_one!(item::BODT, field => fields; bodt_index),
                b"BOD2" => collect_one!(item::BOD2, field => fields; bod2_index),
                b"DEST" => {
                    collect_one_collection!(dest::DEST, dest::DESTCollection; field, field_iter => fields; dest_collection_index)
                }
                b"YNAM" => collect_one!(item::YNAM, field => fields; ynam_index),
                b"ZNAM" => collect_one!(item::ZNAM, field => fields; znam_index),
                b"BMCT" => collect_one!(BMCT, field => fields; bmct_index),
                b"ETYP" => collect_one!(ETYP, field => fields; etyp_index),
                b"BIDS" => collect_one!(BIDS, field => fields; bids_index),
                b"BAMT" => collect_one!(BAMT, field => fields; bamt_index),
                b"RNAM" => collect_one!(RNAM, field => fields; rnam_index),
                b"KSIZ" => {
                    collect_one_collection!(kwda::KSIZ, kwda::KWDACollection; field, field_iter => fields; kwda_collection_index)
                }
                b"DESC" => collect_one!(item::DESC, field => fields; desc_index),
                b"DATA" => collect_one!(item::DATA, field => fields; data_index),
                b"DNAM" => collect_one!(DNAM, field => fields; dnam_index),
                b"TNAM" => collect_one!(TNAM, field => fields; tnam_index),
                _ => fields.push(field.into()),
            }
        }

        if edid_index.is_none() {
            Err(FromRecordError::ExpectedField(
                edid::EDID::static_type_name(),
            ))
        } else if bodt_index.is_none() && bod2_index.is_none() {
            Err(FromRecordError::ExpectedField(
                item::BOD2::static_type_name(),
            ))
        } else if rnam_index.is_none() {
            Err(FromRecordError::ExpectedField(RNAM::static_type_name()))
        } else if data_index.is_none() {
            Err(FromRecordError::ExpectedField(
                item::DATA::static_type_name(),
            ))
        } else if dnam_index.is_none() {
            Err(FromRecordError::ExpectedField(DNAM::static_type_name()))
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
impl_static_type_named!(ARMORecord<'_>, b"ARMO");
impl DataSize for ARMORecord<'_> {
    fn data_size(&self) -> usize {
        self.type_name().data_size() +
        4 + // data size len
        self.common.data_size() +
        self.fields.data_size()
    }
}
impl Writable for ARMORecord<'_> {
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
pub enum ARMOField<'data> {
    EDID(edid::EDID<'data>),
    VMAD(vmad::VMAD<'data, vmad::NoFragments>),
    OBND(obnd::OBND),
    FULL(object::FULL),
    Enchantment(Enchantment),
    MODLCollection(modl::MODLCollection<'data>),
    InventoryMO2LCollection(InventoryMO2LCollection<'data>),
    InventoryMO4LCollection(InventoryMO4LCollection<'data>),
    BODT(item::BODT),
    BOD2(item::BOD2),
    DESTCollection(dest::DESTCollection<'data>),
    YNAM(item::YNAM),
    ZNAM(item::ZNAM),
    BMCT(BMCT<'data>),
    ETYP(ETYP),
    BIDS(BIDS),
    BAMT(BAMT),
    RNAM(RNAM),
    KWDACollection(kwda::KWDACollection),
    // TODO: what does non-standard mean?
    /// uesp: Usually 0 unless the enchantment is non-standard, like archmage robes
    DESC(item::DESC),
    MODLList(MODLList<'data>),
    DATA(item::DATA),
    DNAM(DNAM),
    TNAM(TNAM),
    Unknown(GeneralField<'data>),
}
impl<'data> TypeNamed<'data> for ARMOField<'data> {
    fn type_name(&self) -> &'data bstr::BStr {
        dispatch_all!(
            ARMOField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                Enchantment,
                MODLCollection,
                InventoryMO2LCollection,
                InventoryMO4LCollection,
                BODT,
                BOD2,
                DESTCollection,
                YNAM,
                ZNAM,
                BMCT,
                ETYP,
                BIDS,
                BAMT,
                RNAM,
                KWDACollection,
                DESC,
                MODLList,
                DATA,
                DNAM,
                TNAM,
                Unknown
            ],
            x,
            { x.type_name() }
        )
    }
}
impl DataSize for ARMOField<'_> {
    fn data_size(&self) -> usize {
        dispatch_all!(
            ARMOField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                Enchantment,
                MODLCollection,
                InventoryMO2LCollection,
                InventoryMO4LCollection,
                BODT,
                BOD2,
                DESTCollection,
                YNAM,
                ZNAM,
                BMCT,
                ETYP,
                BIDS,
                BAMT,
                RNAM,
                KWDACollection,
                DESC,
                MODLList,
                DATA,
                DNAM,
                TNAM,
                Unknown
            ],
            x,
            { x.data_size() }
        )
    }
}
impl Writable for ARMOField<'_> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        dispatch_all!(
            ARMOField,
            self,
            [
                EDID,
                VMAD,
                OBND,
                FULL,
                Enchantment,
                MODLCollection,
                InventoryMO2LCollection,
                InventoryMO4LCollection,
                BODT,
                BOD2,
                DESTCollection,
                YNAM,
                ZNAM,
                BMCT,
                ETYP,
                BIDS,
                BAMT,
                RNAM,
                KWDACollection,
                DESC,
                MODLList,
                DATA,
                DNAM,
                TNAM,
                Unknown
            ],
            x,
            { x.write_to(w) }
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Enchantment {
    /// -> ENCH
    pub enchantment: EITM,
    pub amount: Option<EAMT>,
}
impl<'data> CollectField<'data, EITM> for Enchantment {
    fn collect<I>(
        enchantment: EITM,
        field_iter: &mut std::iter::Peekable<I>,
    ) -> PResult<'data, Self, FromFieldError<'data>>
    where
        I: std::iter::Iterator<Item = GeneralField<'data>>,
    {
        let (_, amount) = get_field(field_iter, EAMT::static_type_name())?;
        Ok((
            &[],
            Self {
                enchantment,
                amount,
            },
        ))
    }
}
impl_static_type_named!(Enchantment, EITM::static_type_name());
// TODO: EAMT could easily be statically sized.. but it's not due to the macro creating it
impl DataSize for Enchantment {
    fn data_size(&self) -> usize {
        self.enchantment.data_size() + self.amount.data_size()
    }
}
impl Writable for Enchantment {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.enchantment.write_to(w)?;
        if let Some(amount) = &self.amount {
            amount.write_to(w)?;
        }
        Ok(())
    }
}

make_formid_field!(EITM);
make_single_value_field!([Debug, Copy, Clone, Eq, PartialEq], EAMT, amount, u16);
impl_from_field!(EAMT, [amount: u16]);

/// make_inventory_modl_collection(MODL_name; MODT_name; MODS_name; MODLCollection_name; ICON_name; MICO_name; Name that inventory collection should be named);
macro_rules! make_inventory_modl_collection {
    ($life:lifetime; $modl:ty; $modt:ty; $mods:ty; $modlcol:ty; $icon:ty; $mico:ty; $invcol:ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $invcol<$life> {
            model: $modlcol,
            inventory_image: Option<$icon>,
            message_image: Option<$mico>,
        }

        impl<$life> CollectField<$life, $modl> for $invcol<$life> {
            fn collect<I>(
                modl: $modl,
                field_iter: &mut std::iter::Peekable<I>,
            ) -> PResult<$life, Self, FromFieldError<$life>>
            where
                I: std::iter::Iterator<Item = GeneralField<$life>>,
            {
                let (_, model): (&$life [u8], $modlcol) = <$modlcol>::collect(modl, field_iter)?;
				let (_, icon): (&$life [u8], Option<$icon>) = get_field(field_iter, <$icon>::static_type_name())?;
				let (_, mico) = get_field(field_iter, <$mico>::static_type_name())?;
                Ok((
                    &[],
                    Self {
                        model,
                        inventory_image: icon,
                        message_image: mico,
                    },
                ))
            }
		}
		impl<$life> StaticTypeNamed for $invcol<$life> {
			fn static_type_name() -> &'static bstr::BStr {
				<$modlcol>::static_type_name()
			}
		}
        impl DataSize for $invcol<'_> {
            fn data_size(&self) -> usize {
                self.model.data_size()
                    + self.inventory_image.data_size()
                    + self.message_image.data_size()
            }
        }
        impl Writable for $invcol<'_> {
            fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
            where
                T: std::io::Write,
            {
				self.model.write_to(w)?;
				if let Some(inventory_image) = &self.inventory_image {
					inventory_image.write_to(w)?;
				}
				if let Some(message_image) = &self.message_image {
					message_image.write_to(w)?;
				}
				Ok(())
            }
        }
    };
}

// male
make_model_fields!(MOD2; MO2T; MO2S; MO2LCollection);
make_inventory_modl_collection!('a; MOD2<'a>; MO2T; MO2S; MO2LCollection<'a>; item::ICON<'a>; item::MICO<'a>; InventoryMO2LCollection);
// female
make_model_fields!(MOD4; MO4T; MO4S; MO4LCollection);
make_inventory_modl_collection!('a; MOD4<'a>; MO4T; MO4S; MO4LCollection<'a>; ICO2<'a>; MIC2<'a>; InventoryMO4LCollection);
make_single_value_field!(
    /// Inventory icon filename
    [Debug, Clone, PartialEq],
    ICO2,
    filename,
    NullTerminatedString,
    'data
);
impl_from_field!(ICO2, 'data, [filename: NullTerminatedString]);

make_single_value_field!(
    /// Message icon filename
    [Debug, Clone, PartialEq],
    MIC2,
    filename,
    NullTerminatedString,
    'data
);
impl_from_field!(MIC2, 'data, [filename: NullTerminatedString]);

make_single_value_field!(
    [Debug, Clone, Eq, PartialEq],
    BMCT,
    ragdoll,
    FullString,
    'data
);
impl_from_field!(BMCT, 'data, [ragdoll: FullString]);

make_formid_field!(
    /// ->EQUP (only shields)
    ETYP
);
make_formid_field!(
    /// ->IPDS (only shields)
    BIDS
);
make_formid_field!(
    /// ->MATT (only shields)
    BAMT
);
make_formid_field!(
    /// ->RACE (DefaultRace for for most except race specific skins)
    RNAM
);
make_formid_field!(MODL);
type MODLList<'unused> = FieldList<'unused, MODL>;

make_single_value_field!(
    [Debug, Copy, Clone, Eq, PartialEq],
    DNAM,
    /// Base armor rating * 100.
    /// uesp: seems to only use lower u16
    armor_rating,
    u32
);
impl_from_field!(DNAM, [armor_rating: u32]);

make_formid_field!(
    /// -> ARMO to use as template
    TNAM
);
