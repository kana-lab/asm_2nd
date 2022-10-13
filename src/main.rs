use std::env::args;
use std::fs::File;
use std::io::BufReader;
use asm_1st::encoder;
use asm_1st::lexer::Lexer;
use asm_1st::parser::Parser;

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
    let par = Parser::new(lex);
    let (inst, map) = par.parse().unwrap();
    let binary = encoder::encode(inst, map).unwrap();
    for b in binary {
        println!("{:<08x}", b);
    }
}
