use std::collections::HashMap;
use crate::lexer::{Mnemonic, Register};
use crate::lexer::Mnemonic::*;
use crate::parser::{Instruction, Operand};

#[derive(Debug)]
pub enum EncodeError {
    ImmTooLargeError,
    InvalidOperandNumError,
    SubstitutionToZeroError,
    InvalidOperandKindError,
    LabelNotFoundError,
    LabelTooFarError,
}

fn is_arithmetic(m: Mnemonic) -> bool {
    match m {
        Add | Sub | Sll | Srl | Sra | Addi | Subi | Slli | Srli | Srai |
        Fadd | Fsub | Fmul | Fdiv | Fless => true,
        _ => false
    }
}

fn is_arithmetic_ext(m: Mnemonic) -> bool {
    match m {
        Fispos | Fisneg | Fneg | Ftoi | Itof | Fsqrt => true,
        _ => false
    }
}

fn is_arithmetic_imm(m: Mnemonic) -> bool {
    match m {
        Addi | Subi | Slli | Srli | Srai => true,
        _ => false
    }
}

fn get_register_num(r: Register) -> u8 {
    match r {
        Register::Zero => 255,
        Register::Fp => 254,
        Register::Sp => 253,
        Register::R(n) => n,
    }
}

fn get_op_funct(m: Mnemonic) -> u32 {
    match m {
        Add => 0x01000000,
        Sub => 0x02000000,
        Sll => 0x09000000,
        Srl => 0x0a000000,
        Sra => 0x0c000000,
        Fispos => 0x11000000,
        Fisneg => 0x12000000,
        Fneg => 0x14000000,
        Addi => 0x21000000,
        Subi => 0x22000000,
        Slli => 0x24000000,
        Srli => 0x28000000,
        Srai => 0x30000000,
        Fadd => 0x41000000,
        Fsub => 0x42000000,
        Fmul => 0x44000000,
        Fdiv => 0x48000000,
        Fless => 0x51000000,
        Ftoi => 0x52000000,
        Itof => 0x54000000,
        Fsqrt => 0x58000000,
        Beq => 0x84000000,
        Blt => 0x88000000,
        Ble => 0x90000000,
        J => 0xa0000000,
        Jr => 0xa4000000,
        Lw => 0xe0000000,
        Sw => 0xc0000000,
        Put => 0,
        Mov | Lbeq | Lblt | Lble => unreachable!()
    }
}

pub fn encode(
    instructions: Vec<Instruction>, address_map: HashMap<String, i64>,
) -> Result<Vec<u32>, EncodeError> {
    let mut binary = vec![];

    // 疑似命令のおかげでアドレスがズレるのでaddress_mapの中身は正確ではなくなる
    // 二分探索を用いて調整すべし
    let mut address_padding = 0;
    let mut padding_info: Vec<(usize, i64)> = vec![];

    // 頼む。。。動いてくれー
    fn _label_to_addr(
        address_map: &HashMap<String, i64>, padding_info: &Vec<(usize, i64)>, label: &str,
    ) -> Option<i64> {
        let orig_addr = *address_map.get(label)?;
        let r = padding_info.binary_search_by_key(&(orig_addr as usize), |&(a, b)| a);
        let pad = match r {
            Ok(ind) => padding_info[ind].1,
            Err(ind) => if ind == 0 { 0 } else { padding_info[ind - 1].1 }
        };
        Some(orig_addr + pad)
    }

    let it = instructions.into_iter().enumerate();
    for (address, Instruction(mnemonic, operands, line, ch)) in it {
        let label_to_addr = |label| {
            _label_to_addr(&address_map, &padding_info, label)
        };
        let mut b = 0_u32;

        // operand's format check
        if is_arithmetic(mnemonic) {
            if operands.len() != 3 {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the number of operands must be 3.");
                return Err(EncodeError::InvalidOperandNumError);
            }

            if let Operand::OpRegister(r) = operands[0] {
                if let Register::Zero = r {
                    println!("at line {line}, character {ch}: Warning");
                    println!("substitution to zero register is meaningless.");
                    return Err(EncodeError::SubstitutionToZeroError);
                }

                let reg_num = get_register_num(r);
                b |= (reg_num as u32) << 16;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the first operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }

            if let Operand::OpRegister(r) = operands[1] {
                let reg_num = get_register_num(r);
                b |= (reg_num as u32) << 8;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the second operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }

            if is_arithmetic_imm(mnemonic) {
                if let Operand::OpDigit(n) = operands[2] {
                    if !(-256 < n && n < 256) {
                        println!("at line {line}, character {ch}: Warning");
                        println!("the number exceeds the size of 8bit integer.");
                        return Err(EncodeError::ImmTooLargeError);
                    }

                    let n = n as u8;
                    b |= n as u32;
                } else if let Operand::OpLabel(s) = &operands[2] {
                    let dest_addr = label_to_addr(s);
                    if dest_addr.is_none() {
                        println!("at line {line}, character {ch}: Syntax Error");
                        println!("label \"{}\" not found.", s.clone());
                        return Err(EncodeError::LabelNotFoundError);
                    }
                    let dest_addr = dest_addr.unwrap();

                    let n = dest_addr as u8;
                    b |= n as u32;
                } else {
                    println!("at line {line}, character {ch}: Syntax Error");
                    println!("the third operand must be an immediate value.");
                    return Err(EncodeError::InvalidOperandKindError);
                }
            } else {
                if let Operand::OpRegister(r) = operands[2] {
                    let reg_num = get_register_num(r);
                    b |= reg_num as u32;
                } else {
                    println!("at line {line}, character {ch}: Syntax Error");
                    println!("the third operand must be a register.");
                    return Err(EncodeError::InvalidOperandKindError);
                }
            }
        } else if is_arithmetic_ext(mnemonic) {
            if operands.len() != 2 {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the number of operands must be 2.");
                return Err(EncodeError::InvalidOperandNumError);
            }

            if let Operand::OpRegister(r) = operands[0] {
                if let Register::Zero = r {
                    println!("at line {line}, character {ch}: Warning");
                    println!("substitution to zero register is meaningless.");
                    return Err(EncodeError::SubstitutionToZeroError);
                }

                let reg_num = get_register_num(r);
                b |= (reg_num as u32) << 16;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the first operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }

            if let Operand::OpRegister(r) = operands[1] {
                let reg_num = get_register_num(r);
                b |= reg_num as u32;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the second operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }
        } else if mnemonic == Beq || mnemonic == Blt || mnemonic == Ble {
            if operands.len() != 3 {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the number of operands must be 3.");
                return Err(EncodeError::InvalidOperandNumError);
            }

            if let Operand::OpRegister(r) = operands[0] {
                let reg_num = get_register_num(r);
                b |= (reg_num as u32) << 8;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the first operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }

            if let Operand::OpRegister(r) = operands[1] {
                let reg_num = get_register_num(r);
                b |= reg_num as u32;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the second operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }

            if let Operand::OpLabel(label) = &operands[2] {
                let dest_addr = label_to_addr(label);
                if dest_addr.is_none() {
                    println!("at line {line}, character {ch}: Syntax Error");
                    println!("label \"{}\" not found.", label.clone());
                    return Err(EncodeError::LabelNotFoundError);
                }
                let dest_addr = dest_addr.unwrap();

                let relative_addr = dest_addr - address as i64 - address_padding;
                if !(-512 <= relative_addr && relative_addr < 512) {
                    println!("at line {line}, character {ch}: Warning");
                    println!("label \"{}\" is too far to jump.", label.clone());
                    return Err(EncodeError::LabelTooFarError);
                }

                b |= ((relative_addr as u32) << 16) & 0x03ffffff;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the third operand must be a label.");
                return Err(EncodeError::InvalidOperandKindError);
            }
        } else if mnemonic == Mnemonic::J {
            if operands.len() != 1 {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the number of operands must be 1.");
                return Err(EncodeError::InvalidOperandNumError);
            }

            if let Operand::OpLabel(label) = &operands[0] {
                let dest_addr = label_to_addr(label);
                if dest_addr.is_none() {
                    println!("at line {line}, character {ch}: Syntax Error");
                    println!("label \"{}\" not found.", label.clone());
                    return Err(EncodeError::LabelNotFoundError);
                }
                let dest_addr = dest_addr.unwrap();

                let relative_addr = dest_addr - address as i64 - address_padding;
                let lim = 1 << 25;
                if !(-lim <= relative_addr && relative_addr < lim) {
                    println!("at line {line}, character {ch}: Warning");
                    println!("label \"{}\" is too far to jump.", label.clone());
                    return Err(EncodeError::LabelTooFarError);
                }

                b |= (relative_addr as u32) & 0x03ffffff;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the third operand must be a label.");
                return Err(EncodeError::InvalidOperandKindError);
            }
        } else if mnemonic == Mnemonic::Jr {
            if operands.len() != 1 {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the number of operands must be 1.");
                return Err(EncodeError::InvalidOperandNumError);
            }

            if let Operand::OpRegister(r) = operands[0] {
                let reg_num = get_register_num(r);
                b |= reg_num as u32;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the first operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }
        } else if mnemonic == Mnemonic::Lw {
            if operands.len() != 2 {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the number of operands must be 1.");
                return Err(EncodeError::InvalidOperandNumError);
            }

            if let Operand::OpRegister(r) = operands[0] {
                if let Register::Zero = r {
                    println!("at line {line}, character {ch}: Warning");
                    println!("substitution to zero register is meaningless.");
                    return Err(EncodeError::SubstitutionToZeroError);
                }

                let reg_num = get_register_num(r);
                b |= (reg_num as u32) << 16;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the first operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }

            if let Operand::OpRegister(r) = operands[1] {
                let reg_num = get_register_num(r);
                b |= reg_num as u32;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the second operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }
        } else if mnemonic == Mnemonic::Sw {
            if operands.len() != 2 {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the number of operands must be 1.");
                return Err(EncodeError::InvalidOperandNumError);
            }

            if let Operand::OpRegister(r) = operands[0] {
                let reg_num = get_register_num(r);
                b |= (reg_num as u32) << 8;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the first operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }

            if let Operand::OpRegister(r) = operands[1] {
                let reg_num = get_register_num(r);
                b |= reg_num as u32;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the second operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }
        } else if mnemonic == Mnemonic::Put {
            if operands.len() != 1 {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the number of operands must be 1.");
                return Err(EncodeError::InvalidOperandNumError);
            }

            if let Operand::OpDigit(n) = operands[0] {
                b = n as u32;
            } else if let Operand::OpLabel(label) = &operands[0] {
                let dest_addr = label_to_addr(label);
                if dest_addr.is_none() {
                    println!("at line {line}, character {ch}: Syntax Error");
                    println!("label \"{}\" not found.", label.clone());
                    return Err(EncodeError::LabelNotFoundError);
                }
                let dest_addr = dest_addr.unwrap();
                b = dest_addr as u32;  // FIXME: no check if dest_addr exceeds 32bit
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the first operand must be an immediate value or a label.");
                return Err(EncodeError::InvalidOperandKindError);
            }
        } else if mnemonic == Mnemonic::Mov {
            if operands.len() != 2 {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the number of operands must be 2.");
                return Err(EncodeError::InvalidOperandNumError);
            }

            let reg_num = if let Operand::OpRegister(r) = operands[0] {
                if let Register::Zero = r {
                    println!("at line {line}, character {ch}: Warning");
                    println!("substitution to zero register is meaningless.");
                    return Err(EncodeError::SubstitutionToZeroError);
                }

                get_register_num(r)
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the first operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            };

            let val = if let Operand::OpDigit(n) = operands[1] {
                n as u32
            } else if let Operand::OpLabel(label) = &operands[1] {
                let dest_addr = label_to_addr(label);
                if dest_addr.is_none() {
                    println!("at line {line}, character {ch}: Syntax Error");
                    println!("label \"{}\" not found.", label.clone());
                    return Err(EncodeError::LabelNotFoundError);
                }
                let dest_addr = dest_addr.unwrap();
                dest_addr as u32  // FIXME: no check if dest_addr exceeds 32bit
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the second operand must be an immediate value or a label.");
                return Err(EncodeError::InvalidOperandKindError);
            };

            let mut i = 1;
            while val >> (8 * i) > 0 { i += 1; }
            i -= 1;
            binary.push(get_op_funct(Addi) | 0xff00 | ((reg_num as u32) << 16) | (val >> (8 * i)));

            address_padding += 2 * i;
            padding_info.push((address + 1, address_padding));

            let b_base = get_op_funct(Addi) | ((reg_num as u32) << 16) | ((reg_num as u32) << 8);
            let shift8 = get_op_funct(Slli) | ((reg_num as u32) << 16) | ((reg_num as u32) << 8) | 8;
            while i > 0 {
                i -= 1;
                binary.push(shift8);
                binary.push(b_base | ((val >> (8 * i)) & 0xff));
            }
            continue;
        } else if mnemonic == Lbeq || mnemonic == Lblt || mnemonic == Lble {
            if operands.len() != 3 {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the number of operands must be 3.");
                return Err(EncodeError::InvalidOperandNumError);
            }

            if let Operand::OpRegister(r) = operands[0] {
                let reg_num = get_register_num(r);
                b |= (reg_num as u32) << 8;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the first operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }

            if let Operand::OpRegister(r) = operands[1] {
                let reg_num = get_register_num(r);
                b |= reg_num as u32;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the second operand must be a register.");
                return Err(EncodeError::InvalidOperandKindError);
            }

            if let Operand::OpLabel(label) = &operands[2] {
                let dest_addr = label_to_addr(label);
                if dest_addr.is_none() {
                    println!("at line {line}, character {ch}: Syntax Error");
                    println!("label \"{}\" not found.", label.clone());
                    return Err(EncodeError::LabelNotFoundError);
                }
                let dest_addr = dest_addr.unwrap();

                let relative_addr = dest_addr - address as i64 - address_padding;
                if !(-512 <= relative_addr && relative_addr < 512) {
                    if !(-0x02000000 + 2 <= relative_addr && relative_addr < 0x02000000 - 1) {
                        println!("at line {line}, character {ch}: Warning");
                        println!("label \"{}\" is too far to jump (over 30,000,000 lines).", label.clone());
                        return Err(EncodeError::LabelTooFarError);
                    }

                    if mnemonic == Lbeq {
                        // beqの場合、bne命令がないので下のようにする
                        //     beq r1, r2, 2
                        //     j 2
                        //     j label
                        //     else...
                        // パディングは2になる
                        address_padding += 2;
                        binary.push(get_op_funct(Beq) | 2 << 16 | b);
                        binary.push(get_op_funct(J) | 2);
                        binary.push(get_op_funct(J) | (((relative_addr - 2) as u32) & 0x03ffffff));
                    } else {
                        // blt, bleの場合、命令の否定をとって下のようにする
                        //     ~inst r2, r1, 2
                        //     j label
                        //     else...
                        // パディングは1になる
                        address_padding += 1;
                        b = ((b << 8) | (b >> 8)) & 0xffff;  // swap registers
                        b |= get_op_funct(if mnemonic == Lblt { Ble } else { Blt }) | 2 << 16;
                        binary.push(b);
                        binary.push(get_op_funct(J) | (((relative_addr - 1) as u32) & 0x03ffffff));
                    }

                    padding_info.push((address + 1, address_padding));
                    continue;
                }

                b |= ((relative_addr as u32) << 16) & 0x03ffffff;
                b |= get_op_funct(match mnemonic {
                    Lbeq => Beq,
                    Lblt => Blt,
                    Lble => Ble,
                    _ => unreachable!()
                });
                binary.push(b);
                continue;
            } else {
                println!("at line {line}, character {ch}: Syntax Error");
                println!("the third operand must be a label.");
                return Err(EncodeError::InvalidOperandKindError);
            }
        }

        let op_funct = get_op_funct(mnemonic);
        b |= op_funct;
        binary.push(b);
    }

    Ok(binary)
}
