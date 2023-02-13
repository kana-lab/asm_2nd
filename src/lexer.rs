use std::io::{BufReader, Bytes, Read};
use std::iter::Peekable;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LexToken {
    LexMnemonic(Mnemonic),
    LexRegister(Register),
    LexDigit(i64),
    LexLabel(Vec<u8>),
    LexColon,
    LexComma,
    LexNewline,
    LexEof,
    LexSemicolon,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Mnemonic {
    Add,
    Sub,
    Addi,
    Subi,
    Slli,
    Fabs,
    Fneg,
    Fadd,
    Fsub,
    Fmul,
    Fdiv,
    Fsqrt,
    Itof,
    Ftoi,
    Ibeq,
    Ibne,
    Ible,
    Iblt,
    Fblt,
    Fble,
    Fbps,
    Fbng,
    J,
    Jr,
    Call,
    // Ret,
    Movl,
    Movh,
    Urecv,
    Usend,
    Lw,
    Sw,
    Libeq,
    Libne,
    Liblt,
    Lible,
    Lfblt,
    Lfble,
    Lfbps,
    Lfbng,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Register {
    Zero,
    Sp,
    Fp,
    R(u8),
}

pub struct Lexer<T: Read> {
    br: Peekable<Bytes<BufReader<T>>>,
    pub line: usize,
    pub character: usize,
}

#[derive(Debug)]
pub enum SyntaxError {
    UnknownCharacterError,
    MalformedTokenError,
}

impl<T: Read> Lexer<T> {
    pub fn new(br: BufReader<T>) -> Self {
        let br = br.bytes().peekable();
        Self { br, line: 1, character: 1 }
    }

    // TODO: これで良いか要検討
    fn is_space(ch: u8) -> bool {
        ch == b' ' || ch == b'\t'
    }

    fn get_digit(&mut self) -> Result<i64, SyntaxError> {
        let a = self.br.peek();
        if a.is_none() { return Err(SyntaxError::UnknownCharacterError); }
        let a = *a.unwrap().as_ref().unwrap();

        let is_minus = a == b'-';
        if is_minus {
            self.br.next();
            self.character += 1;
            loop {
                let a = self.br.peek();
                if a.is_none() { return Err(SyntaxError::MalformedTokenError); }
                let a = *a.unwrap().as_ref().unwrap();
                if Self::is_space(a) {
                    self.br.next();
                    self.character += 1;
                } else {
                    if a.is_ascii_digit() {
                        break;
                    } else {
                        return Err(SyntaxError::MalformedTokenError);
                    }
                }
            }
        }

        let a = *self.br.peek().unwrap().as_ref().unwrap();
        if !a.is_ascii_digit() { return Err(SyntaxError::UnknownCharacterError); }
        self.br.next();
        self.character += 1;

        let mut res = (a - b'0') as i64;

        let a = self.br.peek();
        if a.is_none() { return Ok(if is_minus { -res } else { res }); }
        let a = *a.unwrap().as_ref().unwrap();

        if a == b'x' {
            if res != 0 { return Err(SyntaxError::MalformedTokenError); }

            self.br.next();
            self.character += 1;
            let a = self.br.peek();
            if a.is_none() { return Err(SyntaxError::MalformedTokenError); }
            let mut a = *a.unwrap().as_ref().unwrap();
            if !a.is_ascii_hexdigit() { return Err(SyntaxError::MalformedTokenError); }

            loop {
                res *= 16;
                res += if a.is_ascii_digit() {
                    a - b'0'
                } else if a.is_ascii_uppercase() {
                    a - b'A' + 10
                } else {
                    a - b'a' + 10
                } as i64;

                self.br.next();
                self.character += 1;

                let next_a = self.br.peek();
                if next_a.is_none() { return Ok(if is_minus { -res } else { res }); }
                a = *next_a.unwrap().as_ref().unwrap();
                if !a.is_ascii_hexdigit() { return Ok(if is_minus { -res } else { res }); }
            }
        } else if a == b'b' {
            if res != 0 { return Err(SyntaxError::MalformedTokenError); }

            self.br.next();
            self.character += 1;
            let a = self.br.peek();
            if a.is_none() { return Err(SyntaxError::MalformedTokenError); }
            let mut a = *a.unwrap().as_ref().unwrap();
            if !(a == b'0' || a == b'1') { return Err(SyntaxError::MalformedTokenError); }

            loop {
                res *= 2;
                res += (a - b'0') as i64;

                self.br.next();
                self.character += 1;

                let next_a = self.br.peek();
                if next_a.is_none() { return Ok(if is_minus { -res } else { res }); }
                a = *next_a.unwrap().as_ref().unwrap();
                if !(a == b'0' || a == b'1') { return Ok(if is_minus { -res } else { res }); }
            }
        } else {
            if !a.is_ascii_digit() { return Ok(if is_minus { -res } else { res }); }

            let mut a = a;
            loop {
                res *= 10;
                res += (a - b'0') as i64;

                self.br.next();
                self.character += 1;

                let next_a = self.br.peek();
                if next_a.is_none() { return Ok(if is_minus { -res } else { res }); }
                a = *next_a.unwrap().as_ref().unwrap();
                if !a.is_ascii_digit() { return Ok(if is_minus { -res } else { res }); }
            }
        }
    }

    fn get_identifier(&mut self) -> Result<Vec<u8>, SyntaxError> {
        let a = self.br.peek();
        if a.is_none() { return Err(SyntaxError::UnknownCharacterError); }
        let a = *a.unwrap().as_ref().unwrap();
        if !(a.is_ascii_alphabetic() || a == b'_') { return Err(SyntaxError::UnknownCharacterError); }
        self.br.next();

        let mut buf = vec![a];
        loop {
            let a = self.br.peek();
            if a.is_none() { break; }
            let a = *a.unwrap().as_ref().unwrap();
            if a.is_ascii_alphanumeric() || a == b'_' {
                buf.push(a);
                self.br.next();
            } else {
                break;
            }
        }

        self.character += buf.len();
        Ok(buf)
    }

    fn get_control(&mut self) -> Result<LexToken, SyntaxError> {
        let a = self.br.peek();
        if a.is_none() { return Ok(LexToken::LexEof); }
        let a = *a.unwrap().as_ref().unwrap();

        let res = match a {
            b':' => LexToken::LexColon,
            b',' => LexToken::LexComma,
            b';' => LexToken::LexSemicolon,
            b'\n' => LexToken::LexNewline,
            _ => { return Err(SyntaxError::UnknownCharacterError); }
        };
        if a == b'\n' {
            self.line += 1;
            self.character = 1;
        } else {
            self.character += 1;
        }
        self.br.next();

        Ok(res)
    }

    fn skip_space(&mut self) {
        loop {
            let a = self.br.peek();
            if a.is_none() { return; }
            let a = *a.unwrap().as_ref().unwrap();

            if a == b'#' {
                loop {
                    self.br.next();
                    self.character += 1;
                    let a = self.br.peek();
                    if a.is_none() { return; }
                    let a = *a.unwrap().as_ref().unwrap();
                    if a == b'\n' { return; }
                }
            }

            if !Self::is_space(a) { return; }

            self.br.next();
            self.character += 1;
        }
    }
}

impl<T: Read> Iterator for Lexer<T> {
    type Item = (LexToken, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_space();
        let (line, ch) = (self.line, self.character);

        let token = self.get_control();
        if token.is_ok() { return Some((token.unwrap(), line, ch)); }

        let token = self.get_digit();
        if token.is_ok() { return Some((LexToken::LexDigit(token.unwrap()), line, ch)); }

        if let Err(SyntaxError::MalformedTokenError) = token {
            println!("at line {}, character {}: Syntax Error", self.line, self.character);
            println!("malformed number.");
            return None;
        }

        let token = self.get_identifier();
        if token.is_err() {
            println!("at line {}, character {}: Syntax Error", self.line, self.character);
            println!("invalid character.");
            return None;
        }

        let token = token.unwrap();
        if token.eq_ignore_ascii_case(b"zero") { return Some((LexToken::LexRegister(Register::Zero), line, ch)); }
        if token.eq_ignore_ascii_case(b"sp") { return Some((LexToken::LexRegister(Register::Sp), line, ch)); }
        if token.eq_ignore_ascii_case(b"fp") { return Some((LexToken::LexRegister(Register::Fp), line, ch)); }
        if token[0] == b'r' {
            let mut n = 0;
            let mut is_label = false;
            for i in 1..token.len() {
                if !token[i].is_ascii_digit() {
                    is_label = true;
                    break;
                }
                n *= 10;
                n += token[i] - b'0';
            }
            if !is_label { return Some((LexToken::LexRegister(Register::R(n)), line, ch)); }
        }

        if token.eq_ignore_ascii_case(b"add") { return Some((LexToken::LexMnemonic(Mnemonic::Add), line, ch)); }
        if token.eq_ignore_ascii_case(b"sub") { return Some((LexToken::LexMnemonic(Mnemonic::Sub), line, ch)); }
        if token.eq_ignore_ascii_case(b"addi") { return Some((LexToken::LexMnemonic(Mnemonic::Addi), line, ch)); }
        if token.eq_ignore_ascii_case(b"subi") { return Some((LexToken::LexMnemonic(Mnemonic::Subi), line, ch)); }
        if token.eq_ignore_ascii_case(b"slli") { return Some((LexToken::LexMnemonic(Mnemonic::Slli), line, ch)); }
        if token.eq_ignore_ascii_case(b"fabs") { return Some((LexToken::LexMnemonic(Mnemonic::Fabs), line, ch)); }
        if token.eq_ignore_ascii_case(b"fneg") { return Some((LexToken::LexMnemonic(Mnemonic::Fneg), line, ch)); }
        if token.eq_ignore_ascii_case(b"fadd") { return Some((LexToken::LexMnemonic(Mnemonic::Fadd), line, ch)); }
        if token.eq_ignore_ascii_case(b"fsub") { return Some((LexToken::LexMnemonic(Mnemonic::Fsub), line, ch)); }
        if token.eq_ignore_ascii_case(b"fmul") { return Some((LexToken::LexMnemonic(Mnemonic::Fmul), line, ch)); }
        if token.eq_ignore_ascii_case(b"fdiv") { return Some((LexToken::LexMnemonic(Mnemonic::Fdiv), line, ch)); }
        if token.eq_ignore_ascii_case(b"fsqrt") { return Some((LexToken::LexMnemonic(Mnemonic::Fsqrt), line, ch)); }
        if token.eq_ignore_ascii_case(b"itof") { return Some((LexToken::LexMnemonic(Mnemonic::Itof), line, ch)); }
        if token.eq_ignore_ascii_case(b"ftoi") { return Some((LexToken::LexMnemonic(Mnemonic::Ftoi), line, ch)); }
        if token.eq_ignore_ascii_case(b"ibeq") { return Some((LexToken::LexMnemonic(Mnemonic::Ibeq), line, ch)); }
        if token.eq_ignore_ascii_case(b"ibne") { return Some((LexToken::LexMnemonic(Mnemonic::Ibne), line, ch)); }
        if token.eq_ignore_ascii_case(b"ible") { return Some((LexToken::LexMnemonic(Mnemonic::Ible), line, ch)); }
        if token.eq_ignore_ascii_case(b"iblt") { return Some((LexToken::LexMnemonic(Mnemonic::Iblt), line, ch)); }
        if token.eq_ignore_ascii_case(b"fblt") { return Some((LexToken::LexMnemonic(Mnemonic::Fblt), line, ch)); }
        if token.eq_ignore_ascii_case(b"fbps") { return Some((LexToken::LexMnemonic(Mnemonic::Fbps), line, ch)); }
        if token.eq_ignore_ascii_case(b"fbng") { return Some((LexToken::LexMnemonic(Mnemonic::Fbng), line, ch)); }
        if token.eq_ignore_ascii_case(b"j") { return Some((LexToken::LexMnemonic(Mnemonic::J), line, ch)); }
        if token.eq_ignore_ascii_case(b"jr") { return Some((LexToken::LexMnemonic(Mnemonic::Jr), line, ch)); }
        if token.eq_ignore_ascii_case(b"call") { return Some((LexToken::LexMnemonic(Mnemonic::Call), line, ch)); }
        // if token.eq_ignore_ascii_case(b"ret") { return Some((LexToken::LexMnemonic(Mnemonic::Ret), line, ch)); }
        if token.eq_ignore_ascii_case(b"movl") { return Some((LexToken::LexMnemonic(Mnemonic::Movl), line, ch)); }
        if token.eq_ignore_ascii_case(b"movh") { return Some((LexToken::LexMnemonic(Mnemonic::Movh), line, ch)); }
        if token.eq_ignore_ascii_case(b"urecv") { return Some((LexToken::LexMnemonic(Mnemonic::Urecv), line, ch)); }
        if token.eq_ignore_ascii_case(b"usend") { return Some((LexToken::LexMnemonic(Mnemonic::Usend), line, ch)); }
        if token.eq_ignore_ascii_case(b"lw") { return Some((LexToken::LexMnemonic(Mnemonic::Lw), line, ch)); }
        if token.eq_ignore_ascii_case(b"sw") { return Some((LexToken::LexMnemonic(Mnemonic::Sw), line, ch)); }
        if token.eq_ignore_ascii_case(b"libeq") { return Some((LexToken::LexMnemonic(Mnemonic::Libeq), line, ch)); }
        if token.eq_ignore_ascii_case(b"libne") { return Some((LexToken::LexMnemonic(Mnemonic::Libne), line, ch)); }
        if token.eq_ignore_ascii_case(b"lible") { return Some((LexToken::LexMnemonic(Mnemonic::Lible), line, ch)); }
        if token.eq_ignore_ascii_case(b"liblt") { return Some((LexToken::LexMnemonic(Mnemonic::Liblt), line, ch)); }
        if token.eq_ignore_ascii_case(b"lfblt") { return Some((LexToken::LexMnemonic(Mnemonic::Lfblt), line, ch)); }
        if token.eq_ignore_ascii_case(b"lfble") { return Some((LexToken::LexMnemonic(Mnemonic::Lfble), line, ch)); }
        if token.eq_ignore_ascii_case(b"lfbps") { return Some((LexToken::LexMnemonic(Mnemonic::Lfbps), line, ch)); }
        if token.eq_ignore_ascii_case(b"lfbng") { return Some((LexToken::LexMnemonic(Mnemonic::Lfbng), line, ch)); }

        Some((LexToken::LexLabel(token), line, ch))
    }
}
