#![allow(dead_code)]

extern crate pest;

#[macro_use]
extern crate pest_derive;

mod ast;
mod parser;
mod codegen;

use crate::parser::parse_asm;
use crate::codegen::gen_code;

fn main() {
    let prog = r#"
        main:
            put &'h'
            put &'e'
            put &'l'
            put &'l'
            put &'o'
            put &','
            put &' '
            put &'w'
            put &'o'
            put &'r'
            put &'l'
            put &'d'
            put &'!'
            put &'\n'
            hlt
    "#;

    match parse_asm(prog) {
        Ok(stmts) => {
            match gen_code(stmts) {
                Ok(tape) => {
                    for n in tape {
                        println!("{}", n);
                    }
                },
                Err(e)   => eprintln!("{}", e),
            }
        },
        Err(e) => eprintln!("{}", e),
    }

    /*
    match ASMParser::parse(Rule::asm, prog) {
        Ok(stmts) => {
            for stmt in stmts {
                println!("{:#?}", stmt);
            }
        },
        Err(e) => {
            println!("{}", e);
        },
    }
    */
}
