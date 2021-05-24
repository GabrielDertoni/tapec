#![allow(dead_code)]

use clap::clap_app;

use std::io::Write;
use std::fs;

mod ast;
mod parser;
mod codegen;

use crate::parser::parse_asm;
use crate::codegen::gen_code;

fn main() -> std::io::Result<()> {
    let matches = clap_app!(tapec =>
        (version: "0.1.0")
        (author: "Gabriel Dertoni <gab.dertoni@gmail.com>")
        (about: "A compiler for the Tape programming language")
        (@arg SOURCE: +required "The TapeLang source file to compile")
        (@arg output: -o --output +takes_value "Output compiled tape")
    ).get_matches();

    // Ok, SOURCE is required.
    let src_file = matches.value_of("SOURCE").unwrap();
    let out = matches.value_of("output").unwrap_or("a.out");

    let source = fs::read_to_string(src_file)?;

    match parse_asm(&source).and_then(|p| gen_code(&p)) {
        Ok(tape) => {
            if out == "-" {
                for n in tape {
                    println!("{}", n);
                }
            } else {
                let mut file = fs::File::create(out)?;
                for n in tape {
                    writeln!(file, "{}", n)?;
                }
            }
        },
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}
