use std::collections::HashMap;
use crate::lexer::Mnemonic::*;
use crate::lexer::Mnemonic;
use crate::parser::{Instruction, Operand};
use crate::parser::Operand::OpDigit;
use crate::semantics::{is_conditional_branch, is_conditional_branch_ext};

pub enum ResolutionError {
    ImmTooLargeError,
    LabelTooFarError,
}


pub fn is_pseudo_instr(m: Mnemonic) -> bool {
    match m {
        Libeq | Libne | Liblt | Lible |
        Lfblt | Lfble | Lfbps | Lfbng => true,
        _ => false
    }
}

pub fn neg_pseudo_branch_instr(m: Mnemonic) -> Mnemonic {
    match m {
        Libeq => Ibne,
        Libne => Ibeq,
        Lible => Iblt,
        Liblt => Ible,
        Lfblt => Fble,
        Lfble => Fblt,
        Lfbps => Fbng,
        Lfbng => Fbps,
        _ => unreachable!()
    }
}

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

        for s in label {
            addr_map.insert(s.clone(), address as i64 + addr_padding);
        }

        if mnemonic == Libeq || mnemonic == Libne || mnemonic == Liblt || mnemonic == Lible ||
            mnemonic == Lfblt || mnemonic == Lfble || mnemonic == Lfbps || mnemonic == Lfbng {
            addr_padding += 1;
        }
    }

    let mut instr = vec![];
    for Instruction { mnemonic, mut operands, line, ch, .. } in instructions {
        // if is_arithmetic_imm(mnemonic) {
        //     if let Operand::OpLabel(s) = &operands[2] {
        //         let dest_addr = *addr_map.get(s).unwrap();
        //         if dest_addr >= 256 {
        //             println!("at line {line}, character {ch}: Warning");
        //             println!("the number exceeds the size of 8bit integer.");
        //             return Err(ResolutionError::ImmTooLargeError);
        //         }
        //
        //         operands[2] = Operand::OpDigit(dest_addr);
        //     }
        // } else if mnemonic == Beq || mnemonic == Blt || mnemonic == Ble {
        if is_conditional_branch(mnemonic) && !is_pseudo_instr(mnemonic) {
            if let Operand::OpLabel(label) = &operands[2] {
                let dest_addr = *addr_map.get(label).unwrap();
                let relative_addr = dest_addr - instr.len() as i64;
                if !(-1024 <= relative_addr && relative_addr < 1024) {
                    println!("at line {line}, character {ch}: Error");
                    println!("label \"{}\" is too far to jump.", label.clone());
                    return Err(ResolutionError::LabelTooFarError);
                }

                operands[2] = Operand::OpDigit(relative_addr);
            }
        } else if is_conditional_branch_ext(mnemonic) && !is_pseudo_instr(mnemonic) {
            if let Operand::OpLabel(label) = &operands[1] {
                let dest_addr = *addr_map.get(label).unwrap();
                let relative_addr = dest_addr - instr.len() as i64;
                if !(-1024 <= relative_addr && relative_addr < 1024) {
                    println!("at line {line}, character {ch}: Error");
                    println!("label \"{}\" is too far to jump.", label.clone());
                    return Err(ResolutionError::LabelTooFarError);
                }

                operands[1] = Operand::OpDigit(relative_addr);
            }
        } else if mnemonic == J || mnemonic == Call {
            if let Operand::OpLabel(label) = &operands[0] {
                let dest_addr = *addr_map.get(label).unwrap();
                let relative_addr = dest_addr - instr.len() as i64;
                let lim = 1 << 15;
                if !(-lim <= relative_addr && relative_addr < lim) {
                    println!("at line {line}, character {ch}: Error");
                    println!("label \"{}\" is too far to jump (over 32,768 lines).", label.clone());
                    return Err(ResolutionError::LabelTooFarError);
                }

                operands[0] = Operand::OpDigit(relative_addr);
            }
            // } else if mnemonic == Put {
            //     if let Operand::OpLabel(label) = &operands[0] {
            //         let dest_addr = *addr_map.get(label).unwrap();
            //         operands[0] = Operand::OpDigit(dest_addr);
            //     }
        } else if is_conditional_branch(mnemonic) && is_pseudo_instr(mnemonic) {
            if let Operand::OpLabel(label) = &operands[2] {
                let dest_addr = *addr_map.get(label).unwrap();
                let relative_addr = dest_addr - instr.len() as i64;

                let mnemonic = neg_pseudo_branch_instr(mnemonic);
                (operands[0], operands[1]) = (operands[1].clone(), operands[0].clone());
                operands[2] = OpDigit(2);
                instr.push(Instruction { label: vec![], mnemonic, operands, line, ch });
                instr.push(Instruction { label: vec![], mnemonic: J, operands: vec![OpDigit(relative_addr - 1)], line, ch });
            }
            continue;
        } else if is_conditional_branch_ext(mnemonic) && is_pseudo_instr(mnemonic) {
            if let Operand::OpLabel(label) = &operands[1] {
                let dest_addr = *addr_map.get(label).unwrap();
                let relative_addr = dest_addr - instr.len() as i64;

                let mnemonic = neg_pseudo_branch_instr(mnemonic);
                operands[1] = OpDigit(2);
                instr.push(Instruction { label: vec![], mnemonic, operands, line, ch });
                instr.push(Instruction { label: vec![], mnemonic: J, operands: vec![OpDigit(relative_addr - 1)], line, ch });
            }
            continue;
        } else if mnemonic == Movl || mnemonic == Movh {
            if let Operand::OpLabel(label) = &operands[1] {
                let dest_addr = *addr_map.get(label).unwrap();
                let lim = 1 << 16;
                if !(0 <= dest_addr && dest_addr < lim) {
                    println!("at line {line}, character {ch}: Error");
                    println!("label \"{}\" is too large for mov* instruction (over 32,768).", label.clone());
                    return Err(ResolutionError::LabelTooFarError);
                }

                operands[1] = Operand::OpDigit(dest_addr);
            }
        }

        instr.push(Instruction { label: vec![], mnemonic, operands, line, ch });
    }

    Ok(instr)
}