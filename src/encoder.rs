use crate::lexer::{Mnemonic, Register};
use crate::lexer::Mnemonic::*;
use crate::parser::Instruction;
use crate::parser::Operand::*;
use crate::semantics::{is_arithmetic, is_arithmetic_ext, is_arithmetic_imm, is_conditional_branch, is_conditional_branch_ext};

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
        Add => 0x08000000,
        Sub => 0x09000000,
        Addi => 0x28000000,
        Subi => 0x29000000,
        Slli => 0x2a000000,
        Fabs => 0x18000000,
        Fneg => 0x19000000,
        Fadd => 0x41000000,
        Fsub => 0x42000000,
        Fmul => 0x44000000,
        Fdiv => 0x48000000,
        Ftoi => 0x52000000,
        Itof => 0x54000000,
        Fsqrt => 0x58000000,
        Ibeq => 0x80000000,
        Ibne => 0x88000000,
        Iblt => 0x90000000,
        Ible => 0x98000000,
        Fblt => 0xa0000000,
        Fble => 0xa8000000,
        Fbps => 0xb0000000,
        Fbng => 0xb8000000,
        J => 0xf8000000,
        Jr => 0xd8000000,
        Call => 0xf9000000,
        Movl => 0x1c000000,
        Movh => 0x1d000000,
        Urecv => 0x34000000,
        Usend => 0xd4000000,
        Lw => 0x10000000,
        Sw => 0xc0000000,
        // Lwn => 0x11000000,
        // Swn => 0xc1000000,
        _ => unreachable!()
    }
}

macro_rules! cast {
    ($target: expr, $pat: path) => {
        { if let $pat(a) = $target { a } else { unreachable!() } }
    }
}

/// semantic check, put以外の疑似命令の変換, アドレス解決が終わった命令列が渡される事を想定している
/// したがって、命令列に疑似命令やラベルが含まれてはいけない
pub fn encode(instructions: Vec<Instruction>) -> Vec<u32> {
    let mut binary = vec![];

    let it = instructions.into_iter();
    for Instruction { mnemonic, operands, .. } in it {
        let mut b = 0_u32;

        if is_arithmetic(mnemonic) {
            b |= (get_register_num(cast!(operands[0], OpRegister)) as u32) << 16;
            b |= (get_register_num(cast!(operands[1], OpRegister)) as u32) << 8;

            if is_arithmetic_imm(mnemonic) {
                b |= cast!(operands[2], OpDigit) as u32;
            } else {
                b |= get_register_num(cast!(operands[2], OpRegister)) as u32;
            }
        } else if is_arithmetic_ext(mnemonic) {
            b |= (get_register_num(cast!(operands[0], OpRegister)) as u32) << 16;
            b |= get_register_num(cast!(operands[1], OpRegister)) as u32;
        } else if is_conditional_branch(mnemonic) {
            b |= (get_register_num(cast!(operands[0], OpRegister)) as u32) << 8;
            b |= get_register_num(cast!(operands[1], OpRegister)) as u32;
            b |= ((cast!(operands[2], OpDigit) as u32) << 16) & 0x07ffffff;
        } else if is_conditional_branch_ext(mnemonic) {
            b |= get_register_num(cast!(operands[0], OpRegister)) as u32;
            b |= ((cast!(operands[1], OpDigit) as u32) << 16) & 0x07ffffff;
        } else if mnemonic == J || mnemonic == Call {
            b |= ((cast!(operands[0], OpDigit) as u32) << 8) & 0x00ffff00;
        } else if mnemonic == Jr || mnemonic == Usend {
            b |= get_register_num(cast!(operands[0], OpRegister)) as u32;
        } else if mnemonic == Movl || mnemonic == Movh {
            b |= get_register_num(cast!(operands[0], OpRegister)) as u32;
            b |= ((cast!(operands[1], OpDigit) as u32) << 8) & 0x00ffff00;
        } else if mnemonic == Urecv {
            b |= (get_register_num(cast!(operands[0], OpRegister)) as u32) << 16;
        } else if mnemonic == Lw {
            b |= (get_register_num(cast!(operands[0], OpRegister)) as u32) << 16;
            b |= get_register_num(cast!(operands[1], OpRegister)) as u32;
            b |= ((cast!(operands[2], OpDigit) as u32) << 8) & 0x0000ff00;  // 8bitを超えない保証はあるが怖い
        } else if mnemonic == Sw {
            b |= (get_register_num(cast!(operands[0], OpRegister)) as u32) << 8;
            b |= get_register_num(cast!(operands[1], OpRegister)) as u32;
            b |= ((cast!(operands[2], OpDigit) as u32) << 16) & 0x00ff0000;  // 8bitを超えない保証はあるが怖い
            // } else if mnemonic == Put {
            //     b = cast!(operands[0], OpDigit) as u32;
        } else {
            unreachable!();
        }

        let op_funct = get_op_funct(mnemonic);
        b |= op_funct;
        binary.push(b);
    }

    binary
}
