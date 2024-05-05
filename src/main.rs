use std::{
    io::{BufRead, BufReader, Seek, Write},
    path::PathBuf,
};

use clap::Parser;
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

const IRO_SIG: i32 = 0x534f5249; //IROS in bytes

#[allow(dead_code)]
const MIN_VERSION: i32 = 0x10000;
const MAX_VERSION: i32 = 0x10002;

#[derive(Clone)]
struct IroHeader {
    version: i32,
    flags: IroFlags,
    size: i32,
}

impl From<IroHeader> for Vec<u8> {
    fn from(value: IroHeader) -> Self {
        [
            IRO_SIG.to_le_bytes(),
            value.version.to_le_bytes(),
            (value.flags as i32).to_le_bytes(),
            value.size.to_le_bytes(),
        ]
        .concat()
    }
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
    let mod_name = match args.name {
        Some(name) => name,
        None => "mod".to_string(),
    };

    let dir_metadata = std::fs::metadata(&args.dir).unwrap();
    if !dir_metadata.is_dir() {
        println!("Given path is not a directory!");
        return;
    }

    let mut mod_file = std::fs::File::create(mod_name + ".iro").unwrap();
    let iro_header = IroHeader {
        version: MAX_VERSION,
        flags: IroFlags::None,
        size: 16,
    };
    let iro_header_bytes = Vec::from(iro_header.clone());
    let iro_header_size = iro_header_bytes.len() as u64;
    mod_file.write_all(iro_header_bytes.as_ref()).unwrap();
    let entries: Vec<DirEntry> = WalkDir::new(args.dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .collect();
    mod_file
        .write_all(&(entries.len() as u32).to_le_bytes())
        .unwrap();

    let entry_idx_bytes_size: u64 = entries
        .iter()
        .map(|e| {
            let unicode_filename: Vec<u8> =
                str::encode_utf16(e.to_owned().into_path().to_str().unwrap_or(""))
                    .flat_map(|ch| ch.to_le_bytes())
                    .collect();
            unicode_filename.len() as u64 + 16 + 4 // 16 +4 is indexing entry size
        })
        .sum();
    let mut offset: u64 = iro_header_size + 4 + entry_idx_bytes_size;
    mod_file.seek(std::io::SeekFrom::Start(offset)).unwrap();

    let mut positions: Vec<(u64, i32)> = Vec::with_capacity(entries.len());
    for entry in &entries {
        println!("{:?}", entry);
        let file = std::fs::File::open(entry.to_owned().into_path()).unwrap();
        let entry_offset = offset;
        let mut reader = BufReader::new(file);
        loop {
            let bytes = reader.fill_buf().unwrap();
            let consumed = bytes.len();
            if consumed == 0 {
                break;
            }
            mod_file.write_all(bytes).unwrap();
            reader.consume(consumed);
            offset += consumed as u64;
        }
        positions.push((entry_offset, (offset - entry_offset) as i32));
    }

    // indexing data
    mod_file
        .seek(std::io::SeekFrom::Start(iro_header_size + 4))
        .unwrap();
    for (entry, (pos, size)) in entries.iter().zip(positions) {
        let unicode_filename: Vec<u8> =
            str::encode_utf16(entry.to_owned().into_path().to_str().unwrap_or(""))
                .flat_map(|ch| ch.to_le_bytes())
                .collect();
        let len: u16 = unicode_filename.len() as u16 + 4 + 16;
        mod_file.write_all(&len.to_le_bytes()).unwrap();
        mod_file
            .write_all(&(unicode_filename.len().to_owned() as u16).to_le_bytes())
            .unwrap();
        mod_file.write_all(&unicode_filename).unwrap();
        mod_file.write_all(&0i32.to_le_bytes()).unwrap();
        mod_file.write_all(&pos.to_le_bytes()).unwrap();
        mod_file.write_all(&size.to_le_bytes()).unwrap();
    }
}
