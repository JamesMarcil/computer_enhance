use clap::Parser;
use std::{
    cmp::Ordering,
    fs::File,
    io::{BufReader, Bytes, Read},
    iter::Enumerate,
};

#[derive(Parser)]
struct Args {
    input: String,
}

//--------------------------------
//            Byte #1            |
//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
//         OPCODE        | D | W |
//--------------------------------
const OPCODE: u8 = 0b11111100;
const D: u8 = 0b00000010;
const W: u8 = 0b00000001;

//--------------------------------
//            Byte #2            |
//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
//  MOD  |    REG    |    R/M    |
//--------------------------------
const MOD: u8 = 0b11000000;
const REG: u8 = 0b00111000;
const R_M: u8 = 0b00000111;

//------------------------------
//    MOD    | W == 0 | W == 1 |
//------------------------------
// 0 | 0 | 0 |   AL   |   AX   |
//------------------------------
// 0 | 0 | 1 |   CL   |   CX   |
//------------------------------
// 0 | 1 | 0 |   DL   |   DX   |
//------------------------------
// 0 | 1 | 1 |   BL   |   BX   |
//------------------------------
// 1 | 0 | 0 |   AH   |   SP   |
//------------------------------
// 1 | 0 | 1 |   CH   |   BP   |
//------------------------------
// 1 | 1 | 0 |   DH   |   SI   |
//------------------------------
// 1 | 1 | 1 |   BH   |   DI   |
//------------------------------
fn get_mod_register(mode: u8, is_word: bool) -> &'static str {
    if is_word {
        match mode {
            0b000 => "AX",
            0b001 => "CX",
            0b010 => "DX",
            0b011 => "BX",
            0b100 => "SP",
            0b101 => "BP",
            0b110 => "SI",
            0b111 => "DI",
            _ => unreachable!(),
        }
    } else {
        match mode {
            0b000 => "AL",
            0b001 => "CL",
            0b010 => "DL",
            0b011 => "BL",
            0b100 => "AH",
            0b101 => "CH",
            0b110 => "DH",
            0b111 => "BH",
            _ => unreachable!(),
        }
    }
}

// TODO(jmarcil): Add doc comment for mappings.
fn get_displacement_registers(register_memory: u8) -> &'static str {
    match register_memory {
        0b000 => "BX + SI",
        0b001 => "BX + DI",
        0b010 => "BP + SI",
        0b011 => "BP + DI",
        0b100 => "SI",
        0b101 => "DI",
        0b110 => "BP",
        0b111 => "BX",
        _ => unreachable!(),
    }
}

fn i8_displacement(register: &str, displacement: i8) -> String {
    match 0.cmp(&displacement) {
        Ordering::Equal => {
            format!("[{}]", register)
        }
        Ordering::Less => {
            format!("[{} + {}]", register, displacement)
        }
        Ordering::Greater => {
            format!("[{} - {}]", register, i8::abs(displacement))
        }
    }
}

fn i16_displacement(register: &str, displacement: i16) -> String {
    match 0.cmp(&displacement) {
        Ordering::Equal => {
            format!("[{}]", register)
        }
        Ordering::Less => {
            format!("[{} + {}]", register, displacement)
        }
        Ordering::Greater => {
            format!("[{} - {}]", register, i16::abs(displacement))
        }
    }
}

//----------------------------------------------------------------
//                  MOV - Reg/Mem to/from Reg                    |
//----------------------------------------------------------------
//          BYTE #1              |            BYTE #2            |
//----------------------------------------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//----------------------------------------------------------------
// 1 | 0 | 0 | 0 | 1 | 0 | D | W |  MOD  |    REG    |    R/M    |
//----------------------------------------------------------------
fn mov_reg_mem_to_from_reg(byte_one: u8, bytes: &mut Enumerate<Bytes<BufReader<File>>>) {
    let destination_in_reg = (byte_one & D) == D;
    let is_word: bool = (byte_one & W) == W;

    let byte_two: u8 = bytes.next().unwrap().1.unwrap();
    let mode: u8 = (byte_two & MOD) >> 6;
    let register: u8 = (byte_two & REG) >> 3;
    let register_memory: u8 = byte_two & R_M;
    let register_in_reg: &str = get_mod_register(register, is_word);

    match mode {
        // No displacement
        0b00 => {
            let effective_address: &str = match register_memory {
                0b000 => "[BX + SI]",
                0b001 => "[BX + DI]",
                0b010 => "[BP + SI]",
                0b011 => "[BP + DI]",
                0b100 => "[SI]",
                0b101 => "[DI]",
                0b110 => todo!("DIRECT ADDRESS"),
                0b111 => "[BX]",
                _ => unreachable!(),
            };

            let src;
            let dst;
            if destination_in_reg {
                src = effective_address;
                dst = register_in_reg;
            } else {
                src = register_in_reg;
                dst = effective_address;
            }

            println!("MOV {dst}, {src}");
        }
        // 8-bit displacement
        0b01 => {
            let displacement: i8 = bytes.next().unwrap().1.unwrap() as i8;

            let registers: &str = get_displacement_registers(register_memory);

            let src;
            let dst;
            if destination_in_reg {
                src = i8_displacement(registers, displacement);
                dst = register_in_reg.to_owned();
            } else {
                src = register_in_reg.to_owned();
                dst = i8_displacement(registers, displacement);
            };

            println!("MOV {dst}, {src}");
        }
        // 16-bit displacement
        0b10 => {
            let byte_three: u8 = bytes.next().unwrap().1.unwrap();
            let byte_four: u8 = bytes.next().unwrap().1.unwrap();
            let displacement: i16 = (byte_four as i16) << 8 | (byte_three as i16);

            let registers = get_displacement_registers(register_memory);

            let src;
            let dst;
            if destination_in_reg {
                src = i16_displacement(registers, displacement);
                dst = register_in_reg.to_owned();
            } else {
                src = register_in_reg.to_owned();
                dst = i16_displacement(registers, displacement);
            };

            println!("MOV {dst}, {src}");
        }
        // Register
        0b11 => {
            let register_in_r_m = get_mod_register(register_memory, is_word);

            let src;
            let dst;
            if destination_in_reg {
                src = register_in_r_m;
                dst = register_in_reg;
            } else {
                src = register_in_reg;
                dst = register_in_r_m;
            }

            println!("MOV {dst}, {src}");
        }
        _ => {
            unreachable!()
        }
    }
}

//------------------------------------------------------------------------------------------------
//                                       MOV - Imm to Reg                                        |
//------------------------------------------------------------------------------------------------
//          BYTE #1              |            BYTE #2            |            BYTE #3            |
//------------------------------------------------------------------------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//------------------------------------------------------------------------------------------------
// 1 | 0 | 1 | 1 | W |    REG    |             DATA              |         DATA (W == 1)         |
//------------------------------------------------------------------------------------------------
fn mov_imm_to_reg(byte_one: u8, bytes: &mut Enumerate<Bytes<BufReader<File>>>) {
    let reg: u8 = byte_one & 0b111;
    let is_word: bool = (byte_one & 0b1000) == 0b1000;

    let src = get_mod_register(reg, is_word);

    let dst = if is_word {
        let lo = bytes.next().unwrap().1.unwrap() as i16;
        let hi = bytes.next().unwrap().1.unwrap() as i16;
        (hi << 8) | lo
    } else {
        bytes.next().unwrap().1.unwrap() as i16
    };

    println!("MOV {dst}, {src}");
}

// TODO(jmarcil): Replace panic! with more idiomatic error handling.
fn main() {
    let args = Args::parse();

    if let Ok(file) = File::open(&args.input) {
        println!("; {}", args.input);
        println!("bits 16");

        let mut bytes = BufReader::new(file).bytes().enumerate();

        // TODO(jmarcil): Utilize position for labels.
        while let Some((_, Ok(byte_one))) = bytes.next() {
            let opcode: u8 = (byte_one & OPCODE) >> 2;

            // TODO(jmarcil): Add support for additional opcodes.
            // TODO(jmarcil): Replace with a jump table?
            match opcode {
                //--------------------------------
                //  MOV - Reg/Mem to/from Reg
                //--------------------------------
                0b100010 => mov_reg_mem_to_from_reg(byte_one, &mut bytes),
                //--------------------------------
                //  MOV - Imm to Reg
                //--------------------------------
                0b1011_00..=0b1011_11 => mov_imm_to_reg(byte_one, &mut bytes),
                _ => {
                    panic!("Unsupported OPCODE {:b}!", opcode);
                }
            }
        }
    }
}
