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

const MOD_MM_NO_DISP: u8 = 0b00;
const MOD_MM_8_BIT_DISP: u8 = 0b01;
const MOD_MM_16_BIT_DISP: u8 = 0b10;
const MOD_RM_NO_DISP: u8 = 0b11;

//------------------------------
//    REG    | W == 0 | W == 1 |
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
fn get_reg(reg: u8, is_word: bool) -> String {
    if is_word {
        match reg {
            0b000 => String::from("AX"),
            0b001 => String::from("CX"),
            0b010 => String::from("DX"),
            0b011 => String::from("BX"),
            0b100 => String::from("SP"),
            0b101 => String::from("BP"),
            0b110 => String::from("SI"),
            0b111 => String::from("DI"),
            _ => unreachable!(),
        }
    } else {
        match reg {
            0b000 => String::from("AL"),
            0b001 => String::from("CL"),
            0b010 => String::from("DL"),
            0b011 => String::from("BL"),
            0b100 => String::from("AH"),
            0b101 => String::from("CH"),
            0b110 => String::from("DH"),
            0b111 => String::from("BH"),
            _ => unreachable!(),
        }
    }
}

fn get_effective_address(r_m: u8, bytes: &mut Enumerate<Bytes<BufReader<File>>>) -> String {
    match r_m {
        0b000 => String::from("[BX + SI]"),
        0b001 => String::from("[BX + DI]"),
        0b010 => String::from("[BP + SI]"),
        0b011 => String::from("[BP + DI]"),
        0b100 => String::from("[SI]"),
        0b101 => String::from("[DI]"),
        0b110 => {
            let disp_lo = bytes.next().unwrap().1.unwrap() as i16;
            let disp_hi = bytes.next().unwrap().1.unwrap() as i16;
            let disp = (disp_hi << 8) | disp_lo;
            format!("[{disp}]")
        }
        0b111 => String::from("[BX]"),
        _ => unreachable!(),
    }
}

// TODO(jmarcil): Doc comment.
fn get_disp_registers(register_memory: u8) -> String {
    match register_memory {
        0b000 => String::from("BX + SI"),
        0b001 => String::from("BX + DI"),
        0b010 => String::from("BP + SI"),
        0b011 => String::from("BP + DI"),
        0b100 => String::from("SI"),
        0b101 => String::from("DI"),
        0b110 => String::from("BP"),
        0b111 => String::from("BX"),
        _ => unreachable!(),
    }
}

fn get_disp_byte(register: &str, displacement: i8) -> String {
    match 0.cmp(&displacement) {
        Ordering::Equal => {
            format!("[{}]", register)
        }
        Ordering::Less => {
            format!("[{} + {}]", register, displacement)
        }
        Ordering::Greater => {
            format!("[{} - {}]", register, -displacement)
        }
    }
}

fn get_disp_word(register: &str, displacement: i16) -> String {
    match 0.cmp(&displacement) {
        Ordering::Equal => {
            format!("[{}]", register)
        }
        Ordering::Less => {
            format!("[{} + {}]", register, displacement)
        }
        Ordering::Greater => {
            format!("[{} - {}]", register, -displacement)
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
    let register_in_reg = get_reg(register, is_word);

    match mode {
        MOD_MM_NO_DISP => {
            let effective_address = get_effective_address(register_memory, bytes);

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
        MOD_MM_8_BIT_DISP => {
            let disp: i8 = bytes.next().unwrap().1.unwrap() as i8;

            let disp_registers = get_disp_registers(register_memory);

            let src;
            let dst;
            if destination_in_reg {
                src = get_disp_byte(&disp_registers, disp);
                dst = register_in_reg;
            } else {
                src = register_in_reg;
                dst = get_disp_byte(&disp_registers, disp);
            };

            println!("MOV {dst}, {src}");
        }
        MOD_MM_16_BIT_DISP => {
            let disp_lo: u8 = bytes.next().unwrap().1.unwrap();
            let disp_hi: u8 = bytes.next().unwrap().1.unwrap();
            let disp: i16 = (disp_hi as i16) << 8 | (disp_lo as i16);

            let registers = get_disp_registers(register_memory);

            let src;
            let dst;
            if destination_in_reg {
                src = get_disp_word(&registers, disp);
                dst = register_in_reg;
            } else {
                src = register_in_reg;
                dst = get_disp_word(&registers, disp);
            };

            println!("MOV {dst}, {src}");
        }
        MOD_RM_NO_DISP => {
            let register_in_r_m = get_reg(register_memory, is_word);

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

    let src = get_reg(reg, is_word);

    let dst = if is_word {
        let lo = bytes.next().unwrap().1.unwrap() as i16;
        let hi = bytes.next().unwrap().1.unwrap() as i16;
        (hi << 8) | lo
    } else {
        bytes.next().unwrap().1.unwrap() as i16
    };

    println!("MOV {dst}, {src}");
}

// TODO(jmarcil): Doc comment.
fn mov_imm_to_r_m(byte_one: u8, bytes: &mut Enumerate<Bytes<BufReader<File>>>) {
    let is_word = (byte_one & W) == W;

    let byte_two = bytes.next().unwrap().1.unwrap();
    let mode = (byte_two & MOD) >> 6;
    let r_m = byte_two & R_M;

    match mode {
        MOD_MM_NO_DISP => {
            if is_word {
                let lo = bytes.next().unwrap().1.unwrap();
                let hi = bytes.next().unwrap().1.unwrap();
                let src: i16 = (hi as i16) << 8 | lo as i16;

                let dst = get_effective_address(r_m, bytes);

                println!("MOV {dst}, WORD {src}");
            } else {
                let src = bytes.next().unwrap().1.unwrap();
                let dst = get_effective_address(r_m, bytes);

                println!("MOV {dst}, BYTE {src}");
            }
        }
        MOD_MM_8_BIT_DISP => {
            let disp = bytes.next().unwrap().1.unwrap() as i8;
            let disp_registers = get_disp_registers(r_m);
            let dst = get_disp_byte(&disp_registers, disp);

            if is_word {
                let data_lo = bytes.next().unwrap().1.unwrap();
                let data_hi = bytes.next().unwrap().1.unwrap();
                let src = (data_hi as i16) << 8 | data_lo as i16;

                println!("MOV {dst}, WORD {src}");
            } else {
                let src = bytes.next().unwrap().1.unwrap();

                println!("MOV {dst}, BYTE {src}");
            }
        }
        MOD_MM_16_BIT_DISP => {
            let disp_lo = bytes.next().unwrap().1.unwrap() as i8;
            let disp_hi = bytes.next().unwrap().1.unwrap() as i8;
            let disp = (disp_hi as i16) << 8 | disp_lo as i16;
            let disp_registers = get_disp_registers(r_m);
            let dst = get_disp_word(&disp_registers, disp);

            if is_word {
                let data_lo = bytes.next().unwrap().1.unwrap();
                let data_hi = bytes.next().unwrap().1.unwrap();
                let src = (data_hi as i16) << 8 | data_lo as i16;

                println!("MOV {dst}, WORD {src}");
            } else {
                let src = bytes.next().unwrap().1.unwrap();

                println!("MOV {dst}, BYTE {src}");
            }
        }
        MOD_RM_NO_DISP => todo!("MOD_RM_NO_DISP"),
        _ => unreachable!(),
    }
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
                //--------------------------------
                //  MOV - Imm to Reg/Mem
                //--------------------------------
                0b110001 => mov_imm_to_r_m(byte_one, &mut bytes),
                _ => {
                    panic!("Unsupported OPCODE {:b}!", opcode);
                }
            }
        }
    }
}
