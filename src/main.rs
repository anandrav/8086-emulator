use std::env;
use std::fs;
use std::io::Result;

fn read_file_to_bytes(filename: &str) -> Result<Vec<u8>> {
    fs::read(filename)
}

const D_MASK: u8 = 0b00000010;
const W_MASK: u8 = 0b00000001;
const MODE_MASK: u8 = 0b11000000;
const REG_MASK: u8 = 0b00111000;
const RM_MASK: u8 = 0b00000111;

fn main() {
    let args: Vec<_> = env::args().collect();
    let filename = &args[1];
    let bytes = read_file_to_bytes(filename).unwrap();

    println!("bits 16");
    for i in 0..bytes.len() / 2 {
        let b1 = bytes[i * 2];
        let b2 = bytes[i * 2 + 1];

        let d = b1 & D_MASK;
        let w = b1 & W_MASK;

        let mode = (b2 & MODE_MASK) >> 6;
        assert_eq!(mode, 0b11);
        let reg = (b2 & REG_MASK) >> 3;
        let rm = b2 & RM_MASK;

        let mut register1 = register_field_encoding(reg, w);
        let mut register2 = register_field_encoding(rm, w);
        if d == 0 {
            (register1, register2) = (register2, register1);
        }

        println!("mov {}, {}", register1, register2);
    }
}

fn register_field_encoding(reg: u8, w: u8) -> &'static str {
    match (reg, w) {
        (0b000, 0) => "al",
        (0b000, 1) => "ax",
        (0b001, 0) => "cl",
        (0b001, 1) => "cx",
        (0b010, 0) => "dl",
        (0b010, 1) => "dx",
        (0b011, 0) => "bl",
        (0b011, 1) => "bx",
        (0b100, 0) => "ah",
        (0b100, 1) => "sp",
        (0b101, 0) => "ch",
        (0b101, 1) => "bp",
        (0b110, 0) => "dh",
        (0b110, 1) => "si",
        (0b111, 0) => "bh",
        (0b111, 1) => "di",
        _ => panic!("reg={}, w={}", reg, w),
    }
}
