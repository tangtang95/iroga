pub const INDEX_FIXED_BYTE_SIZE: usize = 20;

pub struct IroEntry {
    path: Vec<u8>,
    flags: FileFlags,
    offset: u64,
    data_len: u32,
}

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
