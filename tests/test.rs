use assert_cmd::Command;
use assert_fs::{
    assert::PathAssert,
    fixture::{FileTouch, FileWriteBin, FileWriteStr, PathChild},
};
use hex_literal::hex;

#[test]
pub fn pack_not_exists_file() {
    let dir = assert_fs::TempDir::new().unwrap();
    iroga_cmd()
        .current_dir(dir.path())
        .arg("pack")
        .arg(dir.path().join("not_exists_file"))
        .assert()
        .failure()
        .code(1);
    assert!(!dir.child("not_exists_file").exists());
}

#[test]
pub fn pack_not_dir() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("not_dir").touch().unwrap();
    iroga_cmd()
        .arg("pack")
        .arg(dir.path().join("not_dir"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("not a directory"));
}

#[test]
pub fn pack_output_file_already_exists() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("dir/file.txt").touch().unwrap();
    dir.child("dir.iro").touch().unwrap();
    iroga_cmd()
        .current_dir(dir.path())
        .arg("pack")
        .arg("dir")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("output path already exists"));
}

#[test]
pub fn pack_single_file() {
    const EXPECTED_BYTES: &[u8] = &hex!(
        "49 52 4f 53 02 00 01 00   00 00 00 00 10 00 00 00"
        "01 00 00 00 24 00 10 00   66 00 69 00 6c 00 65 00"
        "2e 00 74 00 78 00 74 00   00 00 00 00 38 00 00 00"
        "00 00 00 00 17 00 00 00   48 65 6c 6c 6f 20 57 6f"
        "72 6c 64 21 0d 0a 0d 0a   48 69 21 0d 0a 0d 0a   "
    );
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("single/file.txt")
        .write_str("Hello World!\r\n\r\nHi!\r\n\r\n")
        .unwrap();

    iroga_cmd()
        .current_dir(dir.path())
        .arg("pack")
        .arg(dir.path().join("single"))
        .assert()
        .success()
        .code(0);

    assert!(dir.child("single.iro").exists());
    dir.child("single.iro").assert(EXPECTED_BYTES);
    dir.close().unwrap();
}

#[test]
pub fn pack_multiple_files() {
    const EXPECTED_BYTES: &[u8] = &hex!(
        "49 52 4f 53 02 00 01 00   00 00 00 00 10 00 00 00"
        "03 00 00 00 1e 00 0a 00   61 00 2e 00 74 00 78 00"
        "74 00 00 00 00 00 76 00   00 00 00 00 00 00 01 00"
        "00 00 1e 00 0a 00 62 00   2e 00 74 00 78 00 74 00"
        "00 00 00 00 77 00 00 00   00 00 00 00 01 00 00 00"
        "26 00 12 00 64 00 69 00   72 00 5c 00 63 00 2e 00"
        "74 00 78 00 74 00 00 00   00 00 78 00 00 00 00 00"
        "00 00 01 00 00 00 41 42   43                     "
    );
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("multiple/a.txt").write_str("A").unwrap();
    dir.child("multiple/b.txt").write_str("B").unwrap();
    dir.child("multiple/dir/c.txt").write_str("C").unwrap();

    iroga_cmd()
        .current_dir(dir.path())
        .arg("pack")
        .arg(dir.path().join("multiple"))
        .assert()
        .success()
        .code(0);

    assert!(dir.child("multiple.iro").exists());
    dir.child("multiple.iro").assert(EXPECTED_BYTES);
    dir.close().unwrap();
}

#[test]
pub fn unpack_not_exists_file() {
    let dir = assert_fs::TempDir::new().unwrap();
    iroga_cmd()
        .current_dir(dir.path())
        .arg("unpack")
        .arg(dir.path().join("not_exists_file.iro"))
        .assert()
        .failure()
        .code(1);
    assert!(!dir.child("not_exists_file.iro").exists());
}

#[test]
pub fn unpack_not_file() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("not_file/is_file").touch().unwrap();
    iroga_cmd()
        .arg("unpack")
        .arg(dir.path().join("not_file"))
        .assert()
        .failure()
        .code(1);
    assert!(dir.child("not_file").is_dir());
}

#[test]
pub fn unpack_output_path_already_exists() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("dir/file.txt").touch().unwrap();
    dir.child("dir.iro").touch().unwrap();
    iroga_cmd()
        .current_dir(dir.path())
        .arg("unpack")
        .arg("dir.iro")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("output path already exists"));
}

#[test]
pub fn unpack_single_file() {
    let iro_bytes: &[u8] = &hex!(
        "49 52 4f 53 02 00 01 00   00 00 00 00 10 00 00 00"
        "01 00 00 00 24 00 10 00   66 00 69 00 6c 00 65 00"
        "2e 00 74 00 78 00 74 00   00 00 00 00 38 00 00 00"
        "00 00 00 00 17 00 00 00   48 65 6c 6c 6f 20 57 6f"
        "72 6c 64 21 0d 0a 0d 0a   48 69 21 0d 0a 0d 0a   "
    );
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("single.iro")
        .write_binary(iro_bytes)
        .unwrap();

    iroga_cmd()
        .current_dir(dir.path())
        .arg("unpack")
        .arg(dir.path().join("single.iro"))
        .assert()
        .success()
        .code(0);

    assert!(dir.child("single/file.txt").exists());
    dir.child("single/file.txt").assert("Hello World!\r\n\r\nHi!\r\n\r\n");
    dir.close().unwrap();
}

#[test]
pub fn unpack_multiple_files() {
    let iro_bytes: &[u8] = &hex!(
        "49 52 4f 53 02 00 01 00   00 00 00 00 10 00 00 00"
        "03 00 00 00 1e 00 0a 00   61 00 2e 00 74 00 78 00"
        "74 00 00 00 00 00 76 00   00 00 00 00 00 00 01 00"
        "00 00 1e 00 0a 00 62 00   2e 00 74 00 78 00 74 00"
        "00 00 00 00 77 00 00 00   00 00 00 00 01 00 00 00"
        "26 00 12 00 64 00 69 00   72 00 5c 00 63 00 2e 00"
        "74 00 78 00 74 00 00 00   00 00 78 00 00 00 00 00"
        "00 00 01 00 00 00 41 42   43                     "
    );
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("multiple.iro")
        .write_binary(iro_bytes)
        .unwrap();

    iroga_cmd()
        .current_dir(dir.path())
        .arg("unpack")
        .arg(dir.path().join("multiple.iro"))
        .assert()
        .success()
        .code(0);

    dir.child("multiple/a.txt").assert("A");
    dir.child("multiple/b.txt").assert("B");
    dir.child("multiple/dir/c.txt").assert("C");
    dir.close().unwrap();
}

fn iroga_cmd() -> Command {
    Command::cargo_bin("iroga").unwrap()
}
