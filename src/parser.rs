use pest_derive::Parser;
use pest::Parser;
use pest::iterators::Pair;

use crate::ast::*;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct ASMParser;

pub type Error = pest::error::Error<Rule>;

macro_rules! error {
    ($msg:expr, $span:expr) => {
        Err(pest::error::Error::new_from_span(pest::error::ErrorVariant::CustomError { message: $msg.into() }, $span))
    };
}

fn parse_label(pair: Pair<Rule>) -> Result<Spanned<&str>, Error> {
    let ident = pair
        .into_inner()
        .next()
        .unwrap();

    Ok(Spanned::new(ident.as_str(), ident.as_span()))
}

fn extract_str(s: &str) -> &str {
    &s[1..s.len()-1]
}

fn extract_chr(s: &str) -> char {
    match &s[1..s.len()-1] {
        "\\n"  => '\n',
        "\\r"  => '\r',
        "\\t"  => '\t',
        "\\\\" => '\\',
        "\\'"  => '\'',
        s      => s.chars().nth(0).unwrap(),
    }
}

fn parse_arg(pair: Pair<Rule>) -> Result<Arg, Error> {
    let arg = pair
        .into_inner()
        .next()
        .unwrap();

    let span = arg.as_span();
    let parsed = match arg.as_rule() {
        Rule::lbl_arg => Arg::Lbl(Spanned::new(arg.as_str(), span)),
        Rule::num_arg => match arg.as_str().parse() {
            Ok(n)  => Arg::Num(Spanned::new(n, span)),
            Err(e) => return error!(e.to_string(), span),
        },
        Rule::str_arg => Arg::Str(Spanned::new(extract_str(arg.as_str()), span)),
        Rule::chr_arg => Arg::Chr(Spanned::new(extract_chr(arg.as_str()), span)),
        Rule::lit_arg => Arg::Lit(Box::new(parse_arg(arg.into_inner().next().unwrap())?)),
        _             => return error!("unexpected arg type", span),
    };

    Ok(parsed)
}

fn parse_inst(pair: Pair<Rule>) -> Result<Inst, Error> {
    let mut inst_iter = pair.into_inner();
    let ident = inst_iter.next().unwrap();

    let op = match ident.as_str() {
        "hlt" => Op::Hlt,
        "add" => Op::Add,
        "mul" => Op::Mul,
        "cle" => Op::Cle,
        "ceq" => Op::Ceq,
        "jmp" => Op::Jmp,
        "beq" => Op::Beq,
        "cpy" => Op::Cpy,
        "put" => Op::Put,
        _ => return error!("not a valid instruction", ident.as_span()),
    };

    let span = ident.as_span();
    let arg_lst: Vec<_> = inst_iter.collect();

    if arg_lst.len() == op.nargs() {
        let args: Result<Vec<_>, _> = arg_lst.into_iter().map(parse_arg).collect();
        let args = args?;
        Ok(Inst { op, args, span })
    } else {
        error!(format!("expected {} argument(s) but got {}", op.nargs(), arg_lst.len()), span)
    }
}

fn parse_stmt(pair: Pair<Rule>) -> Result<Stmt, Error> {
    let stmt = pair
        .into_inner()
        .next()
        .unwrap();

    if stmt.as_rule() == Rule::label {
        Ok(Stmt::Label(parse_label(stmt)?))
    } else if stmt.as_rule() == Rule::inst {
        Ok(Stmt::Inst(parse_inst(stmt)?))
    } else {
        unreachable!()
    }
}

pub fn parse_asm(program: &str) -> Result<Prog, Error> {
    let prog = ASMParser::parse(Rule::asm, program)?.next().unwrap();
    let span = prog.as_span();
    let res: Result<Vec<_>, _> = prog
        .into_inner()
        .filter(|stmt| stmt.as_rule() == Rule::stmt)
        .map(parse_stmt)
        .collect();

    Ok(Prog {
        stmts: res?,
        span,
    })
}
