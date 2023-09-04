use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::Range,
};

use crate::parse::{span_range, Span};

#[derive(Clone, PartialEq, Eq)]
pub struct SyntaxNode<T> {
    range: Option<Range<usize>>,
    pub node: Option<Box<T>>,
}

impl<T> SyntaxNode<T> {
    pub const MISSING: SyntaxNode<T> = SyntaxNode {
        range: None,
        node: None,
    };

    pub fn new(range: Option<Range<usize>>, node: Option<T>) -> Self {
        Self {
            range,
            node: node.map(Box::new),
        }
    }

    pub fn map<F, U>(self, f: F) -> SyntaxNode<U>
    where
        F: FnOnce(T) -> U,
    {
        SyntaxNode {
            range: self.range.clone(),
            node: self.node.map(|box x| f(x)).map(Box::new),
        }
    }

    pub fn range(&self) -> Option<Range<usize>> {
        self.range.clone()
    }

    // pub fn boxify(self) -> SyntaxNode<Box<T>> {
    //     SyntaxNode {
    //         range: self.range.clone(),
    //         node: match self.node {
    //             None => None,
    //             Some(node) => Some(Box::new(node)),
    //         },
    //     }
    // }
}

// pub trait Node {
//     // fn children(&self) -> Vec<dyn Syntax>
// }

pub fn cover_ranges(a: Option<Range<usize>>, b: Option<Range<usize>>) -> Option<Range<usize>> {
    match (a, b) {
        (None, None) => None,
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (Some(a), Some(b)) => Some(Range {
            start: a.start.min(b.start),
            end: a.end.max(b.end),
        }),
    }
}

impl<'a, T> From<(Option<Range<usize>>, T)> for SyntaxNode<T> {
    fn from((range, node): (Option<Range<usize>>, T)) -> Self {
        Self {
            range,
            node: Some(Box::new(node)),
        }
    }
}

impl<'a, T> From<(Span<'a>, T)> for SyntaxNode<T> {
    fn from((span, node): (Span<'a>, T)) -> Self {
        Self {
            range: Some(span_range(&span)),
            node: Some(Box::new(node)),
        }
    }
}

impl<T> From<T> for SyntaxNode<T> {
    fn from(node: T) -> Self {
        Self {
            range: None,
            node: Some(Box::new(node)),
        }
    }
}

impl<T: Display> Display for SyntaxNode<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.node {
            Some(node) => write!(f, "{}", node),
            None => write!(f, "<MISSING>"),
        }
    }
}

impl<T: Debug> Debug for SyntaxNode<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.node {
            Some(node) => write!(f, "{:?}", node),
            None => write!(f, "<MISSING>"),
        }
    }
}

// trait GetChildRanges {
//     fn child_ranges(&self) -> Vec<Range<usize>>;
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Min,
    Ms,
    S,
    Khz,
    Hz,
}

#[derive(Clone, PartialEq)]
pub enum Primitive {
    Bool(bool),
    Float(f64),
    Int(i64),
    Quantity((f64, SyntaxNode<Unit>)),
    Str(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(pub String);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, PartialEq)]
pub enum Stmt {
    Skip,
    Expr(SyntaxNode<Expr>),
    Let((SyntaxNode<Identifier>, SyntaxNode<Expr>)),
    Return(Option<SyntaxNode<Expr>>),
    Play(SyntaxNode<Expr>),
    Decl(SyntaxNode<Decl>),
}

#[derive(Clone, PartialEq)]
pub struct Param {
    pub ty: Option<SyntaxNode<Identifier>>,
    pub name: SyntaxNode<Identifier>,
}

#[derive(Clone, PartialEq)]
pub struct ParamList(pub Vec<SyntaxNode<Param>>);

#[derive(Clone, PartialEq)]
pub struct FnDecl {
    pub name: SyntaxNode<Identifier>,
    pub params: ParamList,
    pub body: SyntaxNode<Block>,
}

#[derive(Clone, PartialEq)]
pub struct AnonymousFn {
    pub params: ParamList,
    pub body: SyntaxNode<Expr>,
}

#[derive(Clone, PartialEq)]
pub enum Decl {
    FnDecl(SyntaxNode<FnDecl>),
}

#[derive(Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<SyntaxNode<Expr>>,
}

impl Block {
    pub fn as_stmts_vec(mut self) -> Vec<Stmt> {
        if let Some(expr) = self.expr.take() {
            self.stmts.push(Stmt::Expr(expr));
        }

        self.stmts
    }
}

#[derive(Clone, PartialEq)]
pub struct CallExpr {
    pub id: SyntaxNode<Identifier>,
    pub args: Vec<SyntaxNode<Expr>>,
}

#[derive(Clone, PartialEq)]
pub enum Expr {
    Prim(SyntaxNode<Primitive>),
    Call(CallExpr),
    Var(SyntaxNode<Identifier>),
    BinOp(SyntaxNode<Expr>, Op, SyntaxNode<Expr>),
    Paren(SyntaxNode<Expr>),
    Block(SyntaxNode<Block>),
    AnonymousFn(SyntaxNode<AnonymousFn>),
}

// impl GetChildRanges for Expr {
//     fn child_ranges(&self) -> Vec<Range<usize>> {
//         match self {
//             Expr::Prim(node) => std::iter::once(node.range()).filter_map(|n| n).collect(),
//             _ => vec![],
//         }
//     }
// }

#[derive(Clone, PartialEq)]
pub struct Document {
    pub stmts: Vec<Stmt>,
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Unit::*;
        match self {
            Min => write!(f, "min"),
            Ms => write!(f, "ms"),
            S => write!(f, "s"),
            Khz => write!(f, "khz"),
            Hz => write!(f, "hz"),
        }
    }
}

impl From<&str> for Unit {
    fn from(value: &str) -> Self {
        match value {
            "min" => Self::Min,
            "ms" => Self::Ms,
            "s" => Self::S,
            "khz" => Self::Khz,
            "hz" => Self::Hz,
            _ => panic!(),
        }
    }
}

impl Primitive {
    pub fn negate(self) -> Self {
        match self {
            Primitive::Bool(b) => Primitive::Bool(!b),
            Primitive::Float(f) => Primitive::Float(-f),
            Primitive::Int(d) => Primitive::Int(-d),
            Primitive::Quantity((f, unit)) => Primitive::Quantity((-f, unit)),
            _ => self,
        }
    }

    pub fn with_unit(self, unit: Unit) -> Self {
        match self {
            Primitive::Float(f) => Primitive::Quantity((f, SyntaxNode::from(unit))),
            Primitive::Int(d) => Primitive::Quantity((d as f64, SyntaxNode::from(unit))),
            Primitive::Quantity((f, _)) => Primitive::Quantity((f, SyntaxNode::from(unit))),
            _ => self,
        }
    }
}

impl Debug for Primitive {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Primitive::*;
        match self {
            Bool(val) => write!(f, "{val}"),
            Float(val) => write!(f, "{val}"),
            Int(val) => write!(f, "{val}"),
            Quantity((val, unit)) => write!(f, "{val}{unit}"),
            Str(val) => write!(f, "{val}"),
        }
    }
}

impl Display for Primitive {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let n = self.stmts.len();
        for (i, stmt) in self.stmts.iter().enumerate() {
            write!(f, "{:?}", stmt)?;
            if i + 1 < n {
                write!(f, "\n\n")?;
            }
        }

        Ok(())
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Expr::*;
        match self {
            Prim(val) => write!(f, "{}", val),
            Call(call) => write!(f, "{}", call),
            Var(id) => write!(f, "{}", id),
            BinOp(left, op, right) => write!(f, "{} {} {}", left, op, right),
            Paren(expr) => write!(f, "({})", expr),
            Block(block) => write!(f, "{}", block),
            AnonymousFn(fun) => write!(f, "{}", fun),
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Expr::*;
        match self {
            Prim(val) => write!(f, "{}", val),
            Call(call) => write!(f, "{}", call),
            Var(id) => write!(f, "{}", id),
            BinOp(left, op, right) => write!(f, "({:?} {} {:?})", left, op, right),
            Paren(expr) => write!(f, "({:?})", expr),
            Block(block) => write!(f, "{:?}", block),
            AnonymousFn(fun) => write!(f, "{:?}", fun),
        }
    }
}

impl Display for CallExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.id)?;
        let n = self.args.len();
        for (i, arg) in self.args.iter().enumerate() {
            write!(f, "{}", arg)?;
            if i + 1 < n {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl Debug for CallExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.id)?;
        let n = self.args.len();
        for (i, arg) in self.args.iter().enumerate() {
            write!(f, "{:?}", arg)?;
            if i + 1 < n {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Op::*;
        match self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Mul => write!(f, "*"),
            Div => write!(f, "/"),
        }
    }
}

impl Debug for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Op::*;
        match self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Mul => write!(f, "*"),
            Div => write!(f, "/"),
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Stmt::*;
        match self {
            Skip => write!(f, ";"),
            Expr(expr) => write!(f, "{};", expr),
            Let((id, expr)) => write!(f, "let {} = {};", id, expr),
            Return(expr) => match expr {
                Some(expr) => write!(f, "return {};", expr),
                None => write!(f, "return;"),
            },
            Play(expr) => write!(f, "play {};", expr),
            Decl(item) => write!(f, "{}", item),
        }
    }
}

impl Debug for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Stmt::*;
        match self {
            Skip => write!(f, ";"),
            Expr(expr) => write!(f, "{:?};", expr),
            Let((id, expr)) => write!(f, "let {} = {:?};", id, expr),
            Return(expr) => match expr {
                Some(expr) => write!(f, "return {:?};", expr),
                None => write!(f, "return;"),
            },
            Play(expr) => write!(f, "play {:?};", expr),
            Decl(item) => write!(f, "{:?}", item),
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for stmt in &self.stmts {
            write!(f, " {}", stmt)?;
        }
        if let Some(expr) = &self.expr {
            write!(f, " {}", expr)?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for stmt in &self.stmts {
            write!(f, " {:?}", stmt)?;
        }
        if let Some(expr) = &self.expr {
            write!(f, " {:?}", expr)?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl Debug for Param {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "{} ", ty)?;
        }
        write!(f, "{}", self.name)
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "{} ", ty)?;
        }
        write!(f, "{}", self.name)
    }
}

impl Debug for ParamList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let n = self.0.len();
        for (i, param) in self.0.iter().enumerate() {
            write!(f, "{}{}", param, if i + 1 == n { "" } else { ", " })?;
        }

        Ok(())
    }
}

impl Display for ParamList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let n = self.0.len();
        for (i, param) in self.0.iter().enumerate() {
            write!(f, "{}{}", param, if i + 1 == n { "" } else { ", " })?;
        }

        Ok(())
    }
}

impl Debug for AnonymousFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "|{}| {:?}", self.params, self.body)
    }
}

impl Display for AnonymousFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "|{}| {}", self.params, self.body)
    }
}

impl Display for FnDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}({}) {}", self.name, self.params, self.body)
    }
}

impl Debug for FnDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}({}) {:?}", self.name, self.params, self.body)
    }
}

impl Display for Decl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Decl::*;
        match self {
            FnDecl(fun) => write!(f, "{}", fun),
        }
    }
}

impl Debug for Decl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Decl::*;
        match self {
            FnDecl(fun) => write!(f, "{:?}", fun),
        }
    }
}
