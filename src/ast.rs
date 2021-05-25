use pest::Span;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Op {
    Hlt = 0,
    Add = 1,
    Mul = 2,
    Cle = 3,
    Ceq = 4,
    Jmp = 5,
    Beq = 6,
    Cpy = 7,
    Put = 8,
    Ptn = 9,
}

impl Op {
    pub fn nargs(&self) -> usize {
        match self {
            Op::Add => 3,
            Op::Mul => 3,
            Op::Cle => 3,
            Op::Ceq => 3,
            Op::Beq => 2,
            Op::Cpy => 2,
            Op::Jmp => 1,
            Op::Put => 1,
            Op::Ptn => 1,
            Op::Hlt => 0,
        }
    }
}

pub struct Prog<'a> {
    pub stmts: Vec<Stmt<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    Label(Label<'a>),
    Inst(Inst<'a>),
    Lit(Lit<'a>),
}

pub type Label<'a> = Spanned<'a, String>;

#[derive(Debug, Clone)]
pub struct Inst<'a> {
    pub op: Op,
    pub args: Vec<Arg<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub enum Lit<'a> {
    Lbl(Label<'a>),
    Num(Spanned<'a, i32>),
    Str(Spanned<'a, String>),
    Chr(Spanned<'a, char>),
    Ref(Box<Lit<'a>>),
    // Actually not every lit can be inside `Deref`, only `Lbl`, `Deref` or `Ref`.
    // This is ensured in parsing though.
    Deref(Box<Lit<'a>>),
}

impl<'a> Lit<'a> {
    pub fn span(&self) -> Span<'a> {
        match self {
            Lit::Lbl(lbl) => lbl.span.clone(),
            Lit::Num(num) => num.span.clone(),
            Lit::Str(s)   => s.span.clone(),
            Lit::Chr(c)   => c.span.clone(),
            Lit::Ref(r)   => r.span(),
            Lit::Deref(d) => d.span(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Arg<'a> {
    Lit(Lit<'a>),
    Lbl(Label<'a>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Spanned<'a, T> {
    pub inner: T,
    pub span: Span<'a>,
}

impl<'a, T> Spanned<'a, T> {
    pub fn new(inner: T, span: Span<'a>) -> Spanned<'a, T> {
        Spanned { inner, span }
    }

    pub fn to_inner(self) -> T {
        self.inner
    }
}

impl<'a, T> std::ops::Deref for Spanned<'a, T> {
    type Target = T;
    fn deref(&self) -> &T { &self.inner }
}

impl<'a, T> std::ops::DerefMut for Spanned<'a, T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.inner }
}

impl<'a, T> std::convert::AsRef<T> for Spanned<'a, T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

pub fn mk_lbl(name: String, span: Span) -> Label {
    Spanned::new(name, span)
}

impl<'a> Inst<'a> {
    pub fn new(op: Op, args: Vec<Arg<'a>>, span: Span<'a>) -> Inst<'a> {
        Inst { op, args, span }
    }
}

use std::fmt;
use std::fmt::{ Display, Formatter };

impl<'a> Display for Stmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Label(lbl) => write!(f, "{}:", lbl.as_str()),
            Stmt::Inst(inst) => Display::fmt(inst, f),
            Stmt::Lit(lit)   => Display::fmt(lit, f),
        }
    }
}

impl<'a> Display for Inst<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.op)?;

        for arg in &self.args {
            write!(f, "{} ", arg)?;
        }

        Ok(())
    }
}

impl<'a> Display for Arg<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Arg::Lbl(lbl) => write!(f, "<{}>", lbl.as_str()),
            Arg::Lit(lit) => Display::fmt(lit, f),
        }
    }
}

impl<'a> Display for Lit<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Lit::Chr(chr) => write!(f, "'{}'", **chr),
            Lit::Str(s)   => write!(f, "\"{}\"", s.as_str()),
            Lit::Num(num) => write!(f, "{}", **num),
            Lit::Lbl(lbl) => write!(f, "'{}", lbl.as_str()),
            Lit::Ref(r)   => write!(f, "&{}", r),
            Lit::Deref(d) => write!(f, "*{}", d),
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Op::Hlt => "hlt",
            Op::Add => "add",
            Op::Mul => "mul",
            Op::Cle => "cle",
            Op::Ceq => "ceq",
            Op::Jmp => "jmp",
            Op::Beq => "beq",
            Op::Cpy => "cpy",
            Op::Put => "put",
            Op::Ptn => "ptn",
        };

        write!(f, "{}", name)
    }
}

impl<'a, T: Display> Display for Spanned<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}
