use crate::float::Float;
use crate::memory::*;
use crate::runtime::*;
use crate::bytecode::program::CodeObject;
use std::fmt::Debug;
use std::collections::HashMap;

pub const BUILTIN_MODULE: &'static str = "__builtins__";
pub const MAIN_MODULE: &'static str = "__main__";



#[derive(Debug, Eq, PartialEq)]
pub enum BuiltInTypeData {
    Int(i128),
    Float(Float),
    String(String),
    List(Vec<MemoryAddress>)
}

impl ToString for BuiltInTypeData {
    fn to_string(&self) -> String {
        match self {
            BuiltInTypeData::Int(i) => i.to_string(),
            BuiltInTypeData::Float(i) => i.0.to_string(),
            BuiltInTypeData::String(i) => "String \"".to_owned() + i + "\"",
            BuiltInTypeData::List(_i) => {
                return "a list".into()
            }
        }
    }
}

impl BuiltInTypeData {
    pub fn take_float(&self) -> f64 {
        match self {
            BuiltInTypeData::Float(Float(f)) => *f,
            _ => panic!("Tried to transform something into float unexpectedly"),
        }
    }

    pub fn take_int(&self) -> i128 {
        match self {
            BuiltInTypeData::Int(i) => *i,
            _ => panic!("Tried to transform into int unexpectedly: {:?}", self),
        }
    }

    pub fn take_string(&self) -> &String {
        match self {
            BuiltInTypeData::String(s) => s,
            _ => panic!("Tried to transform something into string unexpectedly"),
        }
    }

    pub fn take_list(&self) -> &Vec<MemoryAddress> {
        match self {
            BuiltInTypeData::List(s) => s,
            _ => panic!("Tried to transform something into string unexpectedly"),
        }
    }

    pub fn take_list_mut(&mut self) -> &mut Vec<MemoryAddress> {
        match self {
            BuiltInTypeData::List(s) => s,
            _ => panic!("Tried to transform something into string unexpectedly"),
        }
    }
}
#[derive(Debug)]
pub struct CodeObjectContext {
    pub code: CodeObject,
    pub consts: Vec<MemoryAddress>
}
#[derive(Debug)]
pub struct ProgramContext {
    pub code_objects: Vec<CodeObjectContext>,
}

#[derive(Debug)]
pub enum PyObjectStructure {
    None,
    NotImplemented,
    Object {
        raw_data: BuiltInTypeData,
        refcount: usize,
    },
    NativeCallable {
        code: PyCallable,
        name: Option<String>,
    },
    UserDefinedFunction {
        name: String,
        code: CodeObjectContext
    },
    Type {
        name: String,
        methods: HashMap<String, MemoryAddress>,
        functions: HashMap<String, MemoryAddress>,
        supertype: Option<MemoryAddress>,
    },
    Module {
        name: String,
        objects: HashMap<String, MemoryAddress>,
    },
}

#[derive(Debug)]
pub struct PyObject {
    pub type_addr: MemoryAddress,
    pub structure: PyObjectStructure,
    pub properties: HashMap<String, MemoryAddress>,
    pub is_const: bool,
}

impl PyObject {
    pub fn try_get_builtin(&self) -> Option<&BuiltInTypeData> {
        match &self.structure {
            PyObjectStructure::Object {
                raw_data,
                refcount: _,
            } => {
                return Some(raw_data);
            }
            _ => {
                return None;
            }
        }
    }
}