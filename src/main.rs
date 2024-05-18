mod iro_entry;
mod iro_header;
mod iro_parser;

use std::{
    io::{BufRead, BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
    process,
    result::Result,
};

use clap::{Args, Parser, Subcommand};
use iro_entry::{FileFlags, IroEntry, INDEX_FIXED_BYTE_SIZE};
use iro_header::{IroFlags, IroHeader, IroVersion};
use iro_parser::{parse_iro_entry_v2, parse_iro_header_v2};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

/// Command line tool to pack a single directory into a single archive in IRO format
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pack a single directory into an IRO archive
    Pack(PackArgs),
    Unpack(UnpackArgs),
}

#[derive(Args)]
struct PackArgs {
    /// Directory to pack
    #[arg()]
    dir: PathBuf,

    /// Output file path (default is the name of the dir to pack)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args)]
struct UnpackArgs {
    /// IRO file to unpack
    #[arg()]
    iro_path: PathBuf,

    /// Output directory path (default is the name of the IRO to unpack)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    StripPrefix(#[from] ::std::path::StripPrefixError),
    #[error("{0} is not a directory")]
    NotDir(PathBuf),
    #[error("output path already exists: {0}")]
    OutputPathExists(PathBuf),
    #[error("{0} has invalid unicode")]
    InvalidUnicode(PathBuf),
    #[error("could not find default name from {0}")]
    CannotDetectDefaultName(PathBuf),
    #[error("parsing error due to invalid iro flags {0}")]
    InvalidIroFlags(i32),
    #[error("failed to parse binary data")]
    NomParseError(nom::Err<::nom::error::Error<Vec<u8>>>),
    #[error("parsing error due to invalid file flags {0}")]
    InvalidFileFlags(i32),
    #[error("utf16 error {0}")]
    Utf16Error(String),
    #[error("parten file path does not exists: {0}")]
    ParentPathDoesNotExist(PathBuf),
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for Error {
    fn from(err: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Self::NomParseError(err.map_input(|input| input.into()))
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Pack(args) => match pack_archive(args.dir, args.output) {
            Ok(output_filename) => {
                println!(
                    "archive \"{}\" has been created!",
                    output_filename.display()
                );
                process::exit(0);
            }
            Err(err) => {
                let stderr = std::io::stderr();
                writeln!(stderr.lock(), "[iroga error]: {}", err).ok();
                process::exit(1);
            }
        },
        Commands::Unpack(args) => match unpack_archive(args.iro_path, args.output) {
            Ok(output_dir) => {
                println!("iro unpacked into \"{}\" directory", output_dir.display());
                process::exit(0);
            }
            Err(err) => {
                let stderr = std::io::stderr();
                writeln!(stderr.lock(), "[iroga error]: {}", err).ok();
                process::exit(1);
            }
        },
    }
}

fn pack_archive(dir_to_pack: PathBuf, output_path: Option<PathBuf>) -> Result<PathBuf, Error> {
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

fn unpack_archive(iro_path: PathBuf, output_path: Option<PathBuf>) -> Result<PathBuf, Error> {
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

    let iro_file = std::fs::File::open(&iro_path)?;
    let mut buf_reader = BufReader::new(&iro_file);
    let bytes = buf_reader.fill_buf()?;
    let (rem_bytes, iro_header) = parse_iro_header_v2(bytes)?;
    let consumed_bytes_len = bytes.len() - rem_bytes.len();
    buf_reader.consume(consumed_bytes_len);

    println!("{:?}", iro_header);

    let mut iro_entries: Vec<IroEntry> = Vec::new();
    for _ in 0..iro_header.num_files {
        let bytes = buf_reader.fill_buf()?;
        let (rem_bytes, iro_entry) = parse_iro_entry_v2(bytes)?;
        println!("{:?}", iro_entry);

        iro_entries.push(iro_entry);
        let consumed_bytes_len = bytes.len() - rem_bytes.len();
        buf_reader.consume(consumed_bytes_len);
    }

    for iro_entry in iro_entries {
        let iro_path = parse_utf16(&iro_entry.path)?.replace('\\', "/");
        let iro_path = output_path.join(iro_path);
        std::fs::create_dir_all(
            iro_path
                .parent()
                .ok_or(Error::ParentPathDoesNotExist(iro_path.clone()))?,
        )?;
        let mut entry_file = std::fs::File::create(&iro_path).unwrap();

        let mut buf_reader = BufReader::new(&iro_file);
        buf_reader.seek(std::io::SeekFrom::Start(iro_entry.offset))?;
        let mut entry_buffer = buf_reader.take(iro_entry.data_len as u64);
        std::io::copy(&mut entry_buffer, &mut entry_file)?;
    }

    Ok(output_path)
}

fn parse_utf16(path_bytes: &[u8]) -> Result<String, Error> {
    let bytes_u16 = path_bytes
        .chunks(2)
        .map(|e| e.try_into().map(u16::from_le_bytes))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| Error::Utf16Error("uneven bytes".to_owned()))?;

    String::from_utf16(&bytes_u16).map_err(|_| {
        Error::Utf16Error("path_bytes in u16 cannot be converted to string".to_owned())
    })
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
