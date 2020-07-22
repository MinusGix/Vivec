use crate::{
    parse::{single, PResult, Parse},
    util::{DataSize, Writable},
};

/// An RGB structure with an unused (?) third component
/// This is a utility class, to be used in other fields. Such as CNAM, PNAM and others
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RGBU {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    /// Always 0x00 (only used in AACT so far)
    pub unused: u8,
}
impl RGBU {
    pub fn new(red: u8, green: u8, blue: u8, unused: u8) -> RGBU {
        RGBU {
            red,
            green,
            blue,
            unused,
        }
    }
}
impl Parse<'_> for RGBU {
    fn parse(data: &[u8]) -> PResult<RGBU> {
        let (data, red) = single(data)?;
        let (data, green) = single(data)?;
        let (data, blue) = single(data)?;
        let (data, unused) = single(data)?;
        Ok((data, RGBU::new(red, green, blue, unused)))
    }
}
impl DataSize for RGBU {
    fn data_size(&self) -> usize {
        self.red.data_size()
            + self.green.data_size()
            + self.blue.data_size()
            + self.unused.data_size()
    }
}
impl Writable for RGBU {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        self.red.write_to(w)?;
        self.green.write_to(w)?;
        self.blue.write_to(w)?;
        self.unused.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::RGBU;

    #[test]
    fn rgbu_check() {
        let rgbu = RGBU {
            red: 0x10,
            green: 0x24,
            blue: 0x92,
            unused: 0x00,
        };

        crate::assert_size_output!(rgbu);
    }
}
