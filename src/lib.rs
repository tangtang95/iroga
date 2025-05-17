mod compression;
mod error;
mod iro_entry;
mod iro_header;
mod iro_parser;

use std::{
    io::{BufRead, BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
    result::Result,
};

use error::Error;
use iro_entry::{FileFlags, INDEX_FIXED_BYTE_SIZE, IroEntry};
use iro_header::{IroFlags, IroHeader, IroVersion};
use iro_parser::{parse_iro_entry_v2, parse_iro_header_v2};
use walkdir::{DirEntry, WalkDir};

fn glob_includes(files: &[String], entry_path: impl AsRef<[u8]>) -> bool {
    files.iter().any(|f| fast_glob::glob_match(f, &entry_path))
}

fn match_entry_path(
    entry_path: impl AsRef<[u8]>,
    include_files: &Option<Vec<String>>,
    exclude_files: &Option<Vec<String>>,
) -> bool {
    match (include_files, exclude_files) {
        (None, None) => true,
        (Some(includes), None) => glob_includes(includes, &entry_path),
        (None, Some(excludes)) => !glob_includes(excludes, &entry_path),
        (Some(includes), Some(excludes)) => {
            glob_includes(includes, &entry_path) && !glob_includes(excludes, &entry_path)
        }
    }
}

pub fn pack_archive(
    dir_to_pack: PathBuf,
    output_path: Option<PathBuf>,
    include_files: Option<Vec<String>>,
    exclude_files: Option<Vec<String>>,
) -> Result<PathBuf, Error> {
    let dir_metadata = std::fs::metadata(&dir_to_pack)?;
    if !dir_metadata.is_dir() {
        return Err(Error::NotDir(dir_to_pack));
    }

    // compute output filepath: either default generated name or given output_path
    let output_path = match output_path {
        Some(path) => path,
        None => {
            let abs_path = std::fs::canonicalize(&dir_to_pack)?;
            let mut filename = abs_path
                .file_name()
                .ok_or(Error::CannotDetectDefaultName(abs_path.clone()))?
                .to_owned();
            filename.push(".iro");
            Path::new(&filename).to_owned()
        }
    };

    // Do not create IRO archive if the output path already points to an existing file
    if std::fs::File::open(&output_path).is_ok() {
        return Err(Error::OutputPathExists(output_path));
    }

    let entries: Vec<DirEntry> = WalkDir::new(&dir_to_pack)
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| {
            let relative_path = e
                .path()
                .strip_prefix(dir_to_pack.as_path())
                .unwrap()
                .display()
                .to_string();
            match_entry_path(relative_path, &include_files, &exclude_files)
        })
        .collect();
    let mut mod_file = std::fs::File::create(&output_path)?;

    // IRO Header
    let iro_header = IroHeader::new(IroVersion::Two, IroFlags::None, 16, entries.len() as u32);
    let iro_header_bytes = Vec::from(iro_header);
    let iro_header_size = iro_header_bytes.len() as u64;
    mod_file.write_all(iro_header_bytes.as_ref())?;

    let mut offset = iro_header_size;
    for entry in &entries {
        let unicode_filepath: Vec<u8> =
            unicode_filepath_bytes(entry.path(), dir_to_pack.as_path())?;
        offset += (unicode_filepath.len() + INDEX_FIXED_BYTE_SIZE) as u64;
    }
    mod_file.seek(std::io::SeekFrom::Start(offset))?;

    let mut iro_entries: Vec<IroEntry> = Vec::with_capacity(entries.len());
    for entry in &entries {
        let file = std::fs::File::open(entry.to_owned().into_path())?;
        let entry_offset = offset;
        let mut reader = BufReader::new(file);
        loop {
            let bytes = reader.fill_buf()?;
            let consumed = bytes.len();
            if consumed == 0 {
                break;
            }
            mod_file.write_all(bytes)?;
            reader.consume(consumed);
            offset += consumed as u64;
        }
        iro_entries.push(IroEntry::new(
            unicode_filepath_bytes(entry.path(), dir_to_pack.as_path())?,
            FileFlags::Uncompressed,
            entry_offset,
            (offset - entry_offset) as u32,
        ));
    }

    // indexing data
    mod_file.seek(std::io::SeekFrom::Start(iro_header_size))?;
    for entry in iro_entries {
        mod_file.write_all(&Vec::from(entry))?;
    }

    Ok(output_path)
}

pub fn unpack_archive(
    iro_path: PathBuf,
    output_path: Option<PathBuf>,
    include_files: Option<Vec<String>>,
    exclude_files: Option<Vec<String>>,
) -> Result<PathBuf, Error> {
    // compute output filepath: either default generated name or given output_path
    let output_path = match output_path {
        Some(path) => path,
        None => {
            let filename = iro_path
                .file_name()
                .ok_or(Error::CannotDetectDefaultName(iro_path.clone()))?
                .to_str()
                .ok_or(Error::CannotDetectDefaultName(iro_path.clone()))?
                .trim_end_matches(".iro");
            Path::new(filename).to_owned()
        }
    };
    if std::fs::read_dir(&output_path).is_ok() {
        return Err(Error::OutputPathExists(output_path));
    }

    let mut iro_file = std::fs::File::open(&iro_path)?;
    let mut iro_header_bytes = [0u8; 20];
    iro_file.read_exact(&mut iro_header_bytes)?;
    let (_, iro_header) = parse_iro_header_v2(&iro_header_bytes)?;

    println!("IRO metadata");
    println!("- version: {}", iro_header.version);
    println!("- type: {}", iro_header.flags);
    println!("- number of files: {}", iro_header.num_files);
    println!();

    let mut iro_entries: Vec<IroEntry> = Vec::new();
    for _ in 0..iro_header.num_files {
        let mut entry_len_bytes = [0u8; 2];
        iro_file.read_exact(&mut entry_len_bytes)?;
        let entry_len = u16::from_le_bytes(entry_len_bytes);

        let mut entry_bytes = vec![0u8; entry_len as usize - 2];
        iro_file.read_exact(entry_bytes.as_mut())?;

        let (_, iro_entry) = parse_iro_entry_v2(&iro_header, &entry_bytes)?;

        iro_entries.push(iro_entry);
    }

    for iro_entry in iro_entries {
        let iro_entry_path = parse_utf16(&iro_entry.path)?.replace('\\', "/");

        if !match_entry_path(&iro_entry_path, &include_files, &exclude_files) {
            continue;
        }

        let entry_path = output_path.join(&iro_entry_path);
        std::fs::create_dir_all(
            entry_path
                .parent()
                .ok_or(Error::ParentPathDoesNotExist(entry_path.clone()))?,
        )?;
        let mut entry_file = std::fs::File::create(&entry_path).unwrap();

        let mut buf_reader = BufReader::new(&iro_file);
        buf_reader.seek(std::io::SeekFrom::Start(iro_entry.offset))?;
        let mut entry_buffer = buf_reader.take(iro_entry.data_len as u64);
        match iro_entry.flags {
            FileFlags::LzssCompressed => {
                compression::lzss_decompress(&mut entry_buffer, &mut entry_file)?
            }
            FileFlags::LzmaCompressed => {
                compression::lzma_decompress(&mut entry_buffer, &mut entry_file)?
            }
            _ => {
                std::io::copy(&mut entry_buffer, &mut entry_file)?;
            }
        }

        println!("\"{}\" file written!", iro_entry_path);
    }

    Ok(output_path)
}

fn parse_utf16(bytes: &[u8]) -> Result<String, Error> {
    let bytes_u16 = bytes
        .chunks(2)
        .map(|e| e.try_into().map(u16::from_le_bytes))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| Error::InvalidUtf16("uneven bytes".to_owned()))?;

    String::from_utf16(&bytes_u16)
        .map_err(|_| Error::InvalidUtf16("bytes in u16 cannot be converted to string".to_owned()))
}

fn unicode_filepath_bytes(path: &Path, strip_prefix_str: &Path) -> Result<Vec<u8>, Error> {
    Ok(path
        .strip_prefix(strip_prefix_str)?
        .to_str()
        .ok_or(Error::InvalidUnicode(path.to_owned()))?
        .replace('/', "\\")
        .encode_utf16()
        .flat_map(|ch| ch.to_le_bytes())
        .collect())
}
