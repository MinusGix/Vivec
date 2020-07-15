// TODO: support any name for structure, with separate string for typename
/// NOTE: name should always be 4 characters
#[macro_export]
macro_rules! make_formid_field {
	($(#[$outer:meta])* $name:ident) => {
		make_formid_field!($(#[$outer])* $name $name);
	};
	($(#[$outer:meta])* $name:ident $type_name:ident) => {
		$(#[$outer])*
		#[derive(Debug, Copy, Clone, Eq, PartialEq)]
		pub struct $name {
			pub formid: $crate::records::common::FormId,
		}
		impl $name {
			pub fn new (formid: $crate::records::common::FormId) -> Self {
				Self { formid }
			}
		}
		impl $crate::records::fields::common::FromField<'_> for $name {
			fn from_field(field: $crate::records::fields::common::GeneralField<'_>) -> $crate::parse::PResult<Self, $crate::records::fields::common::FromFieldError> {
				let (data, formid) = $crate::records::common::FormId::parse(field.data)?;
				// TODO: check that it used up all the data
				Ok((data, Self::new(formid)))
			}
		}
		impl $crate::records::common::StaticTypeNamed<'static> for $name {
			fn static_type_name () -> &'static bstr::BStr {
				use bstr::ByteSlice;
				stringify!($type_name).as_bytes().as_bstr()
			}
		}
		impl $crate::util::StaticDataSize for $name {
			fn static_data_size() -> usize {
				$crate::records::fields::common::FIELDH_SIZE + $crate::records::common::FormId::static_data_size()
			}
		}
		impl $crate::util::Writable for $name {
			fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
			where
				T: std::io::Write
			{
				$crate::records::fields::common::write_field_header(self, w)?;
				self.formid.write_to(w)?;
				Ok(())
			}
		}
	}
}

// TODO: this should not be public
make_formid_field!(
    /// DO NOT USE THIS
    Test
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_formid() {
        let inst = Test::new(crate::records::common::FormId::new(0x429640aa));
        crate::assert_size_output!(inst);
    }
}
