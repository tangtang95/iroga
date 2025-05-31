use std::{
    io::{BufReader, Read, Seek, Write},
    result::Result,
};

use crate::Error;
use crate::compression;
use crate::iro_entry::FileFlags;
pub use crate::iro_entry::IroEntry;
pub use crate::iro_header::IroHeader;
use crate::iro_parser::{parse_iro_entry_v2, parse_iro_header_v2};

pub struct IroArchive<RW> {
    stream: RW,
}

impl<R: Read + Seek> IroArchive<R> {
    pub fn open(stream: R) -> Self {
        IroArchive { stream }
    }

    pub fn read_header(&mut self) -> Result<IroHeader, Error> {
        let mut iro_header_bytes = [0u8; 20];
        self.stream.read_exact(&mut iro_header_bytes)?;
        let (_, iro_header) = parse_iro_header_v2(&iro_header_bytes)?;
        Ok(iro_header)
    }

    pub fn read_iro_entries(&mut self, iro_header: &IroHeader) -> Result<Vec<IroEntry>, Error> {
        let mut iro_entries: Vec<IroEntry> = Vec::new();
        for _ in 0..iro_header.num_files {
            let mut entry_len_bytes = [0u8; 2];
            self.stream.read_exact(&mut entry_len_bytes)?;
            let entry_len = u16::from_le_bytes(entry_len_bytes);

            let mut entry_bytes = vec![0u8; entry_len as usize - 2];
            self.stream.read_exact(entry_bytes.as_mut())?;

            let (_, iro_entry) = parse_iro_entry_v2(iro_header, &entry_bytes)?;
            iro_entries.push(iro_entry);
        }
        Ok(iro_entries)
    }

    pub fn seek_and_read_file_entry<W: Write>(
        &mut self,
        iro_entry: &IroEntry,
        writer: &mut W
    ) -> Result<(), Error> {
        let mut buf_reader = BufReader::new(&mut self.stream);
        buf_reader.seek(std::io::SeekFrom::Start(iro_entry.offset))?;
        let mut entry_buffer = buf_reader.take(iro_entry.data_len as u64);
        match iro_entry.flags {
            FileFlags::LzssCompressed => compression::lzss_decompress(&mut entry_buffer, writer)?,
            FileFlags::LzmaCompressed => compression::lzma_decompress(&mut entry_buffer, writer)?,
            _ => {
                std::io::copy(&mut entry_buffer, writer)?;
            }
        };
        Ok(())
    }
}
