use crate::bytecode::program::*;
use crate::float::Float;
use crate::runtime::*;

use smallvec::{smallvec, SmallVec};

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
        Const::Integer(i) => {
            runtime.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(*i))
        }
        Const::Float(f) => {
            runtime.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(*f))
        }
        Const::String(s) => {
            runtime.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(s.clone()))
        }
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

pub fn handle_load_method(runtime: &mut Runtime, method_name: &str) {
    let stack_top = runtime.top_stack();
    let pyobj = runtime.get_pyobj_byaddr(stack_top);

    let type_addr = pyobj.type_addr;
    let obj = runtime.get_method_addr_byname(type_addr, method_name);

    match obj {
        None => panic!("type has no method {}", method_name),
        Some(addr) => {
            runtime.push_onto_stack(addr);
        }
    }
}

pub fn handle_load_global(runtime: &mut Runtime, global_name: &str) {
    //for now the only globals are modules
    match runtime.find_module(global_name) {
        Some(addr) => {
            runtime.push_onto_stack(addr);
            return;
        }
        None => {
            panic!("could not find global named {}", global_name)
        }
    };
}

pub fn handle_load_attr(runtime: &mut Runtime, attr_name: &str) {
    let stack_top = runtime.top_stack();

    //first: attempt to load an object property
    match runtime.get_obj_property(stack_top, attr_name) {
        Some(addr) => {
            runtime.push_onto_stack(addr);
            return;
        }
        None => {}
    }
    //second: try to load a method name
    let pyobj = runtime.get_pyobj_byaddr(stack_top);
    let type_addr = pyobj.type_addr;

    let obj = runtime.get_method_addr_byname(type_addr, attr_name);
    match obj {
        None => {}
        Some(addr) => {
            runtime.push_onto_stack(addr);
            return;
        }
    }

    //third: try to load a module function, property, etc
    let obj = runtime.find_in_module_addr(stack_top, attr_name);

    match obj {
        None => panic!("could not find attribute named {}", attr_name),
        Some(addr) => {
            runtime.push_onto_stack(addr);
        }
    }
}

//optimization: if binary add, then we check the TOS and TOS-1. If both are numeric, then
//we just do the operation here and now, very fast, without creating a new stack frame.
//If both types are not numeric or not simple/common to be operated on, we just call __add__ on TOS-1 etc
macro_rules! create_binary_operator {
    ($method_name:tt, $param_a:tt, $param_b:tt, $operation:expr, $pycall:expr) => {
        fn $method_name(runtime: &mut Runtime) {
            let tos = runtime.pop_stack();
            let tos_1 = runtime.pop_stack();
            let pyobj_tos = runtime.get_pyobj_byaddr(tos);
            let pyobj_tos_1 = runtime.get_pyobj_byaddr(tos_1);
            let result;
            let mut refcount_tos: usize = 0;
            let mut refcount_tos_1: usize = 0;
            if let PyObjectStructure::Object {
                raw_data: raw_data_tos,
                refcount: r1,
            } = &pyobj_tos.structure
            {
                if let PyObjectStructure::Object {
                    raw_data: raw_data_tos_1,
                    refcount: r2,
                } = &pyobj_tos_1.structure
                {
                    refcount_tos = *r1;
                    refcount_tos_1 = *r2;
                    match raw_data_tos {
                        BuiltInTypeData::Int(j) => match raw_data_tos_1 {
                            BuiltInTypeData::Int(i) => {
                                let $param_a = i;
                                let $param_b = j;
                                result = Some(BuiltInTypeData::Int($operation))
                            }
                            BuiltInTypeData::Float(f) => {
                                let $param_a = f.0;
                                let $param_b = *j as f64;
                                result = Some(BuiltInTypeData::Float(Float($operation)))
                            }
                            _ => {
                                result = None;
                            }
                        },
                        BuiltInTypeData::Float(j) => match raw_data_tos_1 {
                            BuiltInTypeData::Int(i) => {
                                let $param_a = *i as f64;
                                let $param_b = j.0;
                                result = Some(BuiltInTypeData::Float(Float($operation)))
                            }
                            BuiltInTypeData::Float(f) => {
                                let $param_a = f.0;
                                let $param_b = j.0;
                                result = Some(BuiltInTypeData::Float(Float($operation)))
                            }
                            _ => {
                                result = None;
                            }
                        },
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
                //rebuild the stack to call method (optimization did not work)
                //TODO terrible
                runtime.push_onto_stack(tos_1);
                handle_load_method(runtime, $pycall);
                runtime.push_onto_stack(tos);
                handle_method_call(runtime, 1);
            } else {
                //:GarbageCollector
                if refcount_tos == 0 {
                    runtime.decrease_refcount(tos);
                }

                if refcount_tos_1 == 0 {
                    runtime.decrease_refcount(tos_1);
                }

                let addr = match result {
                    Some(i @ BuiltInTypeData::Int(_)) => {
                        runtime.allocate_type_byaddr_raw(runtime.builtin_type_addrs.int, i)
                    }
                    Some(f @ BuiltInTypeData::Float(_)) => {
                        runtime.allocate_type_byaddr_raw(runtime.builtin_type_addrs.float, f)
                    }
                    _ => {
                        panic!("unknown error")
                    }
                };
                runtime.push_onto_stack(addr);
            }
        }
    };
}
macro_rules! create_compare_operator {
    ($method_name:tt, $param_a:tt, $param_b:tt, $operation:expr, $pycall:expr) => {
        fn $method_name(runtime: &mut Runtime) {
            let tos = runtime.pop_stack();
            let tos_1 = runtime.pop_stack();

            let pyobj_tos = runtime.get_pyobj_byaddr(tos);
            let pyobj_tos_1 = runtime.get_pyobj_byaddr(tos_1);

            let result;
            let mut refcount_tos: usize = 0;
            let mut refcount_tos_1: usize = 0;
            if let PyObjectStructure::Object {
                raw_data: raw_data_tos,
                refcount: r1,
            } = &pyobj_tos.structure
            {
                if let PyObjectStructure::Object {
                    raw_data: raw_data_tos_1,
                    refcount: r2,
                } = &pyobj_tos_1.structure
                {
                    refcount_tos = *r1;
                    refcount_tos_1 = *r2;
                    match raw_data_tos {
                        BuiltInTypeData::Int(j) => {
                            match raw_data_tos_1 {
                                BuiltInTypeData::Int(i) => {
                                    let $param_a = i;
                                    let $param_b = j;
                                    // println!("result of operation {} {} {} is {}", $param_a, $pycall, $param_b, compare_result);
                                    result = Some($operation);
                                }
                                BuiltInTypeData::Float(f) => {
                                    let $param_a = f.0;
                                    let $param_b = *j as f64;
                                    result = Some($operation);
                                }
                                _ => {
                                    result = None;
                                }
                            }
                        }
                        BuiltInTypeData::Float(j) => match raw_data_tos_1 {
                            BuiltInTypeData::Int(i) => {
                                let $param_a = *i as f64;
                                let $param_b = j.0;
                                result = Some($operation);
                            }
                            BuiltInTypeData::Float(f) => {
                                let $param_a = f.0;
                                let $param_b = j.0;
                                result = Some($operation);
                            }
                            _ => {
                                result = None;
                            }
                        },
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
                //rebuild the stack to call method (optimization did not work)
                //TODO terrible
                runtime.push_onto_stack(tos_1);
                handle_load_method(runtime, $pycall);
                runtime.push_onto_stack(tos);
                handle_method_call(runtime, 1);
            } else {
                //:GarbageCollector @TODO Proper garbage collection, this is perhaps not the right thing to do.
                /*
                    Some extensive comparison expressions generate intermediate values which are allocated only temporarily.
                    They aren't bound to anything, they have no ownership.

                    This code tries to find these temporary allocations: If the unstacked values have 0 references to them,
                    it's a sign that they're unbounded/not owned by anyone/temporary. We deallocate them so that they don't accumulate.

                    Perhaps a proper garbage collector would rely on a tracing GC to find these allocations after some time has passed.
                    There is maybe a benefit of deallocating them all in a batch...?
                    Perhaps another option is to create a temporary stack, indicated by new opcodes...?

                    We don't have tracing/mark and sweep GC now so this will have to be sufficient for now.
                */

                if refcount_tos == 0 {
                    runtime.decrease_refcount(tos); //decrease_refcount currently also deallocates if reaches 0
                }

                if refcount_tos_1 == 0 {
                    runtime.decrease_refcount(tos_1);
                }

                if result.unwrap() {
                    runtime.push_onto_stack(runtime.builtin_type_addrs.true_val);
                } else {
                    runtime.push_onto_stack(runtime.builtin_type_addrs.false_val);
                }
            }
        }
    };
}

create_binary_operator!(handle_binary_add, a, b, a + b, "__add__");
create_binary_operator!(handle_binary_mod, a, b, a % b, "__mod__");
create_binary_operator!(handle_binary_sub, a, b, a - b, "__sub__");
create_binary_operator!(handle_binary_mul, a, b, a * b, "__mul__");

create_compare_operator!(handle_compare_greater, a, b, a > b, "__gt__");
create_compare_operator!(handle_compare_greater_eq, a, b, a > b, "__ge__");
create_compare_operator!(handle_compare_less, a, b, a < b, "__lt__");
create_compare_operator!(handle_compare_less_eq, a, b, a > b, "__le__");
create_compare_operator!(handle_compare_equals, a, b, a == b, "__eq__");
create_compare_operator!(handle_compare_not_eq, a, b, a != b, "__ne__");

//Division is weird so we do it separately. It always results in a float result
fn handle_binary_truediv(runtime: &mut Runtime) {
    let tos = runtime.pop_stack();
    let tos_1 = runtime.pop_stack();

    let pyobj_tos = runtime.get_pyobj_byaddr(tos);
    let pyobj_tos_1 = runtime.get_pyobj_byaddr(tos_1);

    let result: Option<f64>;
    let mut refcount_tos: usize = 0;
    let mut refcount_tos_1: usize = 0;
    if let PyObjectStructure::Object {
        raw_data: raw_data_tos,
        refcount: r1,
    } = &pyobj_tos.structure
    {
        if let PyObjectStructure::Object {
            raw_data: raw_data_tos_1,
            refcount: r2,
        } = &pyobj_tos_1.structure
        {
            refcount_tos = *r1;
            refcount_tos_1 = *r2;
            match raw_data_tos {
                BuiltInTypeData::Int(j) => match raw_data_tos_1 {
                    BuiltInTypeData::Int(i) => {
                        result = Some(*i as f64 / *j as f64);
                    }
                    BuiltInTypeData::Float(f) => {
                        result = Some((*f).0 / *j as f64);
                    }
                    _ => {
                        result = None;
                    }
                },
                BuiltInTypeData::Float(j) => match raw_data_tos_1 {
                    BuiltInTypeData::Int(i) => {
                        result = Some(*i as f64 / j.0 as f64);
                    }
                    BuiltInTypeData::Float(f) => {
                        result = Some((*f).0 / (*j).0);
                    }
                    _ => {
                        result = None;
                    }
                },
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
        //rebuild the stack to call method
        //TODO terrible
        runtime.push_onto_stack(tos_1);
        handle_load_method(runtime, "__truediv__");
        runtime.push_onto_stack(tos);
        handle_method_call(runtime, 1);
    } else {
        //:GarbageCollector @TODO Proper garbage collection, this is perhaps not the right thing to do.

        if refcount_tos == 0 {
            runtime.decrease_refcount(tos); //decrease_refcount currently also deallocates if reaches 0
        }
        if refcount_tos_1 == 0 {
            runtime.decrease_refcount(tos_1);
        }

        let addr = match result {
            Some(f) => runtime.allocate_type_byaddr_raw(
                runtime.builtin_type_addrs.float,
                BuiltInTypeData::Float(Float(f)),
            ),
            _ => {
                panic!("unknown error")
            }
        };
        runtime.push_onto_stack(addr);
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
    let raw_value = runtime.get_raw_data_of_pyobj(stack_top);

    if let BuiltInTypeData::Int(x) = raw_value {
        let result = if *x == 0 {
            runtime.set_pc(destination);
            true
        } else {
            false
        };
        runtime.decrease_refcount(stack_top);
        return result;
    } else {
        let as_boolean = runtime.call_method(stack_top, "__bool__", &[]).unwrap();
        let raw_value = runtime.get_raw_data_of_pyobj(as_boolean).take_int();
        let result = if raw_value == 0 {
            runtime.set_pc(destination);
            true
        } else {
            false
        };
        runtime.decrease_refcount(stack_top);
        return result;
    }
}

pub fn handle_build_list(runtime: &mut Runtime, size: usize) {
    let mut elements: Vec<MemoryAddress> = vec![];
    for _ in 0..size {
        elements.push(runtime.pop_stack());
    }
    elements.reverse();

    let built_list = runtime.allocate_type_byaddr_raw(
        runtime.builtin_type_addrs.list,
        BuiltInTypeData::List(elements),
    );

    runtime.push_onto_stack(built_list);
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
        //println!("Executing instruction {:?}", instruction);
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
            Instruction::LoadName(name) => handle_load_name(runtime, *name),

            Instruction::LoadGlobal(name) => handle_load_global(runtime, name),

            Instruction::LoadAttr(name) => handle_load_attr(runtime, name),
            Instruction::StoreName(name) => handle_store_name(runtime, *name),
            Instruction::BinaryAdd => handle_binary_add(runtime),
            Instruction::BinaryModulus => handle_binary_mod(runtime),
            Instruction::BinarySubtract => handle_binary_sub(runtime),
            Instruction::BinaryMultiply => handle_binary_mul(runtime),
            Instruction::CompareLessThan => handle_compare_less(runtime),
            Instruction::CompareLessEquals => handle_compare_less_eq(runtime),
            Instruction::CompareGreaterThan => handle_compare_greater(runtime),
            Instruction::CompareGreaterEquals => handle_compare_greater_eq(runtime),
            Instruction::CompareEquals => handle_compare_equals(runtime),
            Instruction::CompareNotEquals => handle_compare_not_eq(runtime),
            Instruction::BinaryTrueDivision => handle_binary_truediv(runtime),
            Instruction::JumpIfFalseAndPopStack(destination) => {
                advance_pc = !handle_jump_if_false_pop(runtime, *destination)
            }
            Instruction::BuildList { number_elements } => {
                handle_build_list(runtime, *number_elements)
            }
            Instruction::JumpUnconditional(destination) => {
                handle_jump_unconditional(runtime, *destination);
                advance_pc = false;
            }
            _ => {
                panic!("Unsupported instruction: {:?}", instruction);
            }
        }
        if advance_pc {
            runtime.jump_pc(1);
        }
    }
}
