#![allow(dead_code)]
#![feature(box_patterns)]
#![feature(btree_drain_filter)]

use clap::clap_app;

use std::io::Write;
use std::fs;

mod ast;
mod parser;
mod codegen;

use crate::parser::parse_asm;
use crate::codegen::Assembler;

fn main() -> std::io::Result<()> {
    let matches = clap_app!(tapec =>
        (version: "0.1.0")
        (author: "Gabriel Dertoni <gab.dertoni@gmail.com>")
        (about: "A compiler for the Tape programming language")
        (@arg SOURCE: +required "The TapeLang source file to compile")
        (@arg output: -o --output +takes_value "Output compiled tape")
        (@arg size: -s --size +takes_value "Size of the tape")
        (@arg expand: -E --expand "Compile and also print the desugared code")
    ).get_matches();

    // Ok, SOURCE is required.
    let src_file = matches.value_of("SOURCE").unwrap();
    let out = matches.value_of("output").unwrap_or("a.out");
    let expand = matches.is_present("expand");
    let tape_size = matches.value_of("size")
        .map(str::parse)
        .unwrap_or(Ok(256));

    let tape_size = match tape_size {
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("{}", matches.usage());
            std::process::exit(1)
        },
        Ok(n) => n,
    };

    let source = fs::read_to_string(src_file)?;

    match parse_asm(&source).and_then(|p| {
        let asm = Assembler::new(tape_size, expand);
        asm.assemble(&p.stmts)
    }) {
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
