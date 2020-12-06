use crate::runtime::*;

#[derive(Debug, Clone)]
pub enum Const {
    Integer(i128),
    Float(f64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Instruction {
    LoadConst(Const),
    LoadMethod(String),
    LoadFunction(String),
    StoreName(String),
    LoadName(String),
    CallMethod { number_arguments: usize },
    CallFunction { number_arguments: usize },
    JumpIfFalseAndPopStack(usize),
    JumpUnconditional(usize),
    UnresolvedBreak
}

pub fn handle_method_call(interpreter: &Interpreter, number_args: usize) {
    let mut temp_stack = vec![];
    for _ in 0..number_args {
        temp_stack.push(interpreter.pop_stack());
    }
    temp_stack.reverse();

    let function_addr = interpreter.pop_stack();
    let bounded_obj_addr = interpreter.pop_stack();

    interpreter.increase_refcount(bounded_obj_addr);
    for addr in temp_stack.iter() {
        interpreter.increase_refcount(*addr);
    }

    interpreter.new_stack_frame();

    let new_obj =
        interpreter.bounded_function_call_byaddr(bounded_obj_addr, function_addr, &temp_stack);

    interpreter.decrease_refcount(bounded_obj_addr);
    for addr in temp_stack.iter() {
        interpreter.decrease_refcount(*addr);
    }

    interpreter.pop_stack_frame();
    interpreter.push_onto_stack(new_obj);
}

pub fn handle_function_call(interpreter: &Interpreter, number_args: usize) {
    let mut temp_stack = vec![];
    for _ in 0..number_args {
        temp_stack.push(interpreter.pop_stack());
    }
    temp_stack.reverse();

    let function_addr = interpreter.pop_stack();

    for addr in temp_stack.iter() {
        interpreter.increase_refcount(*addr);
    }

    interpreter.new_stack_frame();

    let new_obj = interpreter.unbounded_function_call_byaddr(function_addr, &temp_stack);

    for addr in temp_stack.iter() {
        interpreter.decrease_refcount(*addr);
    }

    interpreter.pop_stack_frame();
    interpreter.push_onto_stack(new_obj);
}

pub fn handle_load_const(interpreter: &Interpreter, const_data: &Const) {
    let loaded_addr = match const_data {
        Const::Integer(i) => interpreter.allocate_type_byname_raw("int", Box::new(*i)),
        Const::Float(f) => interpreter.allocate_type_byname_raw("float", Box::new(*f)),
        Const::String(s) => interpreter.allocate_type_byname_raw("str", Box::new(s.clone())),
        Const::Boolean(b) => {
            if *b {
                interpreter.special_values[&SpecialValue::TrueValue]
            } else {
                interpreter.special_values[&SpecialValue::FalseValue]
            }
        }
    };

    interpreter.push_onto_stack(loaded_addr);
}

pub fn handle_load_method(interpreter: &Interpreter, method_name: &str) {
    let stack_top = interpreter.pop_stack();
    let pyobj = interpreter.get_pyobj_byaddr(stack_top).unwrap();
    interpreter.push_onto_stack(stack_top);

    let type_addr = pyobj.type_addr;
    let obj = interpreter.get_type_method_addr_byname(type_addr, method_name);

    match obj {
        None => panic!("type has no method {}", method_name),
        Some(addr) => {
            interpreter.push_onto_stack(addr);
        }
    }
}

pub fn handle_load_function(interpreter: &Interpreter, method_name: &str) {
    let obj = interpreter.find_in_module(BUILTIN_MODULE, method_name);

    match obj {
        None => panic!("module has no object/function {}", method_name),
        Some(addr) => {
            let pyobj = interpreter.get_pyobj_byaddr(addr).unwrap();
            match &pyobj.structure {
                PyObjectStructure::Callable { code: _, name: _ } => interpreter.push_onto_stack(addr),
                PyObjectStructure::Type {
                    name: _,
                    bounded_functions: _,
                    unbounded_functions: functions,
                    supertype: _,
                } => {
                    let new = functions
                        .get("__new__")
                        .expect("Type has no __new__ function");
                    interpreter.push_onto_stack(*new);
                }
                _ => panic!("not callable: {}", method_name),
            }
        }
    }
}

pub fn handle_load_name(interpreter: &Interpreter, name: &str) {
    let obj = interpreter.get_local(name);

    match obj {
        None => panic!("No local with name {}", name),
        Some(addr) => {
            interpreter.push_onto_stack(addr);
        }
    }
}

pub fn handle_store_name(interpreter: &Interpreter, name: &str) {
    if let Some(addr) = interpreter.get_local(name) {
        interpreter.decrease_refcount(addr);
    }
    let addr = interpreter.pop_stack();
    interpreter.increase_refcount(addr);
    interpreter.bind_local(name, addr)
}

//returns true if jumped
pub fn handle_jump_if_false_pop(interpreter: &Interpreter, destination: usize) -> bool {
    let stack_top = interpreter.pop_stack();
    let as_boolean = interpreter.call_method(stack_top, "__bool__", vec![]).unwrap();
    let raw_value: i128 = *interpreter.get_raw_data_of_pyobj::<i128>(as_boolean);
    let result = if raw_value == 0 {
        interpreter.set_pc(destination);
        true
    } else {
        false
    };

    interpreter.decrease_refcount(as_boolean);

    return result;
}

pub fn handle_jump_unconditional(interpreter: &Interpreter, destination: usize) {
    interpreter.set_pc(destination);
}


pub fn execute_instructions(interpreter: &Interpreter, instructions: Vec<Instruction>) {
    #[cfg(test)]
    {
        println!("Executing instructions");
        for (index, inst) in instructions.iter().enumerate() {
            println!("{} - {:?}", index, inst);
        }
    }

    loop {
        if interpreter.get_pc() >= instructions.len() {
            return;
        }

        let mut advance_pc = true;

        match instructions.get(interpreter.get_pc()).unwrap() {
            Instruction::CallMethod { number_arguments } => {
                handle_method_call(interpreter, *number_arguments)
            }
            Instruction::LoadConst(c) => handle_load_const(interpreter, c),
            Instruction::LoadMethod(s) => handle_load_method(interpreter, s.as_str()),
            Instruction::LoadFunction(s) => handle_load_function(interpreter, s.as_str()),
            Instruction::CallFunction { number_arguments } => {
                handle_function_call(interpreter, *number_arguments)
            }
            Instruction::LoadName(name) => handle_load_name(interpreter, name.as_str()),
            Instruction::StoreName(name) => handle_store_name(interpreter, name.as_str()),
            Instruction::JumpIfFalseAndPopStack(destination) => advance_pc = !handle_jump_if_false_pop(interpreter, *destination),
            Instruction::JumpUnconditional(destination) =>{
                handle_jump_unconditional(interpreter, *destination);
                advance_pc = false;
            },
            Instruction::UnresolvedBreak => {
                panic!("Unsupported instruction: UnresolvedBreak");
            }
        }
        if advance_pc {
            interpreter.jump_pc(1);
        }
    }
}
