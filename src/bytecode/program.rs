use crate::float::Float;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Const {
    Integer(i128),
    Float(Float),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Instruction {
    LoadConst(usize),
    LoadMethod(String),
    LoadFunction(String),
    UnresolvedStoreName(String),
    UnresolvedLoadName(String),
    StoreName(usize),
    LoadName(usize),
    CallMethod { number_arguments: usize },
    CallFunction { number_arguments: usize },
    JumpIfFalseAndPopStack(usize),
    JumpUnconditional(usize),
    UnresolvedBreak
}

pub struct Program {
    pub data: Vec<Const>,
    pub code: Vec<Instruction>
}