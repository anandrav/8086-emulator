use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

fn run_command(command: &str, args: &[&str]) -> Output {
    Command::new(command)
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute {}", command))
}

fn get_decoder() -> PathBuf {
    let mut path = std::env::current_exe().expect("Failed to get current executable path");
    path.pop(); // Remove the test binary name
    path.pop(); // Remove `deps`
    path.push("decoder-8086"); // Add your binary name
    path
}

fn test_decode_reassemble(listing: &str) {
    // Step 1: Prepare binary input
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut listings_dir = manifest_dir;
    listings_dir.pop();
    listings_dir.push(format!("cmuratori/perfaware/part1/{listing}"));
    let input_binary = listings_dir.to_str().unwrap();

    dbg!(input_binary);
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_decoded = temp_dir.path().join("decoded.asm");
    let temp_decoded = temp_decoded.to_str().unwrap();
    let temp_reassembled = temp_dir.path().join("reassembled.bin");
    let temp_reassembled = temp_reassembled.to_str().unwrap();

    // Ensure output directory exists
    fs::create_dir_all("tests/output").expect("Failed to create output directory");

    // Step 2: Decode the binary
    let decode_output = run_command(get_decoder().to_str().unwrap(), &[input_binary]);
    assert!(
        decode_output.status.success(),
        "Decoder failed: {:?}",
        decode_output
    );

    fs::write(temp_decoded, decode_output.stdout)
        .expect("Failed to write decoded assembly to file");

    // Step 3: Reassemble the decoded file
    let reassemble_output = run_command("nasm", &[temp_decoded, "-o", temp_reassembled]);
    assert!(
        reassemble_output.status.success(),
        "Assembler failed: {:?}",
        reassemble_output
    );

    // Step 4: Compare original and reassembled binaries
    let original = fs::read(input_binary).expect("Failed to read original binary");
    let reassembled = fs::read(temp_reassembled).expect("Failed to read reassembled binary");

    assert_eq!(
        original, reassembled,
        "Reassembled binary differs from the original"
    );
}

#[test]
fn test_listing_0037() {
    test_decode_reassemble("listing_0037_single_register_mov");
}

#[test]
fn test_listing_0038() {
    test_decode_reassemble("listing_0038_many_register_mov");
}

#[test]
fn test_listing_0039() {
    test_decode_reassemble("listing_0039_more_movs");
}

#[test]
fn test_listing_0041() {
    test_decode_reassemble("listing_0041_add_sub_cmp_jnz");
}
