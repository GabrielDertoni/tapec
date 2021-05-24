use pest::Span;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Op {
    Hlt,
    Add,
    Mul,
    Cle,
    Ceq,
    Jmp,
    Beq,
    Cpy,
    Put,
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
}

pub type Label<'a> = Spanned<'a, &'a str>;

#[derive(Debug, Clone)]
pub struct Inst<'a> {
    pub op: Op,
    pub args: Vec<Arg<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub enum Arg<'a> {
    Lbl(Spanned<'a, &'a str>),
    Num(Spanned<'a, i32>),
    Str(Spanned<'a, &'a str>),
    Chr(Spanned<'a, char>),
    Lit(Box<Arg<'a>>),
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
}

impl<'a, T> std::ops::Deref for Spanned<'a, T> {
    type Target = T;
    fn deref(&self) -> &T { &self.inner }
}

impl<'a, T> std::ops::DerefMut for Spanned<'a, T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.inner }
}
