use clap::Parser;
use std::io::Read;

#[derive(clap::Parser)]
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
                "BH"
            }
        }
        _ => {
            panic!("Unsupported value for REG or R/M");
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
fn mov_reg_mem_to_from_reg(index: &mut usize, bytes: &[u8]) {
    if bytes.len() < *index + 1 {
        panic!("Invalid instruction length, expected two bytes!");
    }

    let byte_one: u8 = bytes[*index];
    let byte_two: u8 = bytes[*index + 1];

    let register: u8 = (byte_two & REG) >> 3;
    let is_word: bool = (byte_one & W) == W;
    let register_in_reg: &str = get_register(register, is_word);

    let destination_in_reg = (byte_one & D) == D;

    let register_memory: u8 = byte_two & R_M;

    let mode: u8 = (byte_two & MOD) >> 6;
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
                0b110 => panic!("TODO(jmarcil): DIRECT ADDRESS"),
                0b111 => "[BX]",
                _ => panic!("Unsupported REG {}!", register_memory),
            };

            if destination_in_reg {
                println!("MOV {}, {}", register_in_reg, effective_address);
            } else {
                println!("MOV {}, {}", effective_address, register_in_reg);
            }

            *index += 2;
        }
        // 8-bit displacement
        0b01 => {
            if bytes.len() < *index + 2 {
                panic!("Invalid instruction length, expected three bytes!");
            }

            // TODO(jmarcil): Properly display negative displacement.
            let displacement: i8 = bytes[*index + 2] as i8;

            let registers: &str = match register_memory {
                0b000 => "BX + SI",
                0b001 => "BX + DI",
                0b010 => "BP + SI",
                0b011 => "BP + DI",
                0b100 => "SI",
                0b101 => "DI",
                0b110 => "BP",
                0b111 => "BX",
                _ => panic!("Unsupported REG {:b}!", register_memory),
            };

            if destination_in_reg {
                if displacement != 0 {
                    println!(
                        "MOV {}, [{} + {}]",
                        register_in_reg, registers, displacement
                    );
                } else {
                    println!("MOV {}, [{}]", register_in_reg, registers);
                }
            } else if displacement != 0 {
                println!(
                    "MOV [{} + {}], {}",
                    registers, displacement, register_in_reg
                );
            } else {
                println!("MOV [{}], {}", registers, register_in_reg);
            }

            *index += 3;
        }
        // 16-bit displacement
        0b10 => {
            if bytes.len() < *index + 3 {
                panic!("Unexpected instruction length, expected four bytes!");
            }

            let byte_three: u8 = bytes[*index + 2];
            let byte_four: u8 = bytes[*index + 3];

            // TODO(jmarcil): Properly display negative displacement.
            let displacement: i16 = (byte_four as i16) << 8 | (byte_three as i16);

            let registers: &str = match register_memory {
                0b000 => "BX + SI",
                0b001 => "BX + DI",
                0b010 => "BP + SI",
                0b011 => "BP + DI",
                0b100 => "SI",
                0b101 => "DI",
                0b110 => "BP",
                0b111 => "BX",
                _ => panic!("Unsupported REG {:b}!", register_memory),
            };

            if destination_in_reg {
                if displacement != 0 {
                    println!(
                        "MOV {}, [{} + {}]",
                        register_in_reg, registers, displacement
                    );
                } else {
                    println!("MOV {}, [{}]", register_in_reg, registers);
                }
            } else if displacement != 0 {
                println!(
                    "MOV [{} + {}], {}",
                    registers, displacement, register_in_reg
                );
            } else {
                println!("MOV [{}], {}", registers, register_in_reg);
            }

            *index += 4;
        }
        // Register
        0b11 => {
            let register_in_r_m = get_register(register_memory, is_word);

            if destination_in_reg {
                println!("MOV {}, {}", register_in_reg, register_in_r_m);
            } else {
                println!("MOV {}, {}", register_in_r_m, register_in_reg);
            }

            *index += 2;
        }
        _ => {
            panic!("Unsupported MOD {:b}!", mode);
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
fn mov_imm_to_reg(index: &mut usize, bytes: &[u8]) {
    if bytes.len() < *index + 1 {
        panic!("Unexpected instruction length, expected two bytes!");
    }

    let byte_one: u8 = bytes[*index];
    let byte_two: u8 = bytes[*index + 1];

    let reg: u8 = byte_one & 0b111;
    let is_word: bool = (byte_one & 0b1000) == 0b1000;
    let register = get_register(reg, is_word);

    if is_word {
        if bytes.len() < *index + 2 {
            panic!("Unexpected instruction length, expected three bytes!");
        }

        let byte_three: u8 = bytes[*index + 2];
        let immediate: i16 = ((byte_three as i16) << 8) | (byte_two as i16);
        println!("MOV {}, {}", register, immediate);

        *index += 3;
    } else {
        println!("MOV {}, {}", register, byte_two);

        *index += 2;
    }
}

// TODO(jmarcil): Replace panic! with more idiomatic error handling.
fn main() {
    let args = Args::parse();

    if let Ok(file) = std::fs::File::open(&args.input) {
        println!("; {}", args.input);
        println!("bits 16");

        let mut reader = std::io::BufReader::new(file);
        let mut bytes = vec![];
        if reader.read_to_end(&mut bytes).is_ok() {
            let mut index: usize = 0;
            while bytes.len() > index {
                let byte_one: u8 = bytes[index];
                let opcode: u8 = (byte_one & OPCODE) >> 2;

                match opcode {
                    //--------------------------------
                    //  MOV - Reg/Mem to/from Reg
                    //--------------------------------
                    0b100010 => mov_reg_mem_to_from_reg(&mut index, &bytes),
                    //--------------------------------
                    //  MOV - Imm to Reg
                    //--------------------------------
                    0b101100 | 0b101101 | 0b101110 | 0b101111 => mov_imm_to_reg(&mut index, &bytes),
                    _ => {
                        panic!("Unsupported OPCODE {:b}!", opcode);
                    }
                }
            }
        }
    }
}
