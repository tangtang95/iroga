use crate::Error;

pub const INDEX_FIXED_BYTE_SIZE: usize = 20;

#[derive(Debug)]
pub struct IroEntry {
    pub path: Vec<u8>,
    pub flags: FileFlags,
    pub offset: u64,
    pub data_len: u32,
}

#[derive(Debug)]
pub enum FileFlags {
    Uncompressed = 0,
}

impl IroEntry {
    pub fn new(path: Vec<u8>, flags: FileFlags, offset: u64, data_len: u32) -> Self {
        IroEntry {
            path,
            flags,
            offset,
            data_len,
        }
    }
}

impl From<IroEntry> for Vec<u8> {
    fn from(value: IroEntry) -> Self {
        let mut bytes = Vec::new();
        bytes.extend(((value.path.len() + INDEX_FIXED_BYTE_SIZE) as u16).to_le_bytes());
        bytes.extend((value.path.len() as u16).to_le_bytes());
        bytes.extend(value.path);
        bytes.extend((value.flags as i32).to_le_bytes());
        bytes.extend(value.offset.to_le_bytes());
        bytes.extend(value.data_len.to_le_bytes());
        bytes
    }
}

impl TryFrom<i32> for FileFlags {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FileFlags::Uncompressed),
            _ => Err(Error::InvalidFileFlags(value))
        }
    }
}
