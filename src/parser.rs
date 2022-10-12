use std::collections::HashMap;
use std::io::Read;
use std::iter::Peekable;
use std::str::from_utf8;
use crate::lexer::{Lexer, LexToken, Mnemonic, Register};

#[derive(Debug)]
pub enum Operand {
    OpRegister(Register),
    OpLabel(String),
    OpDigit(i32),
}

#[derive(Debug)]
pub struct Instruction(Mnemonic, Vec<Operand>);

#[derive(Debug)]
pub enum ParseError {
    MalformedSentenceError,
    LexicalError,
}

pub struct Parser<T: Read> {
    lexer: Peekable<Lexer<T>>,
    instructions: Vec<Instruction>,
    labels: HashMap<String, i64>,
    line: usize,
    character: usize,
}

impl<T: Read> Parser<T> {
    pub fn new(lexer: Lexer<T>) -> Self {
        Self {
            lexer: lexer.peekable(),
            instructions: vec![],
            labels: HashMap::new(),
            line: 1,
            character: 1,
        }
    }

    pub fn parse(mut self) -> Result<(Vec<Instruction>, HashMap<String, i64>), ParseError> {
        self.asm_program()?;
        let Parser { instructions, labels, .. } = self;
        Ok((instructions, labels))
    }

    fn peek(&mut self) -> Result<LexToken, ParseError> {
        let a = self.lexer.peek();
        if a.is_none() {
            Err(ParseError::LexicalError)
        } else {
            let (a, b, c) = a.unwrap();
            self.line = *b;
            self.character = *c;
            Ok(a.clone())
        }
    }

    fn print_err(&self, msg: &str) {
        println!("at line {}, character {}: Syntax Error", self.line, self.character);
        println!("{}", msg);
    }

    // 以下、再帰下降構文解析

    fn asm_program(&mut self) -> Result<(), ParseError> {
        self.asm_program_line()?;
        let a = self.peek()?;
        if a == LexToken::LexEof { return Ok(()); }
        self.asm_program()
    }

    fn asm_program_line(&mut self) -> Result<(), ParseError> {
        let a = self.peek()?;
        if a == LexToken::LexEof || a == LexToken::LexNewline {
            self.lexer.next();
            return Ok(());
        }

        self.labeled_single_instr()?;

        let a = self.peek()?;
        if a == LexToken::LexEof || a == LexToken::LexNewline {
            self.lexer.next();
            return Ok(());
        }

        if a == LexToken::LexSemicolon {
            return self.asm_program_line();
        }

        self.print_err("expected ';', newline or EOF.");
        Err(ParseError::MalformedSentenceError)
    }

    fn labeled_single_instr(&mut self) -> Result<(), ParseError> {
        let a = self.peek()?;
        if let LexToken::LexLabel(label) = a {
            self.labels.insert(
                from_utf8(&label).unwrap().to_string(),
                self.instructions.len() as i64,
            );
            self.lexer.next();

            let a = self.peek()?;
            if a != LexToken::LexColon {
                self.print_err("expected a semicolon after a label.");
                return Err(ParseError::MalformedSentenceError);
            }
            self.lexer.next();

            while self.peek()? == LexToken::LexNewline {
                self.lexer.next();
            }
        }

        self.single_instr()
    }

    fn single_instr(&mut self) -> Result<(), ParseError> {
        let a = self.peek()?;
        if let LexToken::LexMnemonic(mnemonic) = a {
            self.lexer.next();

            let mut operands = vec![];
            operands.push(self.operand()?);

            self.operand_list(&mut operands)?;

            self.instructions.push(Instruction(mnemonic, operands));
            return Ok(());
        }

        self.print_err("expected a mnemonic.");
        Err(ParseError::MalformedSentenceError)
    }

    fn operand(&mut self) -> Result<Operand, ParseError> {
        let a = self.peek()?;
        if let LexToken::LexRegister(reg) = a {
            self.lexer.next();
            Ok(Operand::OpRegister(reg))
        } else if let LexToken::LexDigit(n) = a {
            self.lexer.next();
            Ok(Operand::OpDigit(n))
        } else if let LexToken::LexLabel(label) = a {
            self.lexer.next();
            Ok(Operand::OpLabel(from_utf8(&label).unwrap().to_string()))
        } else {
            self.print_err("expected some operands.");
            Err(ParseError::MalformedSentenceError)
        }
    }

    fn operand_list(&mut self, operands: &mut Vec<Operand>) -> Result<(), ParseError> {
        let a = self.peek()?;
        if a == LexToken::LexComma {
            self.lexer.next();
            operands.push(self.operand()?);
            self.operand_list(operands)
        } else {
            Ok(())
        }
    }
}
