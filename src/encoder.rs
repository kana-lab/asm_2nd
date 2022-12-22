use crate::lexer::{Mnemonic, Register};
use crate::lexer::Mnemonic::*;
use crate::parser::Instruction;
use crate::parser::Operand::*;
use crate::semantics::{is_arithmetic, is_arithmetic_ext, is_arithmetic_imm};

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
        } else if mnemonic == Beq || mnemonic == Blt || mnemonic == Ble {
            b |= (get_register_num(cast!(operands[0], OpRegister)) as u32) << 8;
            b |= get_register_num(cast!(operands[1], OpRegister)) as u32;
            b |= ((cast!(operands[2], OpDigit) as u32) << 16) & 0x03ffffff;
        } else if mnemonic == J {
            b |= (cast!(operands[0], OpDigit) as u32) & 0x03ffffff;
        } else if mnemonic == Jr {
            b |= get_register_num(cast!(operands[0], OpRegister)) as u32;
        } else if mnemonic == Lw {
            b |= (get_register_num(cast!(operands[0], OpRegister)) as u32) << 16;
            b |= get_register_num(cast!(operands[1], OpRegister)) as u32;
        } else if mnemonic == Sw {
            b |= (get_register_num(cast!(operands[0], OpRegister)) as u32) << 8;
            b |= get_register_num(cast!(operands[1], OpRegister)) as u32;
        } else if mnemonic == Put {
            b = cast!(operands[0], OpDigit) as u32;
        } else {
            unreachable!();
        }

        let op_funct = get_op_funct(mnemonic);
        b |= op_funct;
        binary.push(b);
    }

    binary
}
