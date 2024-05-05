use std::{
    io::{Read, Write},
    path::PathBuf,
};

use clap::Parser;
use walkdir::{DirEntry, WalkDir};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg()]
    dir: PathBuf,
}

enum IroFlags {
    None = 0,
    Patch = 1,
}

const IRO_SIG: i32 = 0x534f5249;
const VERSION: i32 = 0x10002;

struct IroHeader {
    version: i32,
    flags: IroFlags,
    dir: i32,
}

impl From<IroHeader> for Vec<u8> {
    fn from(value: IroHeader) -> Self {
        [
            IRO_SIG.to_le_bytes(),
            value.version.to_le_bytes(),
            (value.flags as i32).to_le_bytes(),
            value.dir.to_le_bytes(),
        ]
        .concat()
    }
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args.dir);

    let dir_metadata = std::fs::metadata(&args.dir).unwrap();
    if !dir_metadata.is_dir() {
        println!("Given path is not a directory!");
        return;
    }

    let mut file = std::fs::File::create("mod.iro").unwrap();
    let iro_header = IroHeader {
        version: VERSION,
        flags: IroFlags::None,
        dir: 16,
    };
    file.write_all(Vec::from(iro_header).as_ref()).unwrap();
    let entries: Vec<DirEntry> = WalkDir::new(args.dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .collect();
    file.write_all(&(entries.len() as u32).to_le_bytes()).unwrap();
    for entry in entries {
        println!("{:?}", entry);
    }
}
