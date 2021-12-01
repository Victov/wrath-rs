use anyhow::Result;
use byteorder::ReadBytesExt;

#[inline]
pub fn read_sized_string<R: std::io::Read>(reader: &mut R, size: usize) -> Result<String> {
    let mut bytes = vec![0; size];
    reader.read_exact(&mut bytes)?;
    let str = std::str::from_utf8(&bytes)?;
    Ok(str.to_string())
}

#[inline]
pub fn read_sized_bytes<R: std::io::Read>(reader: &mut R, size: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0; size];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

#[inline]
pub fn read_sized_string_with_len_field_u8<R: std::io::Read>(reader: &mut R) -> Result<String> {
    let length = reader.read_u8()? as usize;
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    let str = std::str::from_utf8(&bytes)?;
    Ok(str.to_string())
}
