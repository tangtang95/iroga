use std::{io::Write, path::PathBuf, process};

use clap::{Args, Parser, Subcommand};

use iroga::{pack_archive, unpack_archive};

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
    /// Unpack a IRO archive into a directory
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

    /// Files to include
    #[arg(short, long)]
    include: Option<Vec<String>>,

    /// Files to exclude
    #[arg(short, long)]
    exclude: Option<Vec<String>>,
}

#[derive(Args)]
struct UnpackArgs {
    /// IRO file to unpack
    #[arg()]
    iro_path: PathBuf,

    /// Output directory path (default is the name of the IRO to unpack)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Files to include
    #[arg(short, long)]
    include: Option<Vec<String>>,

    /// Files to exclude
    #[arg(short, long)]
    exclude: Option<Vec<String>>,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Pack(args) => {
            match pack_archive(args.dir, args.output, args.include, args.exclude) {
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
            }
        }
        Commands::Unpack(args) => {
            match unpack_archive(args.iro_path, args.output, args.include, args.exclude) {
                Ok(output_dir) => {
                    println!("IRO unpacked into \"{}\" directory", output_dir.display());
                    process::exit(0);
                }
                Err(err) => {
                    let stderr = std::io::stderr();
                    writeln!(stderr.lock(), "[iroga error]: {}", err).ok();
                    process::exit(1);
                }
            }
        }
    }
}
