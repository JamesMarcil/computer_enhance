use clap::Parser;
use std::io::Read;

#[derive(clap::Parser)]
struct Args {
    input: String,
}

//--------------------------------
// Byte #1
//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
//         OPCODE        | D | W |
//--------------------------------
const OPCODE: u8 = 0b11111100;
const D: u8 = 0b00000010;
const W: u8 = 0b00000001;

//--------------------------------
// Byte #2
//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
//  MOD  |    REG    |    R/M    |
//--------------------------------
const MOD: u8 = 0b11000000;
const REG: u8 = 0b00111000;
const R_M: u8 = 0b00000111;

//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
// 1 | 0 | 0 | 0 | 1 | 0 | D | W |
//--------------------------------
const OPCODE_MOV_REG_MEM_TO_FROM_REG: u8 = 0b100010;

//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
// 1 | 1 | 0 | 0 | 0 | 1 | 1 | W |
//--------------------------------
const OPCODE_MOV_IMM_TO_REG_MEM: u8 = 0b110001;

//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
// 1 | 0 | 1 | 1 | W |    REG    |
//--------------------------------
const OPCODE_MOV_IMM_TO_REG: u8 = 0b101100;

//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
// 1 | 0 | 1 | 0 | 0 | 0 | 0 | W |
//--------------------------------
const OPCODE_MOV_MEM_TO_ACC: u8 = 0b101000;

//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
// 1 | 0 | 1 | 0 | 0 | 0 | 1 | W |
//--------------------------------
const OPCODE_MOV_ACC_TO_MEM: u8 = 0b100011;

//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
// 1 | 0 | 0 | 0 | 1 | 1 | 1 | 0 |
//--------------------------------
const OPCODE_MOV_REG_MEM_TO_SEG: u8 = 0b100011;

//--------------------------------
// 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
//--------------------------------
// 1 | 0 | 0 | 0 | 1 | 1 | 0 | 0 |
//--------------------------------
const OPCODE_MOV_SEG_TO_REG_MEM: u8 = 0b100011;

// D
const D_TO_REGISTER: u8 = 0b0;
const D_FROM_REGISTER: u8 = 0b0;

// W
const W_BYTE: u8 = 0b0;
const W_WORD: u8 = 0b1;

// MOD
const MOD_MEMORY_NO_DISPLACEMENT: u8 = 0b00;
const MOD_MEMORY_8_BIT_DISPLACEMENT: u8 = 0b01;
const MOD_MEMORY_16_BIT_DISPLACEMENT: u8 = 0b10;
const MOD_REGISTER: u8 = 0b11;

fn get_register(reg: u8, is_word: bool) -> &'static str {
    match reg {
        0b000 => {
            if is_word {
                "AX"
            } else {
                "AL"
            }
        }
        0b001 => {
            if is_word {
                "CX"
            } else {
                "CL"
            }
        }
        0b010 => {
            if is_word {
                "DX"
            } else {
                "DL"
            }
        }
        0b011 => {
            if is_word {
                "BX"
            } else {
                "BL"
            }
        }
        0b100 => {
            if is_word {
                "SP"
            } else {
                "AH"
            }
        }
        0b101 => {
            if is_word {
                "BP"
            } else {
                "CH"
            }
        }
        0b110 => {
            if is_word {
                "SI"
            } else {
                "DH"
            }
        }
        0b111 => {
            if is_word {
                "DI"
            } else {
                "DH"
            }
        }
        _ => {
            panic!("Unsupported value for REG or R/M");
        }
    }
}

fn main() {
    let args = Args::parse();

    if let Ok(file) = std::fs::File::open(&args.input) {
        println!("; {}", args.input);
        println!("bits 16");

        let mut reader = std::io::BufReader::new(file);
        let mut bytes = vec![];
        if reader.read_to_end(&mut bytes).is_ok() {
            let mut index: usize = 0;
            while bytes.len() > index + 1 {
                let byte_one: u8 = bytes[index];
                let byte_two: u8 = bytes[index + 1];

                let opcode: u8 = (byte_one & OPCODE) >> 2;
                let direction: u8 = (byte_one & D) >> 1;
                let word: u8 = byte_one & W;

                if opcode != OPCODE_MOV_REG_MEM_TO_FROM_REG {
                    panic!("Unsupported OPCODE {}!", opcode);
                }

                let mode: u8 = (byte_two & MOD) >> 6;
                let register: u8 = (byte_two & REG) >> 3;
                let register_memory: u8 = byte_two & R_M;

                if mode != MOD_REGISTER {
                    panic!("Unsupported MOD {}!", mode);
                }

                let is_word = word == W_WORD;
                let register_in_reg = get_register(register, is_word);
                let register_in_r_m = get_register(register_memory, is_word);

                let source_in_reg = direction == D_FROM_REGISTER;
                if source_in_reg {
                    println!("MOV {}, {}", register_in_reg, register_in_r_m);
                } else {
                    println!("MOV {}, {}", register_in_r_m, register_in_reg);
                }

                index += 2;
            }
        }
    }
}
