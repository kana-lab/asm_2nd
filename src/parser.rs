use std::collections::HashSet;
use std::io::Read;
use std::iter::Peekable;
use std::str::from_utf8;
use crate::lexer::{Lexer, LexToken, Mnemonic, Register};

#[derive(Debug, Clone)]
pub enum Operand {
    OpRegister(Register),
    OpLabel(String),
    OpDigit(i64),
}

#[derive(Debug)]
pub struct Instruction {
    pub label: Option<String>,
    pub mnemonic: Mnemonic,
    pub operands: Vec<Operand>,
    pub line: usize,
    pub ch: usize,
}

#[derive(Debug)]
pub enum ParseError {
    MalformedSentenceError,
    LexicalError,
}

pub struct Parser<T: Read> {
    lexer: Peekable<Lexer<T>>,
    instructions: Vec<Instruction>,
    labels: HashSet<String>,
    line: usize,
    character: usize,
}

impl<T: Read> Parser<T> {
    pub fn new(lexer: Lexer<T>) -> Self {
        Self {
            lexer: lexer.peekable(),
            instructions: vec![],
            labels: HashSet::new(),
            line: 1,
            character: 1,
        }
    }

    pub fn parse(mut self) -> Result<(Vec<Instruction>, HashSet<String>), ParseError> {
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
            self.lexer.next();
            return self.asm_program_line();
        }

        self.print_err("expected ';', newline or EOF.");
        Err(ParseError::MalformedSentenceError)
    }

    fn labeled_single_instr(&mut self) -> Result<(), ParseError> {
        let mut label_str = None;

        let a = self.peek()?;
        if let LexToken::LexLabel(label) = a {
            let s = from_utf8(&label).unwrap().to_string();
            self.labels.insert(s.clone());
            label_str = Some(s);
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

        self.single_instr(label_str)
    }

    fn single_instr(&mut self, label: Option<String>) -> Result<(), ParseError> {
        let a = self.peek()?;
        let (line, ch) = (self.line, self.character);
        if let LexToken::LexMnemonic(mnemonic) = a {
            self.lexer.next();

            let mut operands = vec![];
            operands.push(self.operand()?);

            self.operand_list(&mut operands)?;

            self.instructions.push(Instruction { label, mnemonic, operands, line, ch });
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
