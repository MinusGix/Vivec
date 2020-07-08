use crate::records::common::BStrw;
use bstr::ByteSlice;
use std::io::Write;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position<T: Copy + Clone + PartialEq> {
    pub x: T,
    pub y: T,
}
impl<T> Position<T>
where
    T: Copy + Clone + PartialEq,
{
    pub fn new(x: T, y: T) -> Position<T> {
        Position { x, y }
    }
}
// Allow types that implement Eq to also have position have Eq
impl<T> Eq for Position<T> where T: Copy + Clone + PartialEq + Eq {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position3<T: Copy + Clone + PartialEq> {
    pub x: T,
    pub y: T,
    pub z: T,
}
impl<T> Position3<T>
where
    T: Copy + Clone + PartialEq,
{
    pub fn new(x: T, y: T, z: T) -> Position3<T> {
        Position3 { x, y, z }
    }
}
impl<T> Eq for Position3<T> where T: Copy + Clone + PartialEq + Eq {}

impl<T> std::convert::From<Position<T>> for Position3<T>
where
    T: Copy + Clone + PartialEq + Default,
{
    fn from(p: Position<T>) -> Self {
        Position3::new(p.x, p.y, Default::default())
    }
}

pub mod byte {
    /// Converts bytes into u16
    /// little endian
    pub fn as_u16(v: &[u8]) -> u16 {
        (u16::from(v[0]) << 8) | u16::from(v[1])
    }

    /// Converts bytes into u32
    /// little endian
    pub fn as_u32(v: &[u8]) -> u32 {
        (u32::from(v[0]) << 24) | (u32::from(v[1]) << 16) | (u32::from(v[2]) << 8) | u32::from(v[3])
    }

    /// Little endian
    pub fn as_f32(v: &[u8]) -> f32 {
        f32::from_le_bytes([v[0], v[1], v[2], v[3]])
    }

    /// Converts bytes into u64
    /// little endian
    pub fn as_u64(v: &[u8]) -> u64 {
        (u64::from(v[0]) << 56)
            | (u64::from(v[1]) << 48)
            | (u64::from(v[2]) << 40)
            | (u64::from(v[3]) << 32)
            | (u64::from(v[4]) << 24)
            | (u64::from(v[5]) << 16)
            | (u64::from(v[6]) << 8)
            | u64::from(v[7])
    }

    pub fn as_4_bytes(b: &[u8]) -> [u8; 4] {
        [b[0], b[1], b[2], b[3]]
    }
}

pub fn fmt_data<T: std::fmt::Debug>(
    debug_struct: &mut std::fmt::DebugStruct,
    name: &str,
    data: &[T],
    limit: usize,
) {
    if data.len() > limit {
        debug_struct.field(name, &format!("({}) {:x?}..", data.len(), &data[0..limit]));
    } else {
        debug_struct.field(name, &data);
    }
}

/// usage: dispatch_all(SomeEnum, value, [Alpha, Beta, Delta, Omega], x, { x.count_ones() })
/// matches against all enum entries, calling the function on each of their held values
/// This is painfully complicated
#[macro_export]
macro_rules! dispatch_all {
    ( $enumer:ident $( :: $enumer_t:ident)*, $value:expr, [$($name:ident),*], $field:ident, $code:tt) => {
        $crate::dispatch_all!{
            @match
            ( $enumer $(:: $enumer_t)*),
            $value,
            [$($name),*],
            $field,
            $code
        }
    };
    (@match $enumer:tt, $value:expr, [$($name:ident),*], $field:ident, $code:tt )=>{
        match $value {
            $(
                $crate::dispatch_all!(@pati $enumer, $name, $field ) =>
                    $code,
            )*
        }
    };
    (@pati ($($enumer:tt)*), $name:ident, $field:ident )=>{
        $($enumer)*::$name ($field)
    };
}

pub trait Writable {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write;
}
impl Writable for bool {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        (*self as u8).write_to(w)
    }
}
impl Writable for u8 {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(&[*self])
    }
}
impl Writable for i8 {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(&self.to_le_bytes())
    }
}
impl Writable for u16 {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(&self.to_le_bytes())
    }
}
impl Writable for i16 {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(&self.to_le_bytes())
    }
}
impl Writable for u32 {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(&self.to_le_bytes())
    }
}
impl Writable for i32 {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(&self.to_le_bytes())
    }
}
impl Writable for u64 {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(&self.to_le_bytes())
    }
}
impl Writable for i64 {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(&self.to_le_bytes())
    }
}
impl Writable for f32 {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(&self.to_le_bytes())
    }
}
impl<'aleph> Writable for &'aleph bstr::BStr {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        w.write_all(self)
    }
}
impl<'aleph> Writable for BStrw<'aleph> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.as_bstr().write_to(w)
    }
}
impl<'aleph, U> Writable for &[U]
where
    U: Writable,
{
    /// Note: this does not include the size of the slice!
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        for i in 0..self.len() {
            self[i].write_to(w)?;
        }
        Ok(())
    }
}
impl<'aleph, U> Writable for Vec<U>
where
    U: Writable,
{
    /// Note: this does not include the size of the vector!
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.as_slice().write_to(w)
    }
}
impl<U> Writable for Position<U>
where
    U: Sized + Copy + Clone + PartialEq + Writable,
{
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.x.write_to(w)?;
        self.y.write_to(w)
    }
}
impl<U> Writable for Position3<U>
where
    U: Sized + Copy + Clone + PartialEq + Writable,
{
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.x.write_to(w)?;
        self.y.write_to(w)?;
        self.z.write_to(w)
    }
}

/// DataSize function that does not depend upon value
pub trait StaticDataSize {
    fn static_data_size() -> usize;
}
impl StaticDataSize for bool {
    fn static_data_size() -> usize {
        1
    }
}
impl StaticDataSize for u8 {
    fn static_data_size() -> usize {
        1
    }
}
impl StaticDataSize for i8 {
    fn static_data_size() -> usize {
        1
    }
}
impl StaticDataSize for u16 {
    fn static_data_size() -> usize {
        2
    }
}
impl StaticDataSize for i16 {
    fn static_data_size() -> usize {
        2
    }
}
impl StaticDataSize for u32 {
    fn static_data_size() -> usize {
        4
    }
}
impl StaticDataSize for i32 {
    fn static_data_size() -> usize {
        4
    }
}
impl StaticDataSize for u64 {
    fn static_data_size() -> usize {
        8
    }
}
impl StaticDataSize for i64 {
    fn static_data_size() -> usize {
        8
    }
}
impl StaticDataSize for f32 {
    fn static_data_size() -> usize {
        4
    }
}
impl<T> StaticDataSize for Position<T>
where
    T: Sized + Copy + Clone + PartialEq + StaticDataSize,
{
    fn static_data_size() -> usize {
        T::static_data_size() + T::static_data_size() // size(x) + size(y)
    }
}
impl<T> StaticDataSize for Position3<T>
where
    T: Sized + Copy + Clone + PartialEq + StaticDataSize,
{
    fn static_data_size() -> usize {
        T::static_data_size() + T::static_data_size() + T::static_data_size()
    }
}

pub trait DataSize {
    fn data_size(&self) -> usize;
}
impl<T> DataSize for T
where
    T: StaticDataSize,
{
    fn data_size(&self) -> usize {
        T::static_data_size()
    }
}
impl<'aleph> DataSize for &'aleph bstr::BStr {
    fn data_size(&self) -> usize {
        self.len()
    }
}
impl<'aleph> DataSize for BStrw<'aleph> {
    fn data_size(&self) -> usize {
        self.as_bstr().data_size()
    }
}
impl<'aleph, T> DataSize for &[T]
where
    T: DataSize,
{
    fn data_size(&self) -> usize {
        let mut data_size: usize = 0;
        for i in 0..self.len() {
            data_size += self[i].data_size();
        }
        data_size
    }
}
impl<'aleph, T> DataSize for Vec<T>
where
    T: DataSize,
{
    fn data_size(&self) -> usize {
        self.as_slice().data_size()
    }
}
impl<'aleph, T> DataSize for Option<T>
where
    T: DataSize,
{
    fn data_size(&self) -> usize {
        match self {
            Some(x) => x.data_size(),
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_size_slice() {
        let data: &[u32] = &[42, 92, 5, 4, 92];
        assert_eq!(data.data_size(), 20);
    }

    #[test]
    fn data_slice_vec() {
        let data: Vec<u32> = vec![42, 92, 5, 4, 92];
        assert_eq!(data.data_size(), 20);
    }
}
