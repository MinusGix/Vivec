use crate::{
    parse::{le_u16, take, PResult, ParseError},
    records::common::TypeNamed,
    util::{fmt_data, DataSize, Writable},
};
use bstr::{BStr, ByteSlice};
use derive_more::From;
use std::io::Write;

pub mod formid_wrap;
pub mod item;
pub mod rgbu;

/// Always four characters
pub type FieldName<'data> = &'data BStr;

/// Field header size, (type_name_len + data_size_len)
pub const FIELDH_SIZE: usize = 4 + 2;
/// Writes the fields header to [writer]
pub fn write_field_header<'data, T, W>(field: &T, writer: &mut W) -> std::io::Result<()>
where
    T: TypeNamed<'data> + DataSize,
    W: std::io::Write,
{
    assert!(
        field.data_size() >= FIELDH_SIZE,
        format!(
            "Field ({}) data size was less than field header size, this is certainly a bug.",
            field.type_name()
        )
    );
    writer.write_all(field.type_name().as_bstr())?;
    // TODO: assert that data_size fits wthin a u16
    // We subtract the FIELDH_SIZE, since the calculations shouldn't include that
    let data_size = field.data_size() - FIELDH_SIZE;
    writer.write_all(&(data_size as u16).to_le_bytes())?;

    Ok(())
}

/// A general holder for fields which we don't know anything about
/// may or may not be compressed
#[derive(Clone, Eq, PartialEq)]
pub struct GeneralField<'data> {
    pub type_name: FieldName<'data>,
    pub data: &'data [u8],
}
impl<'data> GeneralField<'data> {
    pub fn new(type_name: FieldName<'data>, data: &'data [u8]) -> GeneralField<'data> {
        GeneralField { type_name, data }
    }

    pub fn parse(data: &'data [u8]) -> PResult<GeneralField<'data>> {
        let (data, type_name) = take(data, 4)?;
        let type_name = type_name.as_bstr();
        let (data, field_data_size) = le_u16(data)?;
        let (data, field_data) = take(data, field_data_size as usize)?;

        Ok((data, GeneralField::new(type_name, field_data)))
    }
}
impl<'data> TypeNamed<'data> for GeneralField<'data> {
    fn type_name(&self) -> &'data BStr {
        self.type_name
    }
}
impl<'data> Writable for GeneralField<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        w.write_all(self.data)?;
        Ok(())
    }
}
impl<'data> DataSize for GeneralField<'data> {
    /// Only valid when data.len() is less than u16::MAX
    fn data_size(&self) -> usize {
        FIELDH_SIZE + self.data.len()
    }
}
impl<'data> std::fmt::Debug for GeneralField<'data> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = fmt.debug_struct("GeneralField");
        res.field("type_name", &self.type_name);
        fmt_data(&mut res, "data", self.data, 10);
        res.finish()
    }
}

#[derive(Debug, Clone, PartialEq, From)]
pub enum FromFieldError<'data> {
    /// An unexpected end of fields
    UnexpectedEnd,
    /// Expected Field
    ExpectedSpecificField(FieldName<'data>),
    ParseError(ParseError<'data>),
}

pub trait FromField<'data>: Sized {
    fn from_field(field: GeneralField<'data>) -> PResult<'data, Self, FromFieldError>;
}

#[macro_export]
macro_rules! make_empty_field {
    ($(#[$outer:meta])* $name:ident) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        pub struct $name;
        impl $crate::records::common::StaticTypeNamed<'static> for $name {
            fn static_type_name() -> &'static BStr {
                stringify!($name).as_bytes().as_bstr()
            }
        }
        impl $crate::records::fields::common::FromField<'_> for $name {
            fn from_field(field: GeneralField<'_>) -> crate::parse::PResult<Self, $crate::records::fields::common::FromFieldError> {
                if (!field.data.is_empty()) {
                    return Err($crate::parse::ParseError::ExpectedExact {
                        expected: 0,
                        found: field.data.len()
                    }.into())
                }
                Ok((&[], Self {}))
            }
        }
        impl $crate::util::DataSize for $name {
            fn data_size(&self) -> usize {
                $crate::records::fields::common::FIELDH_SIZE
            }
        }
        impl $crate::util::Writable for $name {
            fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
            where
                T: std::io::Write
            {
                $crate::records::fields::common::write_field_header(self, w)?;
                Ok(())
            }
        }
    };
}

/// make_single_value_field([Debug, Clone], CNAM, author, NullTerminatedString, 'data)
/// make_single_value_field([Debug, Clone, Eq, PartialEq], DATA, value, u64)
#[macro_export]
macro_rules! make_single_value_field {
    ($(#[$outer:meta])* [$($de:ident),*], $name:ident, $(#[$inner:meta])* $field_name:ident, $field_type:ty) => {
        $(#[$outer])*
        #[derive($($de),*)]
        pub struct $name {
            $(#[$inner])*
            pub $field_name: $field_type,
        }
        impl $crate::records::common::StaticTypeNamed<'static> for $name {
            fn static_type_name() -> &'static bstr::BStr {
                use bstr::ByteSlice;
                stringify!($name).as_bytes().as_bstr()
            }
        }
        impl $crate::util::DataSize for $name {
            fn data_size(&self) -> usize {
                $crate::records::fields::common::FIELDH_SIZE + self.$field_name.data_size()
            }
        }
        impl $crate::util::Writable for $name {
            fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
            where
                T: std::io::Write
            {
                $crate::records::fields::common::write_field_header(self, w)?;
                self.$field_name.write_to(w)?;
                Ok(())
            }
        }
    };
    // This is a bit iffy, since it takes in an ident rather than a type, since I can't seem to join a ty and a lifetime together
    ($(#[$outer:meta])* [$($de:ident),*], $name:ident, $(#[$inner:meta])* $field_name:ident, $field_type:ident, $life:lifetime) => {
        make_single_value_field!($(#[$outer])* [$($de),*], $name, $(#[$inner])* $field_name, full_type $field_type<$life>, $life);
    };

    ($(#[$outer:meta])* [$($de:ident),*], $name:ident, $(#[$inner:meta])* $field_name:ident, refer $field_type:ty, $life:lifetime) => {
        make_single_value_field!($(#[$outer])* [$($de),*], $name, $(#[$inner])* $field_name, full_type &$life $field_type, $life);
    };

    ($(#[$outer:meta])* [$($de:ident),*], $name:ident, $(#[$inner:meta])* $field_name:ident, full_type $field_type:ty, $life:lifetime) => {
        $(#[$outer])*
        #[derive($($de),*)]
        pub struct $name<$life> {
            $(#[$inner])*
            pub $field_name: $field_type,
        }
        impl<$life> $crate::records::common::StaticTypeNamed<'static> for $name<$life> {
            fn static_type_name() -> &'static bstr::BStr {
                use bstr::ByteSlice;
                stringify!($name).as_bytes().as_bstr()
            }
        }
        impl<$life> $crate::util::DataSize for $name<$life> {
            fn data_size(&self) -> usize {
                $crate::records::fields::common::FIELDH_SIZE + self.$field_name.data_size()
            }
        }
        impl<$life> $crate::util::Writable for $name<$life> {
            fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
            where
                T: std::io::Write
            {
                $crate::records::fields::common::write_field_header(self, w)?;
                self.$field_name.write_to(w)?;
                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! assert_size_output {
    ($name:ident) => {{
        use $crate::util::{DataSize, Writable};
        let mut data = Vec::new();
        let data_size = $name.data_size();
        data.reserve(data_size);
        $name.write_to(&mut data).unwrap();
        println!("data: {:#?}", data);
        println!("data size: {}", data_size);
        println!("data len: {}", data.len());
        assert_eq!(data_size, data.len());

        data
    }};
}

#[cfg(test)]
mod test {
    use super::GeneralField;
    use crate::assert_size_output;
    use bstr::ByteSlice;

    #[test]
    fn general_field_test() {
        let field = GeneralField::new(
            b"NEMO".as_bstr(),
            &[0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc],
        );

        assert_size_output!(field);
    }
}
