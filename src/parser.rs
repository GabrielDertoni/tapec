use pest_derive::Parser;
use pest::Parser;
use pest::Span;
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

fn extract_str(s: &str, span: Span) -> Result<String, Error> {
    let mut new = String::new();
    let inner = &s[1..s.len()-1];
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        let mut translated = c;

        if c == '\\' {
             translated = match chars.next().unwrap() {
                'n'  => '\n',
                'r'  => '\r',
                't'  => '\t',
                '\\' => '\\',
                '"'  => '"',
                '0'  => '\0',
                any  => return error!(format!("not a valid escape '{}'", any), span),
            };
        }

        new.push(translated);
    }
    Ok(new)
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

fn parse_lbl(pair: Pair<Rule>) -> Result<Spanned<&str>, Error> {
    let ident = pair
        .into_inner()
        .next()
        .unwrap();

    Ok(Spanned::new(ident.as_str(), ident.as_span()))
}

fn parse_lit(pair: Pair<Rule>) -> Result<Lit, Error> {
    let lit = pair
        .into_inner()
        .next()
        .unwrap();

    let span = lit.as_span();
    let parsed = match lit.as_rule() {
        Rule::lbl     => Lit::Lbl(parse_lbl(lit)?),
        Rule::num     => match lit.as_str().parse() {
                            Ok(n)  => Lit::Num(Spanned::new(n, span)),
                            Err(e) => return error!(e.to_string(), span),
                         },
        Rule::str     => Lit::Str(Spanned::new(extract_str(lit.as_str(), span.clone())?, span)),
        Rule::chr     => Lit::Chr(Spanned::new(extract_chr(lit.as_str()), span)),
        Rule::lit_ref => Lit::Ref(Box::new(parse_lit(lit.into_inner().next().unwrap())?)),
        _             => unreachable!(),
    };

    Ok(parsed)
}

fn extract_arg_lbl<'a>(s: &'a str, span: Span<'a>) -> Result<Label<'a>, Error> {
    let len = s.len();
    let inner = &s[1..len-1];
    if inner.chars().nth(0).unwrap() == '.' {
        Ok(Spanned::new(inner, span))
    } else {
        error!("only local labels can be argument labels", span)
    }
}

fn parse_arg(pair: Pair<Rule>) -> Result<Arg, Error> {
    let arg = pair
        .into_inner()
        .next()
        .unwrap();

    let span = arg.as_span();
    let parsed = match arg.as_rule() {
        Rule::lit     => Arg::Lit(parse_lit(arg)?),
        Rule::arg_lbl => {
            let lbl = extract_arg_lbl(arg.as_str(), span.clone())?;
            Arg::Lbl(lbl)
        },
        _             => unreachable!(),
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
        _     => return error!("not a valid instruction", ident.as_span()),
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

    match stmt.as_rule() {
        Rule::label => Ok(Stmt::Label(parse_label(stmt)?)),
        Rule::inst  => Ok(Stmt::Inst(parse_inst(stmt)?)),
        Rule::lit   => Ok(Stmt::Lit(parse_lit(stmt)?)),
        _           => unreachable!(),
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
