use crate::commons::float::Float;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Const {
    Integer(i128),
    Float(Float),
    Boolean(bool),
    String(String),
    CodeObject(CodeObject, String),
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Instruction {
    LoadConst(usize),
    LoadAttr(String),
    MakeFunction,
    MakeClass,
    StoreName(usize),
    StoreAttr(usize),
    LoadName(usize),
    LoadGlobal(usize),
    CallFunction { number_arguments: usize },
    JumpIfFalseAndPopStack(usize),
    JumpUnconditional(usize),
    ReturnValue,
    BinaryAdd,
    BinaryModulus,
    BinarySubtract,
    BinaryMultiply,
    BinaryTrueDivision,
    CompareLessEquals,
    CompareGreaterEquals,
    CompareGreaterThan,
    CompareLessThan,
    CompareEquals,
    CompareNotEquals,
    BuildList { number_elements: usize },
    IndexAccess,
    ForIter(usize),
    Raise,
    UnresolvedBreak,
    UnresolvedStoreAttr(String),
    UnresolvedStoreName(String),
    UnresolvedLoadName(String)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CodeObject {
    pub instructions: Vec<Instruction>,
    pub names: Vec<String>,
    pub params: Vec<String>,
    pub consts: Vec<Const>,
    pub main: bool
}

pub struct Program {
    //bytecode compatibility version
    //needs to recompile if bytecode has different version
    //bytecode depends on where things are at runtime,
    //it's not very good right now
    pub version: u64,
    pub code_objects: Vec<CodeObject>,
}
