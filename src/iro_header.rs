use std::fmt::Display;

use crate::Error;

pub const IRO_SIG: i32 = 0x534f5249; // represents IROS text

#[derive(Clone, Debug)]
pub struct IroHeader {
    pub version: IroVersion,
    pub flags: IroFlags,
    pub size: i32,
    pub num_files: u32,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum IroFlags {
    None = 0,
    Patch = 1,
}

#[derive(Clone, Debug)]
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

impl TryFrom<i32> for IroFlags {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(IroFlags::None),
            1 => Ok(IroFlags::Patch),
            _ => Err(Error::InvalidIroFlags(value)),
        }
    }
}

impl Display for IroFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IroFlags::None => f.write_str("Full IRO"),
            IroFlags::Patch => f.write_str("Patch IRO"),
        }
    }
}

impl Display for IroVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IroVersion::Zero => f.write_str("0x10000"),
            IroVersion::Two => f.write_str("0x10002"),
        }
    }
}
