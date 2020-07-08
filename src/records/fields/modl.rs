use crate::{
    make_single_value_field,
    records::common::FormId,
    util::{DataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use nom::{bytes::complete::take, number::complete::le_u32, IResult};

#[derive(Debug, Clone)]
pub struct AlternateTexture<'data> {
    /// 3d object name inside nif file
    name_3d: &'data BStr,
    /// ->TXST, texture set to use fr this 3d object
    texture_set: FormId,
    index_3d: u32,
}
impl<'data> AlternateTexture<'data> {
    pub fn parse(data: &'data [u8]) -> IResult<&[u8], Self> {
        let (data, size) = le_u32(data)?;
        let (data, name_3d) = take(size as usize)(data)?;
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
            fn from_field(field: $crate::records::fields::common::GeneralField<'data>) -> nom::IResult<&[u8], Self> {
                let (data, filename) = $crate::records::common::NullTerminatedString::parse(field.data)?;
                assert_eq!(data.len(), 0);
                Ok((data, Self { filename }))
            }
        }

        make_single_value_field!(
            /// Model data.
            /// TODO: this is unknown. UESP assumes that it is a series of 12 byte structs of positions (Pos3 as 4 bytes per axis)
            /// I'm skeptical of that, as, for example, OBND uses i16 for it's bounds.
            /// If it is positions, it's likely a 16-bit integer, and if it is in 12 byte groups
            /// then it is two positions per every thing.
            /// This might make sense for a collision box, as you could then specify rectangles of collision
            /// for now, we just store the data raw
            [Debug, Clone, Eq, PartialEq],
            $modt,
            positions,
            refer [u8], // &'data [u8]
            'data
        );
        impl<'data> $crate::records::fields::common::FromField<'data> for $modt<'data> {
            fn from_field(field: $crate::records::fields::common::GeneralField<'data>) -> nom::IResult<&[u8], Self> {
                if field.data.len() % 12 != 0 {
                    panic!("Expected {} to have data that is a multiple of 12!", stringify!($modt));
                }

                let (data, positions) = take(field.data.len())(field.data)?;
                assert_eq!(data.len(), 0);
                Ok((data, Self { positions }))
            }
        }

        #[derive(Debug, Clone)]
        pub struct $mods<'data> {
            pub alternate_textures: Vec<$crate::records::fields::modl::AlternateTexture<'data>>,
        }
        impl<'data> $crate::records::fields::common::FromField<'data> for $mods<'data> {
            fn from_field(field: $crate::records::fields::common::GeneralField<'data>) -> nom::IResult<&[u8], Self> {
                let (data, count) = nom::number::complete::le_u32(field.data)?;
                let (data, alternate_textures) =
                    nom::multi::count($crate::records::fields::modl::AlternateTexture::parse, count as usize)(data)?;
                assert_eq!(data.len(), 0);
                Ok((data, Self { alternate_textures }))
            }
        }
        impl<'data> $crate::records::common::TypeNamed<'static> for $mods<'data> {
            fn type_name(&self) -> &'static BStr {
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
            pub fn collect<I>(modl: $modl<'data>, field_iter: &mut std::iter::Peekable<I>) -> nom::IResult<&'data [u8], Self>
            where
                I: std::iter::Iterator<Item = $crate::records::fields::common::GeneralField<'data>>,
            {
                let model = modl;

                // TODO: should we allow a MODS field without a previous MODT field?

                let modt_typename = stringify!($modt).as_bytes().as_bstr();
                let mods_typename = stringify!($mods).as_bytes().as_bstr();

                let (_, modt) = $crate::records::common::get_field::<_, $modt>(field_iter, modt_typename)?;
                if let Some(modt) = modt {
                    let (_, mods) = $crate::records::common::get_field(field_iter, mods_typename)?;
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
        impl<'data> $crate::records::common::TypeNamed<'static> for $collection<'data> {
            fn type_name(&self) -> &'static BStr {
                self.model.type_name()
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
            positions: &[49, 64, 52, 92, 40, 50, 92, 200, 40, 10, 12, 14],
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
