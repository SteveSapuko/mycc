mod token;
mod lexer;

use lexer::*;

use std::process::exit;
use std::{fs::File, io::Read};

fn main() {
    let mut f = File::open("program.txt").unwrap();
    let mut text = String::new();
    f.read_to_string(&mut text).unwrap();
    text = text.trim().to_string();
    text.push('\0');

    let mut lexer = Lexer::new(text).unwrap();
    let lexemes = 
    match lexer.lex() {
        Ok(t) => t,
        Err(_) => {
            println!("Lexing Error at Line: {} Col: {}", lexer.line, lexer.col);
            exit(0);
        }
    };

    for l in lexemes {
        println!("{}", l);
    }
}
