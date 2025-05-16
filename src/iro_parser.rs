use nom::{
    bytes::complete::{tag, take},
    number::complete::{le_i32, le_u16, le_u32, le_u64},
};

use crate::{
    iro_entry::{FileFlags, IroEntry},
    iro_header::{IroFlags, IroHeader, IroVersion, IRO_SIG},
    Error,
};

pub fn parse_iro_header_v2(bytes: &[u8]) -> Result<(&[u8], IroHeader), Error> {
    let (bytes, _) = tag(&IRO_SIG.to_le_bytes()[..])(bytes)?;
    let (bytes, version) = le_i32(bytes)?;
    let (bytes, flags) = le_i32(bytes)?;
    let (bytes, _) = tag(&16i32.to_le_bytes()[..])(bytes)?;
    let (bytes, num_files) = le_u32(bytes)?;

    Ok((
        bytes,
        IroHeader::new(IroVersion::try_from(version)?, IroFlags::try_from(flags)?, 16, num_files),
    ))
}

/// Parse IroEntry without considering length of entire block
pub fn parse_iro_entry_v2<'a>(header: &IroHeader, bytes: &'a [u8]) -> Result<(&'a [u8], IroEntry), Error> {
    let (bytes, filepath_len) = le_u16(bytes)?;
    let (bytes, filepath) = take(filepath_len)(bytes)?;
    let (bytes, file_flags) = le_i32(bytes)?;
    let (bytes, offset) = if header.version == IroVersion::Zero {
        let (bytes, offset) = le_u32(bytes)?;
        (bytes, offset as u64)
    } else {
        le_u64(bytes)?
    };
    let (bytes, data_len) = le_u32(bytes)?;

    Ok((
        bytes,
        IroEntry::new(
            filepath.to_vec(),
            FileFlags::try_from(file_flags)?,
            offset,
            data_len,
        ),
    ))
}
