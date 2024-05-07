[![CI](https://github.com/tangtang95/iropack-rs/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/tangtang95/iropack-rs/actions/workflows/ci.yaml)
# Iropack

Command line application to pack a single directory into an iro archive.
The iro archive is a format used in [7th heaven](https://github.com/tsunamods-codes/7th-Heaven), a FF7 mod manager application

## Usage

```sh
iropack-rs directory --name modname

# For help information
iropack-rs --help
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
| 0x04 + L  | 4 | File flags (`0`: Non-compressed, other unimplemented) |
| 0x04 + L + 0x04 | 8 | IRO archive offset pointing to the related file in data section |
| 0x04 + L + 0x0C | 4 | Length of the data |

### Data section

Concatenation of bytes of each file
