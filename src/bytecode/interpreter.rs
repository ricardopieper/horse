use crate::bytecode::program::*;
use crate::runtime::*;
use crate::float::Float;

use smallvec::{SmallVec, smallvec};

pub fn handle_method_call(runtime: &mut Runtime, number_args: usize) {
    let mut temp_stack: SmallVec<[MemoryAddress; 2]> = smallvec![];
    for _ in 0..number_args {
        temp_stack.push(runtime.pop_stack());
    }
    temp_stack.reverse();

    let function_addr = runtime.pop_stack();
    let bounded_obj_addr = runtime.pop_stack();

    runtime.increase_refcount(bounded_obj_addr);
    for addr in temp_stack.iter() {
        runtime.increase_refcount(*addr);
    }

    runtime.new_stack_frame();

    let new_obj =
        runtime.bounded_function_call_byaddr(bounded_obj_addr, function_addr, &temp_stack);

    runtime.decrease_refcount(bounded_obj_addr);
    for addr in temp_stack.iter() {
        runtime.decrease_refcount(*addr);
    }

    runtime.pop_stack_frame();
    runtime.push_onto_stack(new_obj);
}

pub fn handle_function_call(runtime: &mut Runtime, number_args: usize) {
    let mut temp_stack = vec![];
    for _ in 0..number_args {
        temp_stack.push(runtime.pop_stack());
    }
    temp_stack.reverse();

    let function_addr = runtime.pop_stack();

    for addr in temp_stack.iter() {
        runtime.increase_refcount(*addr);
    }

    runtime.new_stack_frame();

    let new_obj = runtime.unbounded_function_call_byaddr(function_addr, &temp_stack);

    for addr in temp_stack.iter() {
        runtime.decrease_refcount(*addr);
    }

    runtime.pop_stack_frame();
    runtime.push_onto_stack(new_obj);
}

pub fn handle_load_const(runtime: &mut Runtime, index: usize) {
    let memory_address = runtime.get_const(index);
    runtime.push_onto_stack(memory_address);
}

pub fn handle_store_const(runtime: &mut Runtime, const_data: &Const) {
    let loaded_addr = match const_data {
        Const::Integer(i) => runtime.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(*i)),
        Const::Float(f) => runtime.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(*f)),
        Const::String(s) => runtime.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(s.clone())),
        Const::Boolean(b) => {
            if *b {
                runtime.builtin_type_addrs.true_val
            } else {
                runtime.builtin_type_addrs.false_val
            }
        }
    };

    runtime.store_const(loaded_addr);
}

//optimization: if binary add, then we check the TOS and TOS-1. If both are numeric, then 
//we just do the operation here and now, very fast, without creating a new stack frame.
//If both types are not numeric or not simple/common to be operated on, we just call __add__ on TOS-1 etc
pub fn handle_binary_add(runtime: &mut Runtime) {
    let tos = runtime.top_stack();
    let tos_1 = runtime.get_stack_offset(-1);

    let pyobj_tos = runtime.get_pyobj_byaddr(tos);
    let pyobj_tos_1 = runtime.get_pyobj_byaddr(tos_1);

    let result;
    
    if let PyObjectStructure::Object{ raw_data: raw_data_tos, refcount: _} = &pyobj_tos.structure {
        if let PyObjectStructure::Object{ raw_data: raw_data_tos_1, refcount: _} = &pyobj_tos_1.structure {
            match raw_data_tos {
                BuiltInTypeData::Int(j) => {
                    match raw_data_tos_1 {
                        BuiltInTypeData::Int(i) => {
                            result = Some(BuiltInTypeData::Int(j + i))
                        }
                        BuiltInTypeData::Float(f) => {
                            result = Some(BuiltInTypeData::Float(Float(*j as f64 + f.0)))
                        }
                       _ => {
                            result = None;
                        }
                    }
                }
                BuiltInTypeData::Float(j) => {
                    match raw_data_tos_1 {
                        BuiltInTypeData::Int(i) => {
                            result = Some(BuiltInTypeData::Float(Float(j.0 + *i as f64)))
                        }
                        BuiltInTypeData::Float(f) => {
                            result = Some(BuiltInTypeData::Float(Float( j.0 as f64 + f.0)))
                        }
                        _ => {
                            result = None;
                        }
                    }
                }
                _ => {
                    result = None;
                }
            }
        } else {
            result = None;
        }
    } else {
        result = None;
    }

    if result.is_none() {
        //TODO terrible
        runtime.pop_stack();
        runtime.pop_stack();
        runtime.push_onto_stack(tos_1);
        handle_load_method(runtime, "__add__");
        runtime.push_onto_stack(tos);
        handle_method_call(runtime, 1);
    }
    else {
        let type_addr = match &result.as_ref().unwrap() {
            BuiltInTypeData::Int(_) => runtime.builtin_type_addrs.int,
            BuiltInTypeData::Float(_) => runtime.builtin_type_addrs.float,
            _ => { panic!("unknown error") }
        };

        let addr = runtime.allocate_type_byaddr_raw(type_addr, result.unwrap());
        runtime.push_onto_stack(addr);
    }
}

pub fn handle_load_method(runtime: &mut Runtime, method_name: &str) {
    let stack_top = runtime.top_stack();
    let pyobj = runtime.get_pyobj_byaddr(stack_top);

    let type_addr = pyobj.type_addr;
    let obj = runtime.get_type_method_addr_byname(type_addr, method_name);

    match obj {
        None => panic!("type has no method {}", method_name),
        Some(addr) => {
            runtime.push_onto_stack(addr);
        }
    }
}

pub fn handle_load_function(runtime: &mut Runtime, method_name: &str) {
    let obj = runtime.find_in_module(BUILTIN_MODULE, method_name);

    match obj {
        None => panic!("module has no object/function {}", method_name),
        Some(addr) => {
            let pyobj = runtime.get_pyobj_byaddr(addr);
            let addr = match &pyobj.structure {
                PyObjectStructure::NativeCallable { code: _, name: _ } => addr,
                PyObjectStructure::Type {
                    name: _,
                    methods: _,
                    functions,
                    supertype: _,
                } => {
                    let new = functions
                        .get("__new__")
                        .expect("Type has no __new__ function");
                    *new
                }
                _ => panic!("not callable: {}", method_name),
            };
            runtime.push_onto_stack(addr);
        }
    }
}

pub fn handle_load_name(runtime: &mut Runtime, name: usize) {
    let obj = runtime.get_local(name);

    match obj {
        None => panic!("No local with name {}", name),
        Some(addr) => {
            runtime.push_onto_stack(addr);
        }
    }
}

pub fn handle_store_name(runtime: &mut Runtime, name: usize) {
    if let Some(addr) = runtime.get_local(name) {
        runtime.decrease_refcount(addr);
    }
    let addr = runtime.pop_stack();
    runtime.increase_refcount(addr);
    runtime.bind_local(name, addr)
}

//returns true if jumped
pub fn handle_jump_if_false_pop(runtime: &mut Runtime, destination: usize) -> bool {
    let stack_top = runtime.pop_stack();
    let as_boolean = runtime.call_method(stack_top, "__bool__", &[]).unwrap();
    let raw_value = runtime.get_raw_data_of_pyobj(as_boolean).take_int();
    let result = if raw_value == 0 {
        runtime.set_pc(destination);
        true
    } else {
        false
    };

    runtime.decrease_refcount(as_boolean);

    return result;
}

pub fn handle_jump_unconditional(runtime: &Runtime, destination: usize) {
    runtime.set_pc(destination);
}


pub fn execute_program(runtime: &mut Runtime, program: Program) {
    #[cfg(test)]
    {
        println!("Executing program");
        for (index, inst) in program.code.iter().enumerate() {
            println!("{} - {:?}", index, inst);
        }
    }

    for constval in program.data.iter() {
        handle_store_const(runtime, constval);
    }


    loop {
        if runtime.get_pc() >= program.code.len() {
            return;
        }

        let mut advance_pc = true;
        let instruction = program.code.get(runtime.get_pc()).unwrap();
        match instruction {
            Instruction::CallMethod { number_arguments } => {
                handle_method_call(runtime, *number_arguments)
            }
            Instruction::LoadConst(c) => handle_load_const(runtime, *c),
            Instruction::LoadMethod(s) => handle_load_method(runtime, s.as_str()),
            Instruction::LoadFunction(s) => handle_load_function(runtime, s.as_str()),
            Instruction::CallFunction { number_arguments } => {
                handle_function_call(runtime, *number_arguments)
            }
            Instruction::LoadName(name) 
                => handle_load_name(runtime, *name),
            Instruction::StoreName(name) 
                => handle_store_name(runtime, *name),
            Instruction::BinaryAdd => handle_binary_add(runtime),
                Instruction::JumpIfFalseAndPopStack(destination) => advance_pc = !handle_jump_if_false_pop(runtime, *destination),
            Instruction::JumpUnconditional(destination) =>{
                handle_jump_unconditional(runtime, *destination);
                advance_pc = false;
            },
            _ => {
                panic!("Unsupported instruction: {:?}", instruction);
            }
        }
        if advance_pc {
            runtime.jump_pc(1);
        }
    }
}