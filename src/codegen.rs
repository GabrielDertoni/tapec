use std::collections::HashMap;

use crate::ast;
use crate::parser::Error;

macro_rules! error {
    ($msg:expr, $span:expr) => {
        Err(pest::error::Error::new_from_span(pest::error::ErrorVariant::CustomError { message: $msg.into() }, $span))
    };
}


/*
fn assemble_bound_labels<'a>(stmts: &Vec<ast::Stmt<'a>>) -> Vec<ast::Stmt<'a>> {
    let mut tape = Vec::new();
    let mut labels = HashMap::new();

    let lbls = stmts
        .iter()
        .enumerate()
        .filter_map(|stmt|
            if let Stmt::Label(lbl) = stmt {
                Some(lbl)
            } else {
                None
            }
        );

    for lbl in lbl_stmts
    {
        match stmt {
            ast::Stmt::Inst(inst) => {
                
            },
            ast::Stmt::Label(lbl) => {
                labels.insert(lbl, tape.len());
            },
        }
    }
    tape
}
*/

fn as_prim_op<'a>(inst: &ast::Inst<'a>) -> Result<Vec<ast::Inst<'a>>, Error> {
    match inst.op {
        ast::Op::Hlt |
        ast::Op::Add |
        ast::Op::Mul |
        ast::Op::Cle |
        ast::Op::Ceq |
        ast::Op::Jmp |
        ast::Op::Beq |
        ast::Op::Cpy |
        ast::Op::Put => Ok(vec![inst.clone()]),
    }
}

struct GenState<'a> {
    tape: Vec<ast::Inst<'a>>,
    labels: HashMap<ast::Label<'a>, usize>,
}

#[derive(Clone)]
struct Block<'a> {
    start: usize,
    labels: HashMap<ast::Label<'a>, usize>,
}

fn arg_value(
    arg: &ast::Arg,
    code_size: usize,
    curr_ctxt: &Option<(&ast::Label, &Block)>,
    blocks: &HashMap<ast::Label, Block>,
    values: &mut Vec<i32>
) -> Result<i32, Error>
{
    match arg {
        ast::Arg::Num(n)   => Ok(**n),
        ast::Arg::Lbl(lbl) => {
            if lbl.starts_with('.') {
                if let Some((_, block)) = curr_ctxt {
                    if let Some(off) = block.labels.get(lbl) {
                        Ok(*off as i32)
                    } else {
                        error!("label used but not defined", lbl.span.clone())
                    }
                } else {
                    error!("local label used in global location", lbl.span.clone())
                }
            } else if let Some(block) = blocks.get(lbl) {
                Ok(block.start as i32)
            } else {
                error!("label used but not defined", lbl.span.clone())
            }
        },
        ast::Arg::Str(s) => error!("Not possible", s.span.clone()),
        ast::Arg::Chr(c) => Ok(**c as i32),
        ast::Arg::Lit(l) => {
            let res = arg_value(l, code_size, curr_ctxt, blocks, values);
            values.push(res?);
            Ok((code_size + values.len() - 1) as i32)
        },
    }
}

pub fn gen_code(stmts: Vec<ast::Stmt>) -> Result<Vec<i32>, Error> {

    let mut tape = Vec::new();
    // let mut desugared = Vec::new();
    let mut values = Vec::new();
    let mut blocks: HashMap<ast::Label, Block> = HashMap::new();
    let mut curr_ctxt: Option<(ast::Label, Block)> = None;

    let mut off = 0;
    // Solve labels and desugar.
    for stmt in &stmts {
        match stmt {
            ast::Stmt::Label(lbl) if !lbl.starts_with(".") => {
                if let Some((name, block)) = curr_ctxt.take() {
                    blocks.insert(name, block);
                }
                curr_ctxt.replace((
                    lbl.clone(),
                    Block {
                        start: off,
                        labels: HashMap::new()
                    })
                );
            },
            ast::Stmt::Label(lbl) => {
                if let Some((_, ref mut block)) = curr_ctxt {
                    block.labels.insert(lbl.clone(), off);
                } else {
                    return error!("No parent label", lbl.span.clone());
                }
            },
            ast::Stmt::Inst(inst) => {
                off += 1 + inst.args.len();
            }
        }
    }

    if let Some((name, block)) = curr_ctxt.take() {
        blocks.insert(name, block);
    }

    let mut curr_ctxt: Option<(&ast::Label, &Block)> = None;
    // Build tape
    for stmt in &stmts {
        match stmt {
            ast::Stmt::Inst(inst) => {
                tape.push(inst.op as i32);

                for arg in &inst.args {
                    tape.push(arg_value(arg, off, &curr_ctxt, &blocks, &mut values)?);
                }
            },
            ast::Stmt::Label(lbl) if !lbl.starts_with('.') => {
                let ctxt = blocks.get_key_value(&lbl).unwrap();
                curr_ctxt.replace(ctxt);
            },
            ast::Stmt::Label(_) => (),
        }
    }

    tape.extend(values);
    Ok(tape)
}
