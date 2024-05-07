use std::{
    io::{BufRead, BufReader, Seek, Write},
    path::{Path, PathBuf},
    process,
    result::Result,
};

use clap::Parser;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

/// Command line tool to pack a single directory into a single archive in iro format
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory to pack into iro
    #[arg()]
    dir: PathBuf,

    /// Name of the file (default name: "mod")
    #[arg(short, long)]
    name: Option<String>,
}

#[derive(Clone)]
#[allow(dead_code)]
enum IroFlags {
    None = 0,
    Patch = 1,
}

const IRO_SIG: i32 = 0x534f5249; // represents IROS text

#[allow(dead_code)]
const MIN_VERSION: i32 = 0x10000;
const MAX_VERSION: i32 = 0x10002;

#[derive(Clone)]
struct IroHeader {
    version: i32,
    flags: IroFlags,
    size: i32,
    num_files: u32,
}

impl From<IroHeader> for Vec<u8> {
    fn from(value: IroHeader) -> Self {
        [
            IRO_SIG.to_le_bytes(),
            value.version.to_le_bytes(),
            (value.flags as i32).to_le_bytes(),
            value.size.to_le_bytes(),
            value.num_files.to_le_bytes(),
        ]
        .concat()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    StripPrefix(#[from] ::std::path::StripPrefixError),
    #[error("{0} is not a directory")]
    NotDir(PathBuf),
    #[error("{0} has invalid unicode")]
    InvalidUnicode(PathBuf),
}

fn main() {
    let args = Args::parse();
    let mod_name = match args.name {
        Some(name) => name,
        None => "mod".to_string(),
    } + ".iro";

    match pack_archive(mod_name.clone(), args.dir) {
        Ok(_) => {
            println!("archive \"{}\" has been created!", mod_name);
            process::exit(0);
        }
        Err(err) => {
            let stderr = std::io::stderr();
            writeln!(stderr.lock(), "[iropack error]: {}", err).ok();
            process::exit(1);
        }
    };
}

fn pack_archive(mod_name: String, dir_to_archive: PathBuf) -> Result<(), Error> {
    let dir_metadata = std::fs::metadata(&dir_to_archive)?;
    if !dir_metadata.is_dir() {
        return Err(Error::NotDir(dir_to_archive));
    }

    // Remove mod file first to avoid including it in the archive
    std::fs::remove_file(&mod_name).ok();
    let entries: Vec<DirEntry> = {
        let mut entries: Vec<DirEntry> = WalkDir::new(&dir_to_archive)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
            .collect();
        entries.sort_by(|a, b| a.path().partial_cmp(b.path()).unwrap());
        entries
    };
    let mut mod_file = std::fs::File::create(mod_name)?;

    // IRO Header
    let iro_header = IroHeader {
        version: MAX_VERSION,
        flags: IroFlags::None,
        size: 16,
        num_files: entries.len() as u32,
    };
    let iro_header_bytes = Vec::from(iro_header.clone());
    let iro_header_size = iro_header_bytes.len() as u64;
    mod_file.write_all(iro_header_bytes.as_ref())?;

    let mut offset = iro_header_size;
    for entry in &entries {
        let unicode_filepath: Vec<u8> =
            unicode_filepath_bytes(entry.path(), dir_to_archive.as_path())?;
        offset += unicode_filepath.len() as u64 + 16 + 4 // 16 + 4 is indexing entry size
    }
    mod_file.seek(std::io::SeekFrom::Start(offset))?;

    let mut positions: Vec<(u64, i32)> = Vec::with_capacity(entries.len());
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
        positions.push((entry_offset, (offset - entry_offset) as i32));
    }

    // indexing data
    mod_file.seek(std::io::SeekFrom::Start(iro_header_size))?;
    for (entry, (pos, size)) in entries.iter().zip(positions) {
        let unicode_filepath: Vec<u8> =
            unicode_filepath_bytes(entry.path(), dir_to_archive.as_path())?;
        let len: u16 = unicode_filepath.len() as u16 + 4 + 16;
        mod_file.write_all(&len.to_le_bytes())?;
        mod_file.write_all(&(unicode_filepath.len().to_owned() as u16).to_le_bytes())?;
        mod_file.write_all(&unicode_filepath)?;
        mod_file.write_all(&0i32.to_le_bytes())?;
        mod_file.write_all(&pos.to_le_bytes())?;
        mod_file.write_all(&size.to_le_bytes())?;
    }

    Ok(())
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
