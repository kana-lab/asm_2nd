use std::collections::HashSet;
use crate::lexer::{Mnemonic, Register};
use crate::lexer::Mnemonic::*;
use crate::parser::{Instruction, Operand};
use crate::semantics::operand_kind::*;

#[derive(Debug)]
pub enum SemanticError {
    ImmTooLargeError,
    InvalidOperandNumError,
    SubstitutionToZeroError,
    InvalidOperandKindError,
    LabelNotFoundError,
    LabelTooFarError,
}

mod operand_kind {
    pub const REGISTER: u8 = 1;
    pub const LABEL: u8 = 2;
    pub const DIGIT: u8 = 4;
}

fn confirm(
    operands: &Vec<Operand>, kinds: &Vec<u8>, labels: &HashSet<String>,
    line: usize, ch: usize,
) -> Result<(), SemanticError> {
    if operands.len() != kinds.len() {
        println!("at line {line}, character {ch}: Syntax Error");
        println!("the number of operands must be {}.", kinds.len());
        return Err(SemanticError::InvalidOperandNumError);
    }

    fn print_err(operand_pos: usize, kind: u8, line: usize, ch: usize) {
        const POS_TABLE: [&str; 4] = ["first", "second", "third", "fourth"];
        const KIND_TABLE: [&str; 8] = [
            "", "a register", "a label", "a register or a label",
            "an immediate value", "a register or an immediate value",
            "a label or an immediate value", ""
        ];

        println!("at line {line}, character {ch}: Syntax Error");
        println!("the {} operand must be {}.", POS_TABLE[operand_pos], KIND_TABLE[kind as usize]);
    }

    let it = operands.iter().zip(kinds).enumerate();
    for (i, (operand, kind)) in it {
        if let Operand::OpRegister(_) = operand {
            if kind & REGISTER != 0 { continue; }
        } else if let Operand::OpLabel(label) = operand {
            if kind & LABEL != 0 {
                if labels.contains(label) { continue; }

                println!("at line {line}, character {ch}: Syntax Error");
                println!("label \"{}\" not found.", label.clone());
                return Err(SemanticError::LabelNotFoundError);
            }
        } else if let Operand::OpDigit(_) = operand {
            if kind & DIGIT != 0 { continue; }
        }

        print_err(i, *kind, line, ch);
        return Err(SemanticError::InvalidOperandKindError);
    }

    Ok(())
}

pub fn is_arithmetic(m: Mnemonic) -> bool {
    match m {
        Add | Sub | Addi | Subi | Slli |
        Fadd | Fsub | Fmul | Fdiv => true,
        _ => false
    }
}

pub fn is_arithmetic_ext(m: Mnemonic) -> bool {
    match m {
        Fabs | Fneg | Fsqrt | Itof | Ftoi => true,
        _ => false
    }
}

pub fn is_arithmetic_imm(m: Mnemonic) -> bool {
    match m {
        Addi | Subi | Slli => true,
        _ => false
    }
}

pub fn is_conditional_branch(m: Mnemonic) -> bool {
    match m {
        Ibeq | Ibne | Ible | Iblt | Fblt | Fble |
        Libeq | Libne | Lible | Liblt | Lfblt | Lfble => true,
        _ => false
    }
}

pub fn is_conditional_branch_ext(m: Mnemonic) -> bool {
    match m {
        Fbps | Fbng | Lfbps | Lfbng => true,
        _ => false
    }
}

pub fn check_semantics(
    instructions: &Vec<Instruction>, labels: &HashSet<String>,
) -> Result<(), SemanticError> {
    for Instruction { mnemonic, operands, line, ch, .. } in instructions {
        let (mnemonic, line, ch) = (*mnemonic, *line, *ch);

        if is_arithmetic(mnemonic) {
            if is_arithmetic_imm(mnemonic) {
                confirm(operands, &vec![REGISTER, REGISTER, DIGIT], labels, line, ch)?;

                if let Operand::OpDigit(n) = operands[2] {
                    if !(0 <= n && n < 256) {
                        println!("at line {line}, character {ch}: Error");
                        println!("the number exceeds the size of 8bit integer.");
                        return Err(SemanticError::ImmTooLargeError);
                    }
                }
            } else {
                confirm(operands, &vec![REGISTER, REGISTER, REGISTER], labels, line, ch)?;
            }

            if let Operand::OpRegister(r) = operands[0] {
                if let Register::Zero = r {
                    println!("at line {line}, character {ch}: Error");
                    println!("substitution to zero register is meaningless.");
                    return Err(SemanticError::SubstitutionToZeroError);
                }
            }
        } else if is_arithmetic_ext(mnemonic) {
            confirm(operands, &vec![REGISTER, REGISTER], labels, line, ch)?;

            if let Operand::OpRegister(r) = operands[0] {
                if let Register::Zero = r {
                    println!("at line {line}, character {ch}: Error");
                    println!("substitution to zero register is meaningless.");
                    return Err(SemanticError::SubstitutionToZeroError);
                }
            }
        } else if is_conditional_branch(mnemonic) {
            confirm(operands, &vec![REGISTER, REGISTER, LABEL], labels, line, ch)?;
        } else if is_conditional_branch_ext(mnemonic) {
            confirm(operands, &vec![REGISTER, LABEL], labels, line, ch)?;
        } else if mnemonic == J || mnemonic == Call {
            confirm(operands, &vec![LABEL], labels, line, ch)?;
            // } else if mnemonic == Ret {
            //     confirm(operands, &vec![DIGIT], labels, line, ch)?;
            //
            //     // immの範囲
            //     if let Operand::OpDigit(n) = operands[0] {
            //         if !(-32768 <= n && n < 32768) {
            //             println!("at line {line}, character {ch}: Error");
            //             println!("the number exceeds the size of 16bit integer.");
            //             return Err(SemanticError::ImmTooLargeError);
            //         }
            //     }
        } else if mnemonic == Jr || mnemonic == Usend {
            confirm(operands, &vec![REGISTER], labels, line, ch)?;
        } else if mnemonic == Urecv {
            confirm(operands, &vec![REGISTER], labels, line, ch)?;

            // zeroレジスタ
            if let Operand::OpRegister(r) = operands[0] {
                if let Register::Zero = r {
                    println!("at line {line}, character {ch}: Error");
                    println!("substitution to zero register is meaningless.");
                    return Err(SemanticError::SubstitutionToZeroError);
                }
            }
        } else if mnemonic == Lw {
            confirm(operands, &vec![REGISTER, REGISTER, DIGIT], labels, line, ch)?;

            // immの範囲
            if let Operand::OpDigit(n) = operands[2] {
                if !(0 <= n && n < 256) {
                    println!("at line {line}, character {ch}: Error");
                    println!("the number exceeds the size of 16bit integer.");
                    return Err(SemanticError::ImmTooLargeError);
                }
            }

            // zeroレジスタ
            if let Operand::OpRegister(r) = operands[0] {
                if let Register::Zero = r {
                    println!("at line {line}, character {ch}: Error");
                    println!("substitution to zero register is meaningless.");
                    return Err(SemanticError::SubstitutionToZeroError);
                }
            }
        } else if mnemonic == Sw {
            confirm(operands, &vec![REGISTER, REGISTER, DIGIT], labels, line, ch)?;

            // immの範囲
            if let Operand::OpDigit(n) = operands[2] {
                if !(0 <= n && n < 256) {
                    println!("at line {line}, character {ch}: Error");
                    println!("the number exceeds the size of 16bit integer.");
                    return Err(SemanticError::ImmTooLargeError);
                }
            }
        } else if mnemonic == Movl || mnemonic == Movh {
            confirm(operands, &vec![REGISTER, DIGIT | LABEL], labels, line, ch)?;

            // immの範囲
            if let Operand::OpDigit(n) = operands[1] {
                if !(0 <= n && n < 65536) {
                    println!("at line {line}, character {ch}: Error");
                    println!("the number exceeds the size of 16bit integer.");
                    return Err(SemanticError::ImmTooLargeError);
                }
            }
        }
    }

    Ok(())
}