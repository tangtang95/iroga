use lzs::{Lzs, LzsError};
use nom::number::complete::le_i32;
use lzma_rs::error::Error as LzmaError;

use crate::Error;

pub fn lzss_decompress<R: std::io::Read, W: std::io::Write>(
    mut reader: R,
    mut writer: W
) -> Result<(), Error> {
    match Lzs::new(0x00).decompress(
        lzs::IOSimpleReader::new(&mut reader),
        lzs::IOSimpleWriter::new(&mut writer),
    ) {
        Err(LzsError::ReadError(e)) => Err(Error::Io(e)),
        Err(LzsError::WriteError(e)) => Err(Error::Io(e)),
        Ok(()) => Ok(()),
    }
}

pub fn lzma_decompress<R: std::io::Read, W: std::io::Write>(
    mut reader: R,
    mut writer: W
) -> Result<(), Error> {
    let mut header_bytes = [0u8; 8];
    reader.read_exact(&mut header_bytes)?;
    let bytes: &[u8] = &mut header_bytes;
    let (bytes, dec_size) = le_i32(bytes)?;
    let (_, prop_size) = le_i32(bytes)?;
    let mut buf_reader = std::io::BufReader::new(&mut reader);
    match if prop_size < 5 {
        lzma_rs::lzma2_decompress(&mut buf_reader, &mut writer)
    } else {
        let options = lzma_rs::decompress::Options {
            unpacked_size: lzma_rs::decompress::UnpackedSize::UseProvided(Some(dec_size as u64)),
            allow_incomplete: false,
            memlimit: None,
        };
        lzma_rs::lzma_decompress_with_options(&mut buf_reader, &mut writer, &options)
    } {
        Err(LzmaError::IoError(e)) => Err(Error::Io(e)),
        Err(LzmaError::HeaderTooShort(e)) => Err(Error::Io(e)),
        Err(LzmaError::LzmaError(s)) => {
            Err(Error::Io(std::io::Error::new(std::io::ErrorKind::Other, s)))
        },
        Err(LzmaError::XzError(s)) => {
            Err(Error::Io(std::io::Error::new(std::io::ErrorKind::Other, s)))
        },
        Ok(()) => Ok(()),
    }
}
