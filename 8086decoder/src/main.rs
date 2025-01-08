use std::env;
use std::fs;
use std::io::Result;
use std::slice::Iter;

fn read_file_to_bytes(filename: &str) -> Result<Vec<u8>> {
    fs::read(filename)
}

const OP_MOV_REG_MEM_MASK: u8 = 0b11111100;
const OP_MOV_REG_MEM: u8 = 0b10001000;
const OP_MOV_IMM_REG_MASK: u8 = 0b11110000;
const OP_MOV_IMM_REG: u8 = 0b10110000;
const D_MASK: u8 = 0b00000010;
const W_MASK: u8 = 0b00000001;
const MODE_MASK: u8 = 0b11000000;
const REG_MASK: u8 = 0b00111000;
const RM_MASK: u8 = 0b00000111;
const OP_ARITH_REG_MEM_MASK: u8 = 0b11000100;
const OP_ARITH_REG_MEM: u8 = 0b00000000;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        panic!("wrong number of arguments");
    }
    let filename = &args[1];
    let bytes = read_file_to_bytes(filename).unwrap();

    println!("bits 16");
    let mut iter = bytes.iter();
    let mut nbytes = 0;
    while let Some(b1) = iter.next() {
        nbytes += 1;

        let mut d: u8 = 0;
        let mut w: u8 = 0;
        let mut mode: u8 = 0;
        let mut reg: u8 = 0;
        let mut rm: u8 = 0;

        let mut data: u16 = 0;

        if (b1 & OP_MOV_REG_MEM_MASK) == OP_MOV_REG_MEM
            || (b1 & OP_ARITH_REG_MEM_MASK) == OP_ARITH_REG_MEM
        {
            // println!("-- reg to/from mem --");
            let b2 = iter.next().unwrap();
            nbytes += 1;

            d = b1 & D_MASK;
            w = b1 & W_MASK;

            mode = (b2 & MODE_MASK) >> 6;
            reg = (b2 & REG_MASK) >> 3;
            rm = b2 & RM_MASK;
        } else if (b1 & OP_MOV_IMM_REG_MASK) == OP_MOV_IMM_REG {
            // println!("-- imm to reg --");

            w = (b1 & 0b00001000) >> 3;
            // println!("w={}", w);
            reg = b1 & 0b00000111;

            let b2 = iter.next().unwrap();
            nbytes += 1;

            data = *b2 as u16;
            if w != 0 {
                let hi = *iter.next().unwrap() as u16;
                data |= hi << 8;
                nbytes += 1;
            }
        } else {
            panic!("Unrecognized opcode: {:b}, nbytes={}", b1, nbytes);
        }

        if (b1 & OP_MOV_REG_MEM_MASK) == OP_MOV_REG_MEM {
            if mode == 3 {
                // register to register
                let mut register1 = register_field_encoding(reg, w);
                let mut register2 = register_field_encoding(rm, w);
                if d == 0 {
                    (register1, register2) = (register2, register1);
                }

                println!("mov {}, {}", register1, register2);
            } else {
                // register to/from memory
                let register = register_field_encoding(reg, w);
                let addr = effective_address_calculation(rm, mode, &mut iter, &mut nbytes);
                if d == 0 {
                    println!("mov {}, {}", addr, register);
                } else {
                    println!("mov {}, {}", register, addr);
                }
            }
        } else if (b1 & OP_MOV_IMM_REG_MASK) == OP_MOV_IMM_REG {
            // println!("-- imm to reg --");

            println!("mov {}, {}", register_field_encoding(reg, w), data);
        } else {
            panic!("Unrecognized opcode: {:b}, nbytes={}", b1, nbytes);
        }
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
        _ => panic!("unhandled: reg={}, w={}", reg, w),
    }
}

// direct address is not handled here
fn effective_address_calculation(
    rm: u8,
    mode: u8,
    iter: &mut Iter<'_, u8>,
    nbytes: &mut i32,
) -> String {
    if rm == 0b110 && mode == 0b00 {
        // direct address
        let mut data: u16 = *iter.next().unwrap() as u16;
        *nbytes += 1;

        let hi = *iter.next().unwrap() as u16;
        data |= hi << 8;
        *nbytes += 1;
        return data.to_string();
    }
    let mut ret = "[".to_string();
    ret.push_str(match rm {
        0 => "bx + si",
        1 => "bx + di",
        2 => "bp + si",
        3 => "bp + di",
        4 => "si",
        5 => "di",
        6 => "bp",
        7 => "bx",
        _ => panic!("unhandled: reg={}, mod={}", rm, mode),
    });

    match mode {
        0 => { /* noop */ }
        1 => {
            let data = iter.next().unwrap();
            *nbytes += 1;
            if *data != 0 {
                ret.push_str(" + ");
                ret.push_str(&data.to_string());
            }
        }
        2 => {
            let mut data: u16 = *iter.next().unwrap() as u16;
            *nbytes += 1;
            let hi = *iter.next().unwrap() as u16;
            data |= hi << 8;
            *nbytes += 1;

            if data != 0 {
                ret.push_str(" + ");
                ret.push_str(&data.to_string());
            }
        }
        _ => panic!("unhandled: mod={}", mode),
    }

    ret.push(']');

    ret
}
