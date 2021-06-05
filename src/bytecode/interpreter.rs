use crate::bytecode::program::*;
use crate::commons::float::Float;
use crate::runtime::runtime::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;

//use smallvec::{smallvec, SmallVec};


pub fn handle_function_call(runtime: &Runtime, number_args: usize) {
    let mut temp_stack = vec![];
    for _ in 0..number_args {
        temp_stack.push(runtime.pop_stack());
    }
    

    let function_addr = runtime.pop_stack();

    for addr in temp_stack.iter() {
        runtime.increase_refcount(*addr);
    }

    let returned_value = runtime.run_function(&mut temp_stack, function_addr, None);

    //increase refcount so it survives the pop_stack_frame call. Returned value should have 
    let refcount = runtime.get_refcount(returned_value);
    if refcount == 0{
        panic!("Refcount for addr {:p} is too low: Should be at least 1 to survive stack pop", returned_value);
    }

    runtime.increase_refcount(returned_value);

    for addr in temp_stack.iter() {
        runtime.decrease_refcount(*addr);
    }

    runtime.pop_stack_frame();

    runtime.push_onto_stack(returned_value);
}

pub fn handle_load_const(runtime: &Runtime, code: &CodeObjectContext, index: usize) {
    let memory_address = code.consts[index];
    runtime.push_onto_stack(memory_address);
}

fn get_const_memaddr(runtime: &Runtime, const_data: &Const) -> MemoryAddress {
    let const_addr = match const_data {
        Const::Integer(i) => {
            runtime.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(*i))
        }
        Const::Float(f) => {
            runtime.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(*f))
        }
        Const::String(s) => {
            runtime.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(s.clone()))
        }
        Const::CodeObject(codeobj) => {
            runtime.allocate_builtin_type_byname_raw("code object", BuiltInTypeData::CodeObject(
                register_codeobj_consts(runtime, codeobj)))
        }
        Const::Boolean(b) => {
            if *b {
                runtime.builtin_type_addrs.true_val
            } else {
                runtime.builtin_type_addrs.false_val
            }
        }
        Const::None => {
            runtime.special_values.get(&SpecialValue::NoneValue).unwrap().clone()
        }
    };
    runtime.make_const(const_addr);
    return const_addr;
}


pub fn curry_self(runtime: &Runtime, function: MemoryAddress, self_object: MemoryAddress) -> MemoryAddress {
    runtime.allocate_and_write(PyObject {
        type_addr: runtime.special_values[&SpecialValue::CallableType],
        properties: std::collections::HashMap::new(),
        is_const: false /*binding is temporary and can be deleted*/,
        structure: PyObjectStructure::BoundMethod {
            function_address: function,
            bound_address: self_object
        }
    })
}

pub fn handle_load_attr(runtime: &Runtime, attr_name: &str) {
    let stack_top = runtime.top_stack();

    //if the object is a class instance, we need to check whether 
    //the loaded property is a function.
    //if so, then we need to push a new value on the stack, 
    //which will be a bounded function to the stack_top value
    //i.e. it will be passed to the function as the "self" parameter 

    let pyobj = runtime.get_pyobj_byaddr(stack_top);

    if let PyObjectStructure::Object {raw_data, .. } = &pyobj.structure {
        if let BuiltInTypeData::ClassInstance = &raw_data {

            //ok, so this is a class instance
            //try getting the method from the type
            
            let type_addr = pyobj.type_addr;

            //find method, also checks if it even is a method at attr_name

            let method_addr = runtime.get_method_addr_byname(type_addr, attr_name);

            if let Some(m_addr) = method_addr {
                //create bound method
                let bounded = curry_self(runtime, m_addr, stack_top);
                runtime.increase_refcount(bounded);
                runtime.push_onto_stack(bounded);
                return;
            }
        }
    }

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

pub fn handle_load_global(runtime: &Runtime, code_obj: &CodeObjectContext, name: usize) {
    if let Some(addr) = runtime.builtin_names.get(name).map(|addr| *addr) {
        if addr != *runtime.special_values.get(&SpecialValue::NoneValue).unwrap() {
            runtime.push_onto_stack(addr); 
            return;
        }
    }

    if let Some(name_str) = code_obj.code.names.get(name) {
        if let Some(addr) = runtime.find_in_module(BUILTIN_MODULE, name_str) {
            runtime.push_onto_stack(addr); 
            return;
        } else if let Some(addr) = runtime.find_in_module(MAIN_MODULE, name_str) {
            runtime.push_onto_stack(addr); 
            return;
        }
    }
    
    match runtime.find_module(&code_obj.code.names[name]) {
        Some(addr) => {
            runtime.push_onto_stack(addr);
            return;
        }
        None => {
            panic!("could not find global named {}", &code_obj.code.names[name])
        }
    };
}



//optimization: if binary add, then we check the TOS and TOS-1. If both are numeric, then
//we just do the operation here and now, very fast, without creating a new stack frame.
//If both types are not numeric or not simple/common to be operated on, we just call __add__ on TOS-1 etc
macro_rules! create_binary_operator {
    ($method_name:tt, $param_a:tt, $param_b:tt, $operation:expr, $pycall:expr) => {
        fn $method_name(runtime: &Runtime) {
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
                handle_load_attr(runtime, $pycall);
                runtime.push_onto_stack(tos);
                handle_function_call(runtime, 1);
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
        fn $method_name(runtime: &Runtime) {
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
                handle_load_attr(runtime, $pycall);
                runtime.push_onto_stack(tos);
                handle_function_call(runtime, 1);
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
fn handle_binary_truediv(runtime: &Runtime) {
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
        handle_load_attr(runtime, "__truediv__");
        runtime.push_onto_stack(tos);
        handle_function_call(runtime, 1);
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

pub fn handle_load_name(runtime: &Runtime, code_obj: &CodeObjectContext, name: usize) {
    match runtime.get_local(name) {
        Some(addr) => runtime.push_onto_stack(addr),
        None => match runtime.builtin_names.get(name).map(|addr| *addr) {
            Some(addr) => runtime.push_onto_stack(addr),
            None => match code_obj.code.names.get(name) {
                //@TODO shouldn't it load from the main module first? Or even better, the current module being executed?
                Some(name_str) => match runtime.find_in_module(BUILTIN_MODULE, name_str) {
                    Some(addr) => runtime.push_onto_stack(addr),
                    None => match runtime.find_in_module(MAIN_MODULE, name_str) {
                        Some(addr) => runtime.push_onto_stack(addr),
                        None => panic!("Could not find name {}", name_str),
                    }
                },
                None => panic!("Could not find name")
            },
        }
    }
}

pub fn handle_store_name(runtime: &Runtime, name: usize) {
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

pub fn handle_build_list(runtime: &Runtime, size: usize) {
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

pub fn execute_next_instruction(runtime: &Runtime, code: &CodeObjectContext) {
    let mut advance_pc = true;
    let instruction = code.code.instructions.get(runtime.get_pc()).unwrap();
    //println!(">> {:?} at {:?}", instruction, code.code.objname);
    match instruction {
        Instruction::LoadConst(c) => handle_load_const(runtime, code, *c),
        Instruction::CallFunction { number_arguments } => handle_function_call(runtime, *number_arguments),
        Instruction::LoadName(name) => handle_load_name(runtime, code, *name),
        Instruction::LoadGlobal(name) => handle_load_global(runtime, code, *name),
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
        Instruction::MakeFunction => {
            let name_addr = runtime.pop_stack();
            let codeobj_addr = runtime.pop_stack();

            let qualname = runtime.get_pyobj_byaddr(name_addr).try_get_builtin().unwrap().take_string().clone();
            let codeobj = runtime.get_pyobj_byaddr(codeobj_addr).try_get_builtin().unwrap().take_code_object().clone();

            let function_addr = runtime.allocate_user_defined_function(codeobj, qualname.clone());
            runtime.add_to_module(MAIN_MODULE, qualname.as_str(), function_addr);
            runtime.push_onto_stack(function_addr);
        }
        Instruction::MakeClass => {
            let name_addr = runtime.pop_stack();
            let codeobj_addr = runtime.pop_stack();

            let class_name = runtime.get_pyobj_byaddr(name_addr).try_get_builtin().unwrap().take_string().clone();

            let class_code = runtime.get_pyobj_byaddr(codeobj_addr).try_get_builtin().unwrap().take_code_object().clone();
                    
            runtime.new_stack_frame(class_name.clone());
            //execute the class code
            execute_code_object(runtime, &class_code);

            let mut namespace = std::collections::HashMap::<String, MemoryAddress>::new();
            
            //and observe what changed in the current stack frame namespace 
            let namespace_values = runtime.get_local_namespace();
            for (index, name) in class_code.code.names.iter().enumerate() {
                //Insert the value as-is in the namespace
                namespace.insert(name.clone(), namespace_values[index]);
            }

            runtime.pop_stack_frame();

            let type_addr = runtime.create_type(MAIN_MODULE, &class_name.clone(), None);

            //Registers the regular functions on the type, even those that take the self parameter
            //They will be accessed using `ClassName.function_name`
            for (key, value) in namespace.iter() {
                println!("Registering method addr {} on type {}", key, class_name);
                runtime.register_method_addr_on_type(type_addr, key, *value);
            }

            runtime.register_type_unbounded_func(type_addr, "__new__", move |method_runtime: &Runtime, call_params: CallParams| -> MemoryAddress {
                let instance = method_runtime.allocate_type_byaddr_raw(type_addr, BuiltInTypeData::ClassInstance);
                method_runtime.increase_refcount(instance);
                method_runtime.increase_refcount(instance);
                
                //run the __init__ method if it exists
                match method_runtime.get_method_addr_byname(type_addr, "__init__") {
                    Some(init_addr) => {
                        let pyobj_method = method_runtime.get_pyobj_byaddr(init_addr);
                
                        match &pyobj_method.structure {
                            PyObjectStructure::UserDefinedFunction {code, ..} => {

                                method_runtime.new_stack_frame(code.code.objname.clone());
                                method_runtime.bind_local(0, instance);

                                for (index, item) in call_params.params.iter().enumerate() {
                                    method_runtime.bind_local(index + 1, *item);
                                }

                                execute_code_object(method_runtime, code);
                                method_runtime.pop_stack_frame();
                            },
                            _ => panic!("Unexpected")
                        }
                    },
                    None => {
                        panic!("Method __init__ does not exist on object {:?}", unsafe {&*type_addr})
                    }
                };
                
                return instance;
            });



            //  runtime.register_bounded_func_on_addr(type_addr, "__new__", move |method_runtime: &Runtime, _params: CallParams| -> MemoryAddress {
            //      method_runtime.allocate_type_byaddr_raw(type_addr, BuiltInTypeData::ClassInstance)
            //  });

            runtime.push_onto_stack(type_addr);


                
            
        }
        Instruction::ReturnValue => {
            let top = runtime.top_stack();
            //increase counter because it is being used by the current function
            runtime.increase_refcount(top);
            let instructions_len = code.code.instructions.len();
            runtime.set_pc(instructions_len);
        }
        Instruction::StoreAttr(attr_name) => {
            let obj = runtime.pop_stack();
            let value = runtime.pop_stack();
            let name = &code.code.names[*attr_name];
            runtime.set_attribute(obj, name.clone(), value)
        }
        Instruction::IndexAccess => {
            let index_value = runtime.pop_stack();
            let indexed_value = runtime.pop_stack();

            //for now we only accept integer indexing on lists and strings
            let index_int = runtime.get_raw_data_of_pyobj(index_value).take_int();
            let list = runtime.get_raw_data_of_pyobj(indexed_value).take_list();

            runtime.push_onto_stack(list[index_int as usize]);
        }
        Instruction::Raise => {
            let exception_value = runtime.pop_stack();
            runtime.raise_exception(exception_value);
        }
        _ => {
            panic!("Unsupported instruction: {:?}", instruction);
        }
    }
    if advance_pc {
        runtime.jump_pc(1);
    }
}

pub fn execute_code_object(runtime: &Runtime, code: &CodeObjectContext) {
    loop {
        if runtime.get_pc() >= code.code.instructions.len() {
            return;
        }
       
        execute_next_instruction(runtime, &code);
    }
}


fn register_codeobj_consts(runtime: &Runtime, codeobj: &CodeObject) -> CodeObjectContext {
    let mut consts = vec![];
    for c in codeobj.consts.iter() {
        let memaddr = get_const_memaddr(runtime, c);
        consts.push(memaddr);
    }
    CodeObjectContext{
        code: codeobj.clone(), 
        consts: consts
    }
}

#[allow(dead_code)]
//#[cfg(test)]
fn print_codeobj(codeobj: &CodeObject, codeobj_name: Option<String>) {
    for inst in codeobj.consts.iter() {
        if let Const::CodeObject(obj) = inst {
            print_codeobj(obj, Some(obj.objname.clone()));
        }
    }

    println!("\nInstructions of code object {:?}", codeobj_name);
    for (index, inst) in codeobj.instructions.iter().enumerate() {
        if let Instruction::LoadConst(n) = inst {
            let constval =  &codeobj.consts[*n];
            if let Const::CodeObject(obj) = constval {
                println!("{} - {:?} => code object {}", index, inst, obj.objname);
            } else {
                println!("{} - {:?} => constval = {:?}", index, inst, constval);
            }
        } 
        else if let Instruction::LoadGlobal(n) = inst {
            println!("{} - {:?} => global name = {:?}", index, inst, &codeobj.names[*n]);
        }
        else if let Instruction::LoadName(n) = inst {
            println!("{} - {:?} => name = {:?}", index, inst, &codeobj.names[*n]);
        }
        else if let Instruction::StoreName(n) = inst {
            println!("{} - {:?} => name = {:?}", index, inst, &codeobj.names[*n]);
        }
        else if let Instruction::StoreAttr(n) = inst {
            println!("{} - {:?} => name = {:?}", index, inst, &codeobj.names[*n]);
        }
        else {
            println!("{} - {:?}", index, inst);
        }
    }
}

pub fn execute_program(runtime: &mut Runtime, program: Program) {
    //print_codeobj(&program.code_objects[0], None);

    let main_code = program.code_objects.iter().find(|x| x.main).unwrap();
    let main_codeobj_ctx = register_codeobj_consts(runtime, main_code);
     
    execute_code_object(runtime, &main_codeobj_ctx);
}