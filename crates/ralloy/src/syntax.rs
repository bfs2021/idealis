use crate::Kind;
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mult {
    Set,
    One,
    Lone,
    Some,
}
pub(crate) struct Signature {
    pub name: String,
    pub mult: Mult,
}
pub(crate) struct Field {
    pub name: String,
    pub owner: String,
    pub target: String,
    pub mult: Mult,
}
#[derive(Clone, Copy)]
pub(crate) enum Unary {
    Not,
    No,
    Some,
    One,
    Lone,
    Transpose,
    Closure,
    Reflexive,
}
#[derive(Clone, Copy)]
pub(crate) enum Binary {
    And,
    Or,
    Implies,
    Iff,
    Equal,
    NotEqual,
    In,
    NotIn,
    Union,
    Difference,
    Intersection,
    Product,
    Join,
    Domain,
    Range,
}
#[derive(Clone, Copy)]
pub(crate) enum Quantifier {
    All,
    No,
    Some,
    One,
    Lone,
}
#[derive(Clone)]
pub(crate) struct Declaration {
    pub names: Vec<String>,
    pub domain: Expr,
    pub disjoint: bool,
}
#[derive(Clone)]
pub(crate) enum Expr {
    Name(String),
    Call(String),
    Block(Vec<Self>),
    Unary(Unary, Box<Self>),
    Binary(Binary, Box<Self>, Box<Self>),
    Quantified(Quantifier, Vec<Declaration>, Box<Self>),
}
pub(crate) struct RawScope {
    pub overall: Option<usize>,
    pub entries: Vec<(String, usize, bool)>,
    pub bitwidth: usize,
}
pub(crate) struct RawCommand {
    pub label: String,
    pub kind: Kind,
    pub body: Expr,
    pub scope: RawScope,
}
#[derive(Default)]
pub(crate) struct Module {
    pub signatures: Vec<Signature>,
    pub fields: Vec<Field>,
    pub facts: Vec<Expr>,
    pub predicates: BTreeMap<String, Expr>,
    pub assertions: BTreeMap<String, Expr>,
    pub commands: Vec<RawCommand>,
}
