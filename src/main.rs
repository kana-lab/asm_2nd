use std::env::args;
use std::fs::File;
use std::io::BufReader;
use log::{debug, error};
use asm_1st::lexer::{Lexer, LexToken};

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        println!("[usage:] ./asm_1st path/to/assembly");
        return;
    }

    let path = &args[1];
    let f = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            println!("could not open file: {}", e);
            return;
        }
    };
    let br = BufReader::new(f);
    let lex = Lexer::new(br);
    for token in lex.into_iter() {
        println!("{:?}", token);
        if let LexToken::LexEof = token { break; }
    }
}
