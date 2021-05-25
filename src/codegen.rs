#![allow(unused_macros)]

use std::collections::HashMap;
use std::collections::VecDeque;

use crate::ast;
use crate::parser::Error;

macro_rules! error {
    ($msg:expr, $span:expr) => {
        Err(pest::error::Error::new_from_span(pest::error::ErrorVariant::CustomError { message: $msg.into() }, $span))
    };
}

// Evaluates to Some(expr) if the pattern matches. Else evaluates to None.
/*
macro_rules! mextract {
    ($to_match:expr, $pat:pat => $expr:expr) => {
        match $to_match {
            $pat => Some($expr),
            _    => None,
        }
    };
}
*/

macro_rules! inst_args {
    ($span:expr => (@lit $lit:expr)) => {
        ast::Arg::Lit($lit)
    };

    ($span:expr => (@lbl $lbl:expr)) => {
        ast::Arg::Lbl($lbl)
    };

    ($span:expr => $lit:expr) => {
        $lit
    };

    ($span:expr =>) => { };
}

macro_rules! inst {
    ($span:expr => $inst:ident, $($tail:tt),*) => {
        ast::Inst::new(ast::Op::$inst, vec![$(inst_args!($span => $tail)),*], $span)
    };
}

macro_rules! stmt {
    ($span:expr => label $name:expr) => {
        ast::Stmt::Label(ast::mk_lbl(name, $span.clone()))
    };

    ($span:expr => $($toks:tt)*) => {
        ast::Stmt::Inst(inst!($span => $($toks)*))
    };
}

macro_rules! stmts {
    ($span:expr => $([$($toks:tt)*])+) => {
        vec![$(stmt!($span => $($toks)*)),+]
    };
}

const TAPE_SIZE: usize = 256;
const EMPTY_DEFAULT: i32 = -1;

struct Desugarer<'a> {
    stmts: &'a [ast::Stmt<'a>],
    sugar_queue: VecDeque<ast::Stmt<'a>>,
    head_pos: usize,
}

impl<'a> Desugarer<'a> {
    fn new(prog: &'a ast::Prog<'a>) -> Desugarer<'a> {
        Desugarer {
            stmts: &prog.stmts,
            sugar_queue: VecDeque::new(),
            head_pos: 0,
        }
    }

    fn desugar(&mut self, stmt: &ast::Stmt<'a>) {
        match stmt {
            ast::Stmt::Inst(inst) => desugar_inst(inst, self),
            ast::Stmt::Lit(_)  |
            ast::Stmt::Label(_) =>
                self.sugar_queue.push_back(stmt.clone()),
        }
    }

    fn push_sugar(&mut self, stmt: ast::Stmt<'a>) {
        self.sugar_queue.push_back(stmt);
    }

    fn get_head_pos(&self) -> usize { self.head_pos }
}

fn solve_arg_deref<'a>(
    deref_to: ast::Lit<'a>,
    lbl: ast::Label<'a>,
    pos: usize,
    depth: usize,
    stmts: &mut Vec<ast::Stmt<'a>>
) {
    match deref_to {
        ast::Lit::Deref(box deref) => {
            let cpy_lbl = ast::mk_lbl(format!(":__cpy_{}_{}", pos, depth + 1), deref.span());
            solve_arg_deref(deref.clone(), cpy_lbl.clone(), pos, depth + 1, stmts);
            let lbl_lit = ast::Lit::Lbl(lbl);

            let stmt = stmt!(deref.span() => Cpy, (@lbl cpy_lbl), (@lit lbl_lit));
            stmts.push(stmt);
        },
        other => {
            let lbl_lit = ast::Lit::Lbl(lbl);
            let span = other.span();
            stmts.push(stmt!(span => Cpy, (@lit other), (@lit lbl_lit)));
        },
    }
}

fn desugar_arg_deref<'a>(
    inst: &ast::Inst<'a>,
    res: &mut Vec<ast::Stmt<'a>>,
    pos: usize,
    depth: usize,
) {
    // put *'ptr
    // ------------------
    // cpy 'ptr ':arg_1_1
    // put <:arg_1_1>
    //
    //
    // put **'ptr
    // ------------------
    // cpy 'ptr ':arg_2_1
    // cpy <:arg_2_1> ':arg_1_1
    // put <:arg_1_1>

    let mut new_args = Vec::with_capacity(inst.args.len());

    for arg in &inst.args {
        let new_arg = match arg {
            ast::Arg::Lit(ast::Lit::Deref(box d)) => {
                let lbl = ast::mk_lbl(format!(":__arg_{}_{}", pos, depth), d.span());
                solve_arg_deref(d.clone(), lbl.clone(), pos, depth, res);
                ast::Arg::Lbl(lbl)
            },
            otherwise => otherwise.clone(),
        };
        new_args.push(new_arg);
    }
    res.push(ast::Stmt::Inst(ast::Inst::new(inst.op, new_args, inst.span.clone())));
}

/*
fn solve_imediate_labels(stmts: Vec<ast::Stmt>) -> Vec<ast::Stmt> {
    let mut labels = HashMap::new();
    let mut off = 0;

    for stmt in &stmts {
        match stmt {
            ast::Stmt::Label(lbl) if lbl.starts_with(":") => {
                if let Some(_) = labels.insert((*lbl).clone(), off) {
                    panic!("Dupliate direct labels!!!");
                }
            },
            ast::Stmt::Inst(inst) => {
                off += 1;
                for arg in &inst.args {
                    match arg {
                        ast::Arg::Lbl(lbl) if lbl.starts_with(":") =>
                            if let Some(_) = labels.insert((*lbl).clone(), off) {
                                panic!("Dupliate direct labels!!!");
                            },
                        _ => (),
                    }
                    off += 1;
                }
            },
            _ => (),
        }
    }

    let new_stmts = Vec::new();
    for stmt in stmts {
        match stmt {
            ast::Stmt::Inst(inst) => {
            },
            // TODO: Maybe here we should handle literals?
            _ => (),
        }
    }

    new_stmts
}
*/

fn desugar_inst<'a>(inst: &ast::Inst<'a>, desugarer: &mut Desugarer<'a>) {
    fn desugar_inst_rec<'a>(inst: &ast::Inst<'a>, pos: usize, depth: usize) -> Vec<ast::Stmt<'a>> {
        let mut res = Vec::new();
        let op = inst.op;
        match op {
            ast::Op::Hlt |
            ast::Op::Add |
            ast::Op::Mul |
            ast::Op::Cle |
            ast::Op::Ceq |
            ast::Op::Jmp |
            ast::Op::Beq |
            ast::Op::Cpy |
            ast::Op::Put |
            ast::Op::Ptn => {
                desugar_arg_deref(inst, &mut res, pos, depth);
            },
        }
        res
    }

    let res = desugar_inst_rec(inst, 0, desugarer.get_head_pos());
    // solve_imediate_labels(res);
    desugarer.sugar_queue.extend(res);
}

struct GenState<'a> {
    tape: Vec<ast::Inst<'a>>,
    labels: HashMap<ast::Label<'a>, usize>,
}

#[derive(Debug, Clone)]
struct Block {
    start: usize,
    labels: HashMap<String, usize>,
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

    fn val_at(&self, idx: usize) -> Option<i32> {
        if idx < self.idx || idx < self.data_idx {
            self.tape.get(idx).map(Clone::clone)
        } else {
            None
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

fn get_lbl<'a>(
    lbl: &ast::Label<'a>,
    curr_ctxt: &Option<(&String, &Block)>,
    blocks: &HashMap<String, Block>
) -> Result<ast::Spanned<'a, i32>, Error> {
    let span = lbl.span.clone();
    if lbl.starts_with('.') | lbl.starts_with(':') {
        if let Some((_, block)) = curr_ctxt {
            if let Some(off) = block.labels.get(lbl.as_str()) {
                Ok(ast::Spanned::new(*off as i32, span))
            } else {
                error!(format!("label \"{}\" used but not defined", lbl), span)
            }
        } else {
            error!("local label used in global location", span)
        }
    } else if let Some(block) = blocks.get(lbl.as_str()) {
        Ok(ast::Spanned::new(block.start as i32, span))
    } else {
        error!(format!("label \"{}\" used but not defined", lbl), span)
    }
}

fn expand_lit<'a>(
    lit: &ast::Lit<'a>,
    code_size: usize,
    curr_ctxt: &Option<(&String, &Block)>,
    blocks: &HashMap<String, Block>,
    tape: &mut TapeWriter,
) -> Result<ast::Spanned<'a, i32>, Error> {
    match lit {
        ast::Lit::Num(n)   => Ok(ast::Spanned::new(**n, n.span.clone())),
        ast::Lit::Chr(c)   => Ok(ast::Spanned::new(**c as i32, c.span.clone())),
        ast::Lit::Lbl(lbl) => get_lbl(lbl, curr_ctxt, blocks),
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

        // At this point, we are already supposed to have resolved derefs in argument position all
        // that is left is derefs in literal position.
        ast::Lit::Deref(box lit) => {
            let addr = match lit {
                ast::Lit::Ref(box lit)   => expand_lit(lit, code_size, curr_ctxt, blocks, tape)?,
                ast::Lit::Deref(box lit) => expand_lit(lit, code_size, curr_ctxt, blocks, tape)?,
                ast::Lit::Lbl(lbl)       => get_lbl(lbl, curr_ctxt, blocks)?,

                // This should never happen because the parser ensures it.
                _                        => unreachable!(),
            };

            if let Some(val) = tape.val_at(*addr as usize) {
                Ok(ast::Spanned::new(val, addr.span))
            } else {
                // If a label is defined afterwards and a literal dereference to it is made before,
                // it will not be able to calculate the dereference because the literal beeing
                // accessed was not created already.
                error!("cannot be dereferenced, at least not at compile time", addr.span)
            }
        },
    }
}

impl<'a> Iterator for Desugarer<'a> {
    type Item = ast::Stmt<'a>;

    fn next(&mut self) -> Option<ast::Stmt<'a>> {
        loop {
            if let Some(dequeued) = self.sugar_queue.pop_front() {
                self.head_pos += 1;
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

pub fn gen_code(prog: &ast::Prog, print_desugared: bool) -> Result<Vec<i32>, Error> {
    let mut blocks: HashMap<String, Block> = HashMap::new();
    let mut curr_ctxt: Option<(ast::Label, Block)> = None;

    let mut desugared = Vec::new();

    let mut off = 0;
    // Solve labels and desugar.
    for stmt in Desugarer::new(prog) {
        match &stmt {
            ast::Stmt::Label(lbl) if lbl.starts_with('.') | lbl.starts_with(':') => {
                if let Some((_, ref mut block)) = curr_ctxt {
                    block.labels.insert(lbl.to_string(), off);
                } else {
                    return error!("No parent label", lbl.span.clone());
                }
            },
            ast::Stmt::Label(lbl) => {
                if let Some((name, block)) = curr_ctxt.take() {
                    blocks.insert(name.to_inner(), block);
                }
                curr_ctxt.replace((
                    lbl.clone(),
                    Block {
                        start: off,
                        labels: HashMap::new()
                    })
                );
            },
            ast::Stmt::Inst(inst) => {
                off += 1;
                for arg in &inst.args {
                    if let ast::Arg::Lbl(lbl) = arg {
                        if let Some((_, ref mut block)) = curr_ctxt {
                            block.labels.insert(lbl.to_string(), off);
                        } else {
                            return error!("No parent label", lbl.span.clone());
                        }
                    }
                    off += 1;
                }
            },
            ast::Stmt::Lit(lit) => {
                off += lit_size(lit);
            },
        }
        desugared.push(stmt);
    }

    if print_desugared {
        let mut lvl = 0;
        for stmt in &desugared {
            if matches!(stmt, ast::Stmt::Label(_)) {
                lvl = 0;
                println!("{:indent$}{}", "", stmt, indent=lvl);
                lvl = 4;
            } else {
                println!("{:indent$}{}", "", stmt, indent=lvl);
            }
        }
    }

    if let Some((name, block)) = curr_ctxt.take() {
        blocks.insert(name.to_inner(), block);
    }

    let mut curr_ctxt: Option<(&String, &Block)> = None;
    let mut tape = TapeWriter::new(off);

    // Build tape
    for stmt in &desugared {
        match stmt {
            ast::Stmt::Inst(inst) => {
                tape.write(inst.op as i32);

                for arg in &inst.args {
                    match arg {
                        ast::Arg::Lit(lit) => {
                            let val = expand_lit(lit, off, &curr_ctxt, &blocks, &mut tape)?;
                            tape.write(*val);
                        },
                        ast::Arg::Lbl(_) => {
                            tape.write(EMPTY_DEFAULT);
                        },
                    }
                }
            },
            ast::Stmt::Label(lbl) if !lbl.starts_with('.') => {
                let ctxt = blocks.get_key_value(lbl.as_str()).unwrap();
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
