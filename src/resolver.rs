use std::collections::HashMap;
use crate::lexer::Mnemonic::*;
use crate::lexer::Register;
use crate::parser::{Instruction, Operand};
use crate::parser::Operand::OpDigit;
use crate::semantics::is_arithmetic_imm;

pub enum ResolutionError {
    ImmTooLargeError,
    LabelTooFarError,
}

// fn expand_naive(instructions: Vec<Instruction>, )

/// semantic checkが済んだ命令列に対して、最適化をせずに疑似命令を展開し、アドレス解決をする
/// 出力された命令列にはラベルは含まれない
pub fn resolve_without_optimization(
    instructions: Vec<Instruction>
) -> Result<Vec<Instruction>, ResolutionError> {
    let mut addr_map = HashMap::new();
    let mut addr_padding = 0_i64;

    let it = instructions.iter().enumerate();
    for (address, Instruction { label, mnemonic, .. }) in it {
        let mnemonic = *mnemonic;

        if let Some(s) = label {
            addr_map.insert(s.clone(), address as i64 + addr_padding);
        }

        if mnemonic == Mov {
            addr_padding += 6;
        } else if mnemonic == Lblt || mnemonic == Lble {
            addr_padding += 1;
        } else if mnemonic == Lbeq {
            addr_padding += 2;
        }
    }

    let mut instr = vec![];
    for Instruction { mnemonic, mut operands, line, ch, .. } in instructions {
        if is_arithmetic_imm(mnemonic) {
            if let Operand::OpLabel(s) = &operands[2] {
                let dest_addr = *addr_map.get(s).unwrap();
                if dest_addr >= 256 {
                    println!("at line {line}, character {ch}: Warning");
                    println!("the number exceeds the size of 8bit integer.");
                    return Err(ResolutionError::ImmTooLargeError);
                }

                operands[2] = Operand::OpDigit(dest_addr);
            }
        } else if mnemonic == Beq || mnemonic == Blt || mnemonic == Ble {
            if let Operand::OpLabel(label) = &operands[2] {
                let dest_addr = *addr_map.get(label).unwrap();
                let relative_addr = dest_addr - instr.len() as i64;
                if !(512 <= relative_addr && relative_addr < 512) {
                    println!("at line {line}, character {ch}: Warning");
                    println!("label \"{}\" is too far to jump.", label.clone());
                    return Err(ResolutionError::LabelTooFarError);
                }

                operands[2] = Operand::OpDigit(relative_addr);
            }
        } else if mnemonic == J {
            if let Operand::OpLabel(label) = &operands[0] {
                let dest_addr = *addr_map.get(label).unwrap();
                let relative_addr = dest_addr - instr.len() as i64;
                let lim = 1 << 25;
                if !(-lim <= relative_addr && relative_addr < lim) {
                    println!("at line {line}, character {ch}: Warning");
                    println!("label \"{}\" is too far to jump (over 30,000,000 lines).", label.clone());
                    return Err(ResolutionError::LabelTooFarError);
                }

                operands[0] = Operand::OpDigit(relative_addr);
            }
        } else if mnemonic == Put {
            if let Operand::OpLabel(label) = &operands[0] {
                let dest_addr = *addr_map.get(label).unwrap();
                operands[2] = Operand::OpDigit(dest_addr);
            }
        } else if mnemonic == Mov {
            let reg = operands[0].clone();
            let val = if let Operand::OpDigit(n) = operands[1] {
                n as u32
            } else if let Operand::OpLabel(label) = &operands[1] {
                *addr_map.get(label).unwrap() as u32
            } else { unreachable!(); };

            let mut new_operands = vec![
                reg.clone(), Operand::OpRegister(Register::Zero), Operand::OpDigit(val as i64 >> 24),
            ];
            let slli = vec![reg.clone(), reg.clone(), OpDigit(8)];
            for i in 0..3 {
                instr.push(Instruction { label: None, mnemonic: Addi, operands: new_operands.clone(), line, ch });
                instr.push(Instruction { label: None, mnemonic: Slli, operands: slli.clone(), line, ch });
                new_operands[1] = reg.clone();
                new_operands[2] = Operand::OpDigit((val as i64 >> (8 * (2 - i))) & 0xff);
            }
            instr.push(Instruction { label: None, mnemonic: Addi, operands: new_operands, line, ch });
            continue;
        } else if mnemonic == Lbeq || mnemonic == Lblt || mnemonic == Lble {
            if let Operand::OpLabel(label) = &operands[2] {
                let dest_addr = *addr_map.get(label).unwrap();
                let relative_addr = dest_addr - instr.len() as i64;

                if mnemonic == Lbeq {
                    operands[2] = Operand::OpDigit(2);
                    instr.push(Instruction { label: None, mnemonic: Beq, operands, line, ch });
                    instr.push(Instruction { label: None, mnemonic: J, operands: vec![OpDigit(2)], line, ch });
                    instr.push(Instruction { label: None, mnemonic: J, operands: vec![OpDigit(relative_addr - 2)], line, ch });
                } else {
                    let mnemonic = if mnemonic == Lblt { Ble } else { Blt };
                    (operands[0], operands[1]) = (operands[1].clone(), operands[0].clone());
                    operands[2] = OpDigit(2);
                    instr.push(Instruction { label: None, mnemonic, operands, line, ch });
                    instr.push(Instruction { label: None, mnemonic: J, operands: vec![OpDigit(relative_addr - 1)], line, ch });
                }
            }
            continue;
        }

        instr.push(Instruction { label: None, mnemonic, operands, line, ch });
    }

    Ok(instr)
}