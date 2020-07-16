#[derive(Debug, Clone, PartialEq)]
pub enum ParseError<'data> {
    /// Expected specific bytes
    ExpectedBytes(&'data [u8]),
    /// Expected bytes and found EOF
    UnexpectedEOF,
    /// Expected there to be no more bytes
    ExpectedEOF,
    // TODO: have some way to know what the value you got was
    /// There was an invalid value for an enumeration
    InvalidEnumerationValue,
    /// Expected an exact number of bytes
    ExpectedExact { expected: usize, found: usize },
    /// More general version of above, for when the amount of bytes was invalid
    InvalidByteCount { found: usize },
}

pub type PResult<'data, V, E = ParseError<'data>> = Result<(&'data [u8], V), E>;

pub fn single(data: &[u8]) -> PResult<u8> {
    if data.is_empty() {
        Err(ParseError::UnexpectedEOF)
    } else {
        Ok((&data[1..], data[0]))
    }
}

/// Returns slice with exactly [amount] entries
pub fn take<'data>(data: &'data [u8], amount: usize) -> PResult<&'data [u8]> {
    if data.len() < amount {
        Err(ParseError::UnexpectedEOF)
    } else {
        Ok((&data[amount..], &data[..amount]))
    }
}

pub fn le_u16(data: &[u8]) -> PResult<u16> {
    let (data, v) = take(data, 2)?;
    Ok((data, u16::from_le_bytes([v[0], v[1]])))
}
pub fn le_i16(data: &[u8]) -> PResult<i16> {
    let (data, v) = take(data, 2)?;
    Ok((data, i16::from_le_bytes([v[0], v[1]])))
}

pub fn le_u32(data: &[u8]) -> PResult<u32> {
    let (data, v) = take(data, 4)?;
    Ok((data, u32::from_le_bytes([v[0], v[1], v[2], v[3]])))
}
pub fn le_i32(data: &[u8]) -> PResult<i32> {
    let (data, v) = take(data, 4)?;
    Ok((data, i32::from_le_bytes([v[0], v[1], v[2], v[3]])))
}

pub fn le_u64(data: &[u8]) -> PResult<u64> {
    let (data, v) = take(data, 8)?;
    Ok((
        data,
        u64::from_le_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]),
    ))
}
pub fn le_i64(data: &[u8]) -> PResult<i64> {
    let (data, v) = take(data, 8)?;
    Ok((
        data,
        i64::from_le_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]),
    ))
}

pub fn le_f32(data: &[u8]) -> PResult<f32> {
    let (data, v) = take(data, 4)?;
    Ok((data, f32::from_le_bytes([v[0], v[1], v[2], v[3]])))
}

/// Note: this loops over as many times as possible.
/// It is different from nom's many0, as it will stop when there's no more data
pub fn many<'data, T, R, V>(mut data: &'data [u8], func: T) -> Result<(&'data [u8], Vec<V>), R>
where
    R: From<ParseError<'data>>,
    T: Fn(&'data [u8]) -> Result<(&'data [u8], V), R>,
{
    let mut result = Vec::new();
    loop {
        if data.is_empty() {
            break;
        }

        let (data_val, value) = func(data)?;
        data = data_val;
        result.push(value);
    }
    Ok((data, result))
}

/// Expects [expect] bytes to be there, consumes them, and returns them.
pub fn tag<'data>(data: &'data [u8], expect: &'data [u8]) -> PResult<'data, &'data [u8]> {
    if data.len() < expect.len() {
        return Err(ParseError::ExpectedBytes(expect));
    }

    if &data[..expect.len()] == expect {
        Ok((&data[expect.len()..], expect))
    } else {
        Err(ParseError::ExpectedBytes(expect))
    }
}

/// Consumes bytes until [until], returning the bytes that were eaten. Does not include [until]
/// TODO: support multi-byte [until] (currently not needed)
pub fn take_until(data: &[u8], until: u8) -> PResult<&[u8]> {
    for i in 0..data.len() {
        if data[i] == until {
            return Ok((&data[i..], &data[..i]));
        }
    }
    // Expected [until]
    Err(ParseError::UnexpectedEOF)
}

pub fn count<'data, F, R, V>(
    mut data: &'data [u8],
    func: F,
    amount: usize,
) -> Result<(&'data [u8], Vec<V>), R>
where
    R: From<ParseError<'data>>,
    F: Fn(&'data [u8]) -> Result<(&'data [u8], V), R>,
{
    let mut result = Vec::new();

    for _ in 0..amount {
        let (data_val, value) = func(data)?;
        data = data_val;
        result.push(value);
    }

    Ok((data, result))
}

pub trait Parse: Sized {
    fn parse(data: &[u8]) -> PResult<Self>;
}
impl Parse for u8 {
    fn parse(data: &[u8]) -> PResult<Self> {
        single(data)
    }
}
impl Parse for i8 {
    fn parse(data: &[u8]) -> PResult<Self> {
        let (data, value) = single(data)?;
        Ok((data, i8::from_le_bytes([value])))
    }
}
impl Parse for u16 {
    fn parse(data: &[u8]) -> PResult<Self> {
        le_u16(data)
    }
}
impl Parse for i16 {
    fn parse(data: &[u8]) -> PResult<Self> {
        le_i16(data)
    }
}
impl Parse for u32 {
    fn parse(data: &[u8]) -> PResult<Self> {
        le_u32(data)
    }
}
impl Parse for i32 {
    fn parse(data: &[u8]) -> PResult<Self> {
        le_i32(data)
    }
}
impl Parse for u64 {
    fn parse(data: &[u8]) -> PResult<Self> {
        le_u64(data)
    }
}
impl Parse for i64 {
    fn parse(data: &[u8]) -> PResult<Self> {
        le_i64(data)
    }
}
impl Parse for f32 {
    fn parse(data: &[u8]) -> PResult<Self> {
        le_f32(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DATA: &[u8] = &[
        0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,
    ];

    #[test]
    fn test_take() {
        let (res, bytes) = take(DATA, 4).unwrap();
        assert_eq!(bytes, &[0x1, 0x2, 0x3, 0x4]);
        assert_eq!(
            res,
            &[0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf]
        );
        let (res, bytes) = take(res, 2).unwrap();
        assert_eq!(bytes, &[0x5, 0x6]);
        assert_eq!(res, &[0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf]);
    }

    #[test]
    fn test_tag() {
        let (res, bytes) = tag(DATA, &[0x1, 0x2]).unwrap();
        assert_eq!(bytes, &[0x1, 0x2]);
        assert_eq!(
            res,
            &[0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf]
        );
    }

    #[test]
    fn test_take_until() {
        let (res, bytes) = take_until(DATA, 0x4).unwrap();
        assert_eq!(bytes, [0x1, 0x2, 0x3]);
        assert_eq!(
            res,
            &[0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf]
        );
    }

    #[test]
    fn test_count() {
        let (res, bytes) = count(DATA, |x| take(x, 2), 3).unwrap();
        assert_eq!(bytes.len(), 3);
        assert_eq!(bytes[0], &[0x1, 0x2]);
        assert_eq!(bytes[1], &[0x3, 0x4]);
        assert_eq!(bytes[2], &[0x5, 0x6]);
        assert_eq!(res, &[0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf]);
    }

    #[test]
    fn test_many() {
        let (res, bytes) = many(DATA, |x| take(x, 3)).unwrap();
        assert_eq!(res, &[]);
        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes[0], &[0x1, 0x2, 0x3]);
        assert_eq!(bytes[1], &[0x4, 0x5, 0x6]);
        assert_eq!(bytes[2], &[0x7, 0x8, 0x9]);
        assert_eq!(bytes[3], &[0xa, 0xb, 0xc]);
        assert_eq!(bytes[4], &[0xd, 0xe, 0xf]);
    }
}
