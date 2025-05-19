[![CI](https://github.com/tangtang95/iroga/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/tangtang95/iroga/actions/workflows/ci.yaml)
# Iroga

Command line application to pack a single directory into an IRO archive.
The IRO archive is a format used in [7th heaven](https://github.com/tsunamods-codes/7th-Heaven), a FF7 mod manager application

## CLI Usage

```sh
# Simple usage
iroga pack <DIR>

# For help information
iroga --help
```

## Lib Usage

```rust
use iroga::error::Error;
use iroga::iro_archive::IroArchive;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

let iro_file = File::open(Path::new("foobar.iro"))?;
let mut iro_archive = IroArchive::open(iro_file);
let iro_header = iro_archive.read_header()?;
let iro_entries = iro_archive.read_iro_entries(&iro_header)?;

for iro_entry in iro_entries {
    let mut buf_writer = BufWriter::new(Vec::new());
    iro_archive.seek_and_read_file_entry(&iro_entry, &mut buf_writer)?;
}
```

## IRO format

| Offset | Size | Description |
| ------------- | -------------- | -------------- |
| 0x00 | 20 | IRO Header |
| 0x20 | (20 + L) * N | File indexing section |
| 0x20 + (20 + L) * N | B * N | Data section |

> N is the number of files, L is the dynamic length of file paths, B is the dynamic byte length of the files

### IRO Header

| Offset | Size | Description |
| ------------- | -------------- | -------------- |
| 0x00 | 4 | `IROS` constant text in ASCII |
| 0x04 | 4 | Version (latest version: `0x10002`) |
| 0x08 | 4 | Flags (`0`: full IRO, `1`: patch) |
| 0x0C | 4 | Size of IRO header (always `16`) |
| 0x10 | 4 | Number of files inside the archive |

### File indexing section

Section repeated for each file inside the archive

| Offset | Size | Description |
| ------------- | -------------- | -------------- |
| 0x00 | 2 | Length of this section (`filepath_length + 20`) |
| 0x02 | 2 | Length of the file path |
| 0x04 | L | File path in unicode UTF16 |
| 0x04 + L  | 4 | File flags (`0`: Non-compressed, `1`: LZSS-compressed, `2`: LZMA-compressed) |
| 0x04 + L + 0x04 | 8 | IRO archive offset pointing to the related file in data section (Size=4 on version 0x10000) |
| 0x04 + L + 0x0C | 4 | Length of the data |

### Data section

Concatenation of bytes of each file
