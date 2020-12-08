use crate::bytecode::program::*;
use crate::runtime::*;

pub fn handle_method_call(runtime: &Runtime, number_args: usize) {
    let mut temp_stack = vec![];
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

pub fn handle_function_call(runtime: &Runtime, number_args: usize) {
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

pub fn handle_load_const(runtime: &Runtime, index: usize) {
    let memory_address = runtime.get_const(index);
    runtime.push_onto_stack(memory_address);
}

pub fn handle_store_const(runtime: &Runtime, const_data: &Const) {
    let loaded_addr = match const_data {
        Const::Integer(i) => runtime.allocate_type_byname_raw("int", Box::new(*i)),
        Const::Float(f) => runtime.allocate_type_byname_raw("float", Box::new(f.0)),
        Const::String(s) => runtime.allocate_type_byname_raw("str", Box::new(s.clone())),
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

pub fn handle_load_method(runtime: &Runtime, method_name: &str) {
    let stack_top = runtime.pop_stack();
    let pyobj = runtime.get_pyobj_byaddr(stack_top).unwrap();
    runtime.push_onto_stack(stack_top);

    let type_addr = pyobj.type_addr;
    let obj = runtime.get_type_method_addr_byname(type_addr, method_name);

    match obj {
        None => panic!("type has no method {}", method_name),
        Some(addr) => {
            runtime.push_onto_stack(addr);
        }
    }
}

pub fn handle_load_function(runtime: &Runtime, method_name: &str) {
    let obj = runtime.find_in_module(BUILTIN_MODULE, method_name);

    match obj {
        None => panic!("module has no object/function {}", method_name),
        Some(addr) => {
            let pyobj = runtime.get_pyobj_byaddr(addr).unwrap();
            match &pyobj.structure {
                PyObjectStructure::Callable { code: _, name: _ } => runtime.push_onto_stack(addr),
                PyObjectStructure::Type {
                    name: _,
                    bounded_functions: _,
                    unbounded_functions: functions,
                    supertype: _,
                } => {
                    let new = functions
                        .get("__new__")
                        .expect("Type has no __new__ function");
                    runtime.push_onto_stack(*new);
                }
                _ => panic!("not callable: {}", method_name),
            }
        }
    }
}

pub fn handle_load_name(runtime: &Runtime, name: &str) {
    let obj = runtime.get_local(name);

    match obj {
        None => panic!("No local with name {}", name),
        Some(addr) => {
            runtime.push_onto_stack(addr);
        }
    }
}

pub fn handle_store_name(runtime: &Runtime, name: &str) {
    if let Some(addr) = runtime.get_local(name) {
        runtime.decrease_refcount(addr);
    }
    let addr = runtime.pop_stack();
    runtime.increase_refcount(addr);
    runtime.bind_local(name, addr)
}

//returns true if jumped
pub fn handle_jump_if_false_pop(runtime: &Runtime, destination: usize) -> bool {
    let stack_top = runtime.pop_stack();
    let as_boolean = runtime.call_method(stack_top, "__bool__", &[]).unwrap();
    let raw_value: i128 = *runtime.get_raw_data_of_pyobj::<i128>(as_boolean);
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


pub fn execute_program(runtime: &Runtime, program: Program) {
    #[cfg(test)]
    {
        println!("Executing program");
        for (index, inst) in program.data.iter().enumerate() {
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

        match program.code.get(runtime.get_pc()).unwrap() {
            Instruction::CallMethod { number_arguments } => {
                handle_method_call(runtime, *number_arguments)
            }
            Instruction::LoadConst(c) => handle_load_const(runtime, *c),
            Instruction::LoadMethod(s) => handle_load_method(runtime, s.as_str()),
            Instruction::LoadFunction(s) => handle_load_function(runtime, s.as_str()),
            Instruction::CallFunction { number_arguments } => {
                handle_function_call(runtime, *number_arguments)
            }
            Instruction::LoadName(name) => handle_load_name(runtime, name.as_str()),
            Instruction::StoreName(name) => handle_store_name(runtime, name.as_str()),
            Instruction::JumpIfFalseAndPopStack(destination) => advance_pc = !handle_jump_if_false_pop(runtime, *destination),
            Instruction::JumpUnconditional(destination) =>{
                handle_jump_unconditional(runtime, *destination);
                advance_pc = false;
            },
            Instruction::UnresolvedBreak => {
                panic!("Unsupported instruction: UnresolvedBreak");
            }
        }
        if advance_pc {
            runtime.jump_pc(1);
        }
    }
}