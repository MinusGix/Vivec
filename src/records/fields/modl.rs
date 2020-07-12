use crate::{
    make_single_value_field,
    parse::{le_u32, take, PResult},
    records::common::FormId,
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};

#[derive(Debug, Clone)]
pub struct AlternateTexture<'data> {
    /// 3d object name inside nif file
    name_3d: &'data BStr,
    /// ->TXST, texture set to use fr this 3d object
    texture_set: FormId,
    index_3d: u32,
}
impl<'data> AlternateTexture<'data> {
    pub fn parse(data: &'data [u8]) -> PResult<Self> {
        let (data, size) = le_u32(data)?;
        let (data, name_3d) = take(data, size as usize)?;
        let name_3d = name_3d.as_bstr();
        let (data, texture_set) = FormId::parse(data)?;
        let (data, index_3d) = le_u32(data)?;
        Ok((
            data,
            Self {
                name_3d,
                texture_set,
                index_3d,
            },
        ))
    }
}
impl<'data> DataSize for AlternateTexture<'data> {
    fn data_size(&self) -> usize {
        use crate::util::StaticDataSize;
        u32::static_data_size() + // string size
            self.name_3d.data_size() +
            self.texture_set.data_size() +
            self.index_3d.data_size()
    }
}
impl<'data> Writable for AlternateTexture<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        // TODO: assert that string length fits
        (self.name_3d.len() as u32).write_to(w)?;
        self.name_3d.write_to(w)?;
        self.texture_set.write_to(w)?;
        self.index_3d.write_to(w)
    }
}

#[macro_export]
macro_rules! make_model_fields {
    ($modl:ident; $modt:ident; $mods:ident; $collection:ident) => {
        // TODO: I can't seem to do: $crate::make_single_value_field! :/
        make_single_value_field!(
            [Debug, Clone, Eq, PartialEq],
            $modl,
            /// Path to .nif model file
            filename,
            full_type $crate::records::common::NullTerminatedString<'data>,
            'data
        );
        impl<'data> $crate::records::fields::common::FromField<'data> for $modl<'data> {
            fn from_field(field: $crate::records::fields::common::GeneralField<'data>) -> $crate::parse::PResult<'data, Self, $crate::records::fields::common::FromFieldError<'data>> {
                let (data, filename) = $crate::records::common::NullTerminatedString::parse(field.data)?;
                assert_eq!(data.len(), 0);
                Ok((data, Self { filename }))
            }
        }

        make_single_value_field!(
            /// Model data.
            /// TODO: this is unknown. UESP has some info, but it's still iffy at best.
            [Debug, Clone, Eq, PartialEq],
            $modt,
            values,
            refer [u8], // &'data [u8]
            'data
        );
        impl<'data> $crate::records::fields::common::FromField<'data> for $modt<'data> {
            fn from_field(field: $crate::records::fields::common::GeneralField<'data>) -> $crate::parse::PResult<'data, Self, $crate::records::fields::common::FromFieldError<'data>> {
                // The MODT field is scary
                //if field.data.len() % 12 != 0 {
                //    return Err($crate::parse::ParseError::InvalidByteCount {
                //        found: field.data.len()
                //    }.into());
                //}

                let (data, values) = $crate::parse::take(field.data, field.data.len())?;
                assert_eq!(data.len(), 0);
                Ok((data, Self { values }))
            }
        }

        #[derive(Debug, Clone)]
        pub struct $mods<'data> {
            pub alternate_textures: Vec<$crate::records::fields::modl::AlternateTexture<'data>>,
        }
        impl<'data> $crate::records::fields::common::FromField<'data> for $mods<'data> {
            fn from_field(field: $crate::records::fields::common::GeneralField<'data>) -> $crate::parse::PResult<'data, Self, $crate::records::fields::common::FromFieldError<'data>> {
                let (data, count) = $crate::parse::le_u32(field.data)?;
                let (data, alternate_textures) = $crate::parse::count(data, $crate::records::fields::modl::AlternateTexture::parse, count as usize)?;
                assert_eq!(data.len(), 0);
                Ok((data, Self { alternate_textures }))
            }
        }
        impl<'data> $crate::records::common::StaticTypeNamed<'static> for $mods<'data> {
            fn static_type_name() -> &'static BStr {
                use bstr::ByteSlice;
                stringify!($mods).as_bytes().as_bstr()
            }
        }
        impl<'data> $crate::util::DataSize for $mods<'data> {
            fn data_size(&self) -> usize {
                use $crate::util::StaticDataSize;
                $crate::records::fields::common::FIELDH_SIZE +
                    u32::static_data_size() + // alternate textures len
                    self.alternate_textures.data_size()
            }
        }
        impl<'data> $crate::util::Writable for $mods<'data> {
            fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
            where
                T: std::io::Write,
            {
                $crate::records::fields::common::write_field_header(self, w)?;
                // TODO: assert that it fits within
                (self.alternate_textures.len() as u32).write_to(w)?;
                self.alternate_textures.write_to(w)
            }
        }

        #[derive(Debug, Clone)]
        pub struct $collection<'data> {
            pub model: $modl<'data>,
            pub texture_data: Option<$modt<'data>>,
            pub alternate_textures: Option<$mods<'data>>,
        }
        impl<'data> $collection<'data> {
            pub fn collect<I>(modl: $modl<'data>, field_iter: &mut std::iter::Peekable<I>) -> $crate::parse::PResult<'data, Self, $crate::records::fields::common::FromFieldError<'data>>
            where
                I: std::iter::Iterator<Item = $crate::records::fields::common::GeneralField<'data>>,
            {
                use $crate::records::common::StaticTypeNamed;
                let model = modl;

                // TODO: should we allow a MODS field without a previous MODT field?

                let (_, modt) = $crate::records::common::get_field::<_, $modt>(field_iter, $modt::static_type_name())?;
                if let Some(modt) = modt {
                    let (_, mods) = $crate::records::common::get_field(field_iter, $mods::static_type_name())?;
                    Ok((
                        &[],
                        Self {
                            model,
                            texture_data: Some(modt),
                            alternate_textures: mods,
                        },
                    ))
                } else {
                    Ok((
                        &[],
                        Self {
                            model,
                            texture_data: None,
                            alternate_textures: None,
                        },
                    ))
                }
            }
        }
        // TODO: this is rather hacky, since a collection doesn't have a name :/
        impl<'data> $crate::records::common::StaticTypeNamed<'static> for $collection<'data> {
            fn static_type_name() -> &'static BStr {
                $modl::static_type_name()
            }
        }
        impl<'data> $crate::util::DataSize for $collection<'data> {
            fn data_size(&self) -> usize {
                self.model.data_size() + self.texture_data.data_size() + self.alternate_textures.data_size()
            }
        }
        impl<'data> $crate::util::Writable for $collection<'data> {
            fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
            where
                T: std::io::Write,
            {
                self.model.write_to(w)?;
                if let Some(modt) = &self.texture_data {
                    modt.write_to(w)?;
                }

                if let Some(mods) = &self.alternate_textures {
                    mods.write_to(w)?;
                }

                Ok(())
            }
        }
    };
}

make_model_fields!(MODL; MODT; MODS; MODLCollection);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;
    use crate::records::common::NullTerminatedString;
    use bstr::ByteSlice;
    #[test]
    fn modl_test() {
        let modl = MODL {
            filename: NullTerminatedString::new(b"omega_model".as_bstr()),
        };
        assert_size_output!(modl);
    }

    #[test]
    fn modt_test() {
        let modt = MODT {
            values: &[49, 64, 52, 92, 40, 50, 92, 200, 40, 10, 12, 14],
        };
        assert_size_output!(modt);
    }

    #[test]
    fn mods_test() {
        let name_3d = b"A".as_bstr();
        let mods = MODS {
            alternate_textures: vec![AlternateTexture {
                name_3d: &name_3d,
                texture_set: FormId::new(42),
                index_3d: 92,
            }],
        };
        assert_size_output!(mods);
    }
}
