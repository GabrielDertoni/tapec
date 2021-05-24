use std::collections::HashMap;
use std::collections::VecDeque;

use crate::ast;
use crate::parser::Error;

macro_rules! error {
    ($msg:expr, $span:expr) => {
        Err(pest::error::Error::new_from_span(pest::error::ErrorVariant::CustomError { message: $msg.into() }, $span))
    };
}

const TAPE_SIZE: usize = 256;

fn as_prim_op<'a>(inst: &ast::Inst<'a>) -> Vec<ast::Stmt<'a>> {
    match inst.op {
        ast::Op::Hlt |
        ast::Op::Add |
        ast::Op::Mul |
        ast::Op::Cle |
        ast::Op::Ceq |
        ast::Op::Jmp |
        ast::Op::Beq |
        ast::Op::Cpy |
        ast::Op::Put => vec![ast::Stmt::Inst(inst.clone())],
    }
}

struct GenState<'a> {
    tape: Vec<ast::Inst<'a>>,
    labels: HashMap<ast::Label<'a>, usize>,
}

#[derive(Debug, Clone)]
struct Block<'a> {
    start: usize,
    labels: HashMap<&'a str, usize>,
}

struct TapeWriter {
    tape: Vec<i32>,
    idx: usize,
    data_idx: usize,
}

impl TapeWriter {
    fn new(data_idx: usize) -> TapeWriter {
        TapeWriter {
            tape: vec![0; TAPE_SIZE],
            idx: 0,
            data_idx,
        }
    }

    fn write(&mut self, val: i32) -> usize {
        let i = self.idx;
        self.tape[i] = val;
        self.idx += 1;
        i
    }

    fn write_str(&mut self, s: &str) -> usize {
        let i = self.idx;
        for c in s.chars() {
            self.write(c as i32);
        }
        i
    }

    fn write_end(&mut self, val: i32) -> Option<usize> {
        let i = self.data_idx;
        let pos = self.tape.get_mut(i)?;
        *pos = val;
        self.data_idx += 1;
        Some(i)
    }

    fn write_end_str(&mut self, s: &str) -> Option<usize> {
        let i = self.idx;
        for c in s.chars() {
            self.write_end(c as i32)?;
        }
        Some(i)
    }
}

fn expand_lit<'a>(
    lit: &ast::Lit<'a>,
    code_size: usize,
    curr_ctxt: &Option<(&&str, &Block)>,
    blocks: &HashMap<&str, Block>,
    tape: &mut TapeWriter,
) -> Result<ast::Spanned<'a, i32>, Error> {
    match lit {
        ast::Lit::Num(n)   => Ok(ast::Spanned::new(**n, n.span.clone())),
        ast::Lit::Chr(c)   => Ok(ast::Spanned::new(**c as i32, c.span.clone())),
        ast::Lit::Lbl(lbl) => {
            let span = lbl.span.clone();
            if lbl.starts_with('.') {
                if let Some((_, block)) = curr_ctxt {
                    if let Some(off) = block.labels.get(lbl.as_ref()) {
                        Ok(ast::Spanned::new(*off as i32, span))
                    } else {
                        error!("label used but not defined", span)
                    }
                } else {
                    error!("local label used in global location", span)
                }
            } else if let Some(block) = blocks.get(lbl.as_ref()) {
                Ok(ast::Spanned::new(block.start as i32, span))
            } else {
                error!("label used but not defined", span)
            }
        },
        ast::Lit::Ref(lit_ref) => {
            if let ast::Lit::Str(s) = lit_ref.as_ref() {
                let span = s.span.clone();
                if let Some(v) = tape.write_end_str(s.as_str()) {
                    Ok(ast::Spanned::new(v as i32, span))
                } else {
                    error!("tape size exceeded", span)
                }
            } else {
                let res = expand_lit(lit_ref, code_size, curr_ctxt, blocks, tape)?;
                if let Some(val) = tape.write_end(*res) {
                    Ok(ast::Spanned::new(val as i32, res.span))
                } else {
                    error!("tape size exceeded", res.span)
                }
            }
        },
        ast::Lit::Str(_)   => unreachable!(),
    }
}

struct Desugarer<'a> {
    stmts: &'a [ast::Stmt<'a>],
    sugar_queue: VecDeque<ast::Stmt<'a>>,
}

impl<'a> Desugarer<'a> {
    fn new(prog: &'a ast::Prog<'a>) -> Desugarer<'a> {
        Desugarer {
            stmts: &prog.stmts,
            sugar_queue: VecDeque::new(),
        }
    }

    fn desugar(&mut self, stmt: &'a ast::Stmt<'a>) {
        match stmt {
            ast::Stmt::Inst(inst) =>
                self.sugar_queue.extend(as_prim_op(inst)),

            ast::Stmt::Lit(_)  |
            ast::Stmt::Label(_) =>
                self.sugar_queue.push_back(stmt.clone()),
        }
    }
}

impl<'a> Iterator for Desugarer<'a> {
    type Item = ast::Stmt<'a>;

    fn next(&mut self) -> Option<ast::Stmt<'a>> {
        loop {
            if let Some(dequeued) = self.sugar_queue.pop_front() {
                break Some(dequeued);
            } else {
                let (fst, rest) = self.stmts.split_first()?;
                self.stmts = rest;
                self.desugar(&fst);
            }
        }
    }
}

pub fn lit_size(lit: &ast::Lit) -> usize {
    match lit {
        ast::Lit::Str(s) => s.len(),
        _                => 1,
    }
}

pub fn gen_code(prog: &ast::Prog) -> Result<Vec<i32>, Error> {
    let mut blocks: HashMap<&str, Block> = HashMap::new();
    let mut curr_ctxt: Option<(ast::Label, Block)> = None;

    let mut desugared = Vec::new();

    let mut off = 0;
    // Solve labels and desugar.
    for stmt in Desugarer::new(prog) {
        match &stmt {
            ast::Stmt::Label(lbl) if !lbl.starts_with(".") => {
                if let Some((name, block)) = curr_ctxt.take() {
                    blocks.insert(name.as_ref(), block);
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
                    block.labels.insert(lbl.as_ref(), off);
                } else {
                    return error!("No parent label", lbl.span.clone());
                }
            },
            ast::Stmt::Inst(inst) => {
                off += 1 + inst.args.len();
            },
            ast::Stmt::Lit(lit) => {
                off += lit_size(lit);
            },
        }
        desugared.push(stmt);
    }

    if let Some((name, block)) = curr_ctxt.take() {
        blocks.insert(name.as_ref(), block);
    }

    let mut curr_ctxt: Option<(&&str, &Block)> = None;
    let mut tape = TapeWriter::new(off);

    // Build tape
    for stmt in &desugared {
        match stmt {
            ast::Stmt::Inst(inst) => {
                tape.write(inst.op as i32);

                for arg in &inst.args {
                    let val = expand_lit(arg, off, &curr_ctxt, &blocks, &mut tape)?;
                    tape.write(*val);
                }
            },
            ast::Stmt::Label(lbl) if !lbl.starts_with('.') => {
                let ctxt = blocks.get_key_value(lbl.as_ref()).unwrap();
                curr_ctxt.replace(ctxt);
            },
            ast::Stmt::Label(_) => (),
            ast::Stmt::Lit(lit) => {
                if let ast::Lit::Str(s) = lit {
                    tape.write_str(s.as_ref());
                } else {
                    let val = expand_lit(lit, off, &curr_ctxt, &blocks, &mut tape)?;
                    tape.write(*val);
                }
            },
        }
    }

    Ok(tape.tape)
}
