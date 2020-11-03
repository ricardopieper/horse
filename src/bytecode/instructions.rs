use crate::runtime::*;


#[derive(Debug, Clone)]
pub enum Const{
    Integer(i128),
    Float(f64)
}

#[derive(Debug, Clone)]
pub enum Instruction {
    LoadConst(Const),
    LoadMethod(String),
    LoadFunction(String),
    CallMethod { number_arguments: usize },
    CallFunction { number_arguments: usize }
}


pub fn handle_method_call(interpreter: &Interpreter, number_args: usize) {
    let mut temp_stack = vec![];
    for _ in 0 .. number_args {
        temp_stack.push(interpreter.pop_stack());
    }
    temp_stack.reverse();

    let function_addr = interpreter.pop_stack();
    let bounded_obj_addr = interpreter.pop_stack();

    interpreter.new_stack_frame();

    let new_obj = interpreter.bounded_function_call_byaddr(
        bounded_obj_addr, 
        function_addr, 
        temp_stack);
    
    interpreter.pop_stack_frame();
    interpreter.push_stack(new_obj);
}

pub fn handle_function_call(interpreter: &Interpreter, number_args: usize) {
    let mut temp_stack = vec![];
    for _ in 0 .. number_args {
        temp_stack.push(interpreter.pop_stack());
    }
    temp_stack.reverse();

    let function_addr = interpreter.pop_stack();

    interpreter.new_stack_frame();

    let new_obj = interpreter.unbounded_function_call_byaddr(
        function_addr, 
        temp_stack);
    
    interpreter.pop_stack_frame();
    interpreter.push_stack(new_obj);
}

pub fn handle_load_const(interpreter: &Interpreter, const_data: &Const) {
    let loaded_addr = match const_data {
        Const::Integer(i) => {
            interpreter.allocate_type_byname_raw("int", Box::new(*i))
        },
        Const::Float(f) => {
            interpreter.allocate_type_byname_raw("float", Box::new(*f))
        }
    };

    interpreter.push_stack(loaded_addr);
}

pub fn handle_load_method(interpreter: &Interpreter, method_name: &str) {

    let stack_top = interpreter.pop_stack();
    let pyobj = interpreter.get_pyobj_byaddr(stack_top).unwrap();
    interpreter.push_stack(stack_top);
    
    let type_addr = pyobj.type_addr;
    let obj = interpreter.get_type_method_addr_byname(type_addr, method_name);

    match obj {
        None => panic!("type has no method {}", method_name),
        Some(addr) => {
            interpreter.push_stack(addr);
        }
    }
}

pub fn handle_load_function(interpreter: &Interpreter, method_name: &str) {
    let obj = interpreter.find_in_module(BUILTIN_MODULE, method_name);

    match obj {
        None => panic!("module has no object/function {}", method_name),
        Some(addr) => {
            interpreter.push_stack(addr);
        }
    }
}


pub fn execute_instructions(interpreter: &Interpreter, instructions: Vec<Instruction>) {
    
    #[cfg(test)] {
        println!("Executing instructions");
        for inst in instructions.iter() {
            println!("{:?}", inst);
        }
    }

    loop {
        if interpreter.get_pc() >= instructions.len() {
            return;
        }

        match instructions.get(interpreter.get_pc()).unwrap() {
            Instruction::CallMethod{ number_arguments} =>
                handle_method_call(interpreter, *number_arguments),
            Instruction::LoadConst(c) => 
                handle_load_const(interpreter, c),
            Instruction::LoadMethod(s) =>
                handle_load_method(interpreter, s.as_str()),
            Instruction::LoadFunction(s) =>
                handle_load_function(interpreter, s.as_str()),
            Instruction::CallFunction { number_arguments} =>
                handle_function_call(interpreter, *number_arguments),
        }
        interpreter.jump_pc(1);
    }
}