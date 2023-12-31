use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expected<T>(pub Option<T>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unit {
    Min,
    Ms,
    S,
    Khz,
    Hz,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Primitive {
    Bool(bool),
    Float(f64),
    Int(i64),
    Quantity((f64, Unit)),
    Str(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Stmt {
    Skip,
    Expr(Box<Expr>),
    Let((Expected<Identifier>, Expected<Box<Expr>>)),
    Return(Option<Box<Expr>>),
    Play(Expected<Box<Expr>>),
    Item(Box<Item>),
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Param {
    pub ty: Option<Identifier>,
    pub name: Identifier,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct ParamList(pub Vec<Param>);

#[derive(Clone, PartialEq, PartialOrd)]
pub struct FnDecl {
    pub name: Identifier,
    pub params: ParamList,
    pub body: Box<Block>,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct AnonymousFn {
    pub params: ParamList,
    pub body: Box<Expr>,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Item {
    FnDecl(Box<FnDecl>),
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct CallExpr {
    pub id: Identifier,
    pub args: Vec<Expr>,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Expr {
    Prim(Primitive),
    Call(CallExpr),
    Var(Identifier),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
    Block(Box<Block>),
    AnonymousFn(Box<AnonymousFn>),

    Error,
}

#[derive(Clone, PartialEq, PartialOrd)]
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
            Primitive::Float(f) => Primitive::Quantity((f, unit)),
            Primitive::Int(d) => Primitive::Quantity((d as f64, unit)),
            Primitive::Quantity((f, _)) => Primitive::Quantity((f, unit)),
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

impl<T: Display> Display for Expected<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{}", value),
            None => write!(f, "<MISSING>"),
        }
    }
}

impl<T: Debug> Debug for Expected<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{:?}", value),
            None => write!(f, "<MISSING>"),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Expr::*;
        match self {
            Prim(val) => write!(f, "{}", val),
            Call(call) => write!(f, "{}", call),
            Var(id) => write!(f, "{}", id),
            Add(left, right) => write!(f, "{} + {}", left, right),
            Sub(left, right) => write!(f, "{} - {}", left, right),
            Mul(left, right) => write!(f, "{} * {}", left, right),
            Div(left, right) => write!(f, "{} / {}", left, right),
            Paren(expr) => write!(f, "({})", expr),
            Block(block) => write!(f, "{}", block),
            AnonymousFn(fun) => write!(f, "{}", fun),
            Error => write!(f, "<ERR>"),
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
            Add(left, right) => write!(f, "({:?} + {:?})", left, right),
            Sub(left, right) => write!(f, "({:?} - {:?})", left, right),
            Mul(left, right) => write!(f, "({:?} * {:?})", left, right),
            Div(left, right) => write!(f, "({:?} / {:?})", left, right),
            Paren(expr) => write!(f, "({:?})", expr),
            Block(block) => write!(f, "{:?}", block),
            AnonymousFn(fun) => write!(f, "{:?}", fun),
            Error => write!(f, "<ERR>"),
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
        write!(f, ")")?;

        Ok(())
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
        write!(f, ")")?;

        Ok(())
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
            Item(item) => write!(f, "{}", item),
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
            Item(item) => write!(f, "{:?}", item),
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
        write!(f, "{}", self.name)?;

        Ok(())
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "{} ", ty)?;
        }
        write!(f, "{}", self.name)?;

        Ok(())
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
        write!(f, "|{}| {:?}", self.params, self.body)?;

        Ok(())
    }
}

impl Display for AnonymousFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "|{}| {}", self.params, self.body)?;

        Ok(())
    }
}

impl Display for FnDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}({}) {}", self.name, self.params, self.body)?;

        Ok(())
    }
}

impl Debug for FnDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}({}) {:?}", self.name, self.params, self.body)?;

        Ok(())
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Item::*;
        match self {
            FnDecl(fun) => write!(f, "{}", fun),
        }
    }
}

impl Debug for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::Item::*;
        match self {
            FnDecl(fun) => write!(f, "{:?}", fun),
        }
    }
}
