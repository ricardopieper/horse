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
    BinaryAdd,
    UnresolvedBreak
}

pub struct Program {
    //bytecode compatibility version
    //needs to recompile if bytecode has different version
    //bytecode depends on where things are at runtime, 
    //it's not very good right now
    pub version: u64,
    //constant values, bytecode refers to consts using indexes on data
    pub data: Vec<Const>,
    //the bytecode itself
    pub code: Vec<Instruction>
}