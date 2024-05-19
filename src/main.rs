mod display;
mod expr;
mod lexer;
mod parser;
mod semantics;
mod stmt;
mod token;
mod typed_ast;
mod types;
mod cgen;

use lexer::*;
use parser::*;
use semantics::generate_typed_ast;

use std::process::exit;
use std::{fs::File, io::Read};

fn main() {
    let mut f = File::open("program.txt").unwrap();
    let mut text = String::new();
    f.read_to_string(&mut text).unwrap();
    text = text.trim().to_string();
    text.push('\0');

    let mut lexer = Lexer::new(text).unwrap();
    let lexemes = match lexer.lex() {
        Ok(t) => t,
        Err(_) => {
            println!("Lexing Error at Line: {} Col: {}", lexer.line, lexer.col);
            exit(0);
        }
    };

    for l in &lexemes {
        println!("{}", l);
    }

    println!("");

    let mut parser = Parser::new(lexemes);
    let ast = match parser.parse() {
        Ok(t) => t,
        Err(e) => {
            println!("{}", e);
            exit(0);
        }
    };

    for s in &ast {
        println!("{}", s);
    }

    let (typed_ast, custom_types) = match generate_typed_ast(ast) {
        Ok(t) => t,
        Err(e) => {
            println!("{:?}", e);
            exit(0);
        }
    };

    println!("\n---\n");

    for t in custom_types {
        println!("{:#?}", t);
    }

    println!("\n---\n");

    for stmt in typed_ast {
        println!("{:#?}", stmt);
    }
}
