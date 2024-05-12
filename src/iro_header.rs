const IRO_SIG: i32 = 0x534f5249; // represents IROS text

#[derive(Clone)]
pub struct IroHeader {
    version: IroVersion,
    flags: IroFlags,
    size: i32,
    num_files: u32,
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum IroFlags {
    None = 0,
    Patch = 1,
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum IroVersion {
    Zero = 0x10000,
    Two = 0x10002,
}

impl IroHeader {
    pub fn new(version: IroVersion, flags: IroFlags, size: i32, num_files: u32) -> Self {
        IroHeader {
            version,
            flags,
            size,
            num_files,
        }
    }
}

impl From<IroHeader> for Vec<u8> {
    fn from(value: IroHeader) -> Self {
        [
            IRO_SIG.to_le_bytes(),
            (value.version as i32).to_le_bytes(),
            (value.flags as i32).to_le_bytes(),
            value.size.to_le_bytes(),
            value.num_files.to_le_bytes(),
        ]
        .concat()
    }
}
