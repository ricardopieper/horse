use crate::bytecode::program::*;
use crate::commons::float::Float;
use crate::runtime::vm::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;

use smallvec::{smallvec, SmallVec};


pub fn handle_function_call(vm: &VM, number_args: usize) {
    let mut temp_stack:SmallVec<[MemoryAddress; 4]>= smallvec![];
    for _ in 0..number_args {
        temp_stack.push(vm.pop_stack());
    }

    let function_addr = vm.pop_stack();

    for addr in temp_stack.iter() {
        vm.increase_refcount(*addr);
    }

    let (returned_value, popped_frame) = vm.run_function(PositionalParameters::from_stack_popped(&temp_stack), function_addr, None);

    //increase refcount so it survives the pop_stack_frame call.
    let refcount = vm.get_refcount(returned_value);
    if refcount == 0{
        panic!("Refcount for addr {:p} is too low: Should be at least 1 to survive stack pop", returned_value);
    }

    vm.increase_refcount(returned_value);

    for addr in temp_stack.iter() {
        vm.decrease_refcount(*addr);
    }
  
    if let Some(exception) = popped_frame.exception {
        vm.raise_exception(exception);
    }
  
    vm.push_onto_stack(returned_value);
}

pub fn handle_load_const(vm: &VM, code: &CodeObjectContext, index: usize) {
    let memory_address = code.consts[index];
    vm.push_onto_stack(memory_address);
}

fn get_const_memaddr(vm: &VM, const_data: &Const) -> MemoryAddress {
    let const_addr = match const_data {
        Const::Integer(i) => {
            vm.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(*i))
        }
        Const::Float(f) => {
            vm.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(*f))
        }
        Const::String(s) => {
            vm.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(s.clone()))
        }
        Const::CodeObject(codeobj) => {
            vm.allocate_builtin_type_byname_raw("code object", BuiltInTypeData::CodeObject(
                register_codeobj_consts(vm, codeobj)))
        }
        Const::Boolean(b) => {
            if *b {
                vm.builtin_type_addrs.true_val
            } else {
                vm.builtin_type_addrs.false_val
            }
        }
        Const::None => {
            vm.special_values.get(&SpecialValue::NoneValue).unwrap().clone()
        }
    };
    vm.make_const(const_addr);
    return const_addr;
}


pub fn curry_self(vm: &VM, function: MemoryAddress, self_object: MemoryAddress) -> MemoryAddress {
    vm.allocate_and_write(PyObject {
        type_addr: vm.special_values[&SpecialValue::CallableType],
        properties: std::collections::BTreeMap::new(),
        is_const: false /*binding is temporary and can be deleted*/,
        structure: PyObjectStructure::BoundMethod {
            function_address: function,
            bound_address: self_object
        }
    })
}

pub fn handle_load_attr(vm: &VM, attr_name: &str) {
    let stack_top = vm.pop_stack();

    //if the object is a class instance, we need to check whether 
    //the loaded property is a function.
    //if so, then we need to push a new value on the stack, 
    //which will be a bounded function to the stack_top value
    //i.e. it will be passed to the function as the "self" parameter 
    let pyobj = vm.get_pyobj_byaddr(stack_top);
    //println!("Stack top value: {:?}", pyobj);

    if let PyObjectStructure::Object {raw_data, .. } = &pyobj.structure {
        if let BuiltInTypeData::ClassInstance = &raw_data {

            //ok, so this is a class instance
            //try getting the method from the type
            
            let type_addr = pyobj.type_addr;

            //find method, also checks if it even is a method at attr_name

            let method_addr = vm.get_method_addr_byname(type_addr, attr_name);

            if let Some(m_addr) = method_addr {
                //create bound method
                let bounded = curry_self(vm, m_addr, stack_top);
                vm.increase_refcount(bounded);
                vm.push_onto_stack(bounded);
                return;
            }
        }
    }

    //first: attempt to load an object property
    match vm.get_obj_property(stack_top, attr_name) {
        Some(addr) => {
            vm.push_onto_stack(addr);
            return;
        }
        None => {}
    }
    //second: try to load a method name

    let type_addr = pyobj.type_addr;

    let obj = vm.get_method_addr_byname(type_addr, attr_name);
    match obj {
        None => {}
        Some(addr) => {
            let bounded = curry_self(vm, addr, stack_top);
            vm.increase_refcount(bounded);
            vm.push_onto_stack(bounded);
            return;
        }
    }

    //third: try to load a module function, property, etc
    let obj = vm.find_in_module_addr(stack_top, attr_name);

    match obj {
        None => panic!("could not find attribute named {}", attr_name),
        Some(addr) => {
            vm.push_onto_stack(addr);
        }
    }
}

pub fn handle_load_global(vm: &VM, code_obj: &CodeObjectContext, name: usize) {
    /*
    if let Some(addr) = vm.builtin_names.get(name).map(|addr| *addr) {
        if addr != *vm.special_values.get(&SpecialValue::NoneValue).unwrap() {
            vm.push_onto_stack(addr); 
            return;
        }
    }
    */

    if let Some(name_str) = code_obj.code.names.get(name) {
        if let Some(addr) = vm.find_in_module(BUILTIN_MODULE, name_str) {
            vm.push_onto_stack(addr); 
            return;
        } else if let Some(addr) = vm.find_in_module(MAIN_MODULE, name_str) {
            vm.push_onto_stack(addr); 
            return;
        }
    }
    
    match vm.find_module(&code_obj.code.names[name]) {
        Some(addr) => {
            vm.push_onto_stack(addr);
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
        fn $method_name(vm: &VM) {
            let tos = vm.pop_stack();
            let tos_1 = vm.pop_stack();
            let pyobj_tos = vm.get_pyobj_byaddr(tos);
            let pyobj_tos_1 = vm.get_pyobj_byaddr(tos_1);
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
                vm.push_onto_stack(tos_1);
                handle_load_attr(vm, $pycall);
                vm.push_onto_stack(tos);
                handle_function_call(vm, 1);
            } else {
                //:GarbageCollector
                if refcount_tos == 0 {
                    vm.decrease_refcount(tos);
                }

                if refcount_tos_1 == 0 {
                    vm.decrease_refcount(tos_1);
                }
                let addr = match result {
                    Some(i @ BuiltInTypeData::Int(_)) => {
                        vm.allocate_type_byaddr_raw(vm.builtin_type_addrs.int, i)
                    }
                    Some(f @ BuiltInTypeData::Float(_)) => {
                        vm.allocate_type_byaddr_raw(vm.builtin_type_addrs.float, f)
                    }
                    _ => {
                        panic!("unknown error")
                    }
                };

                vm.push_onto_stack(addr);
            }
        }
    };
}
macro_rules! create_compare_operator {
    ($method_name:tt, $param_a:tt, $param_b:tt, $operation:expr, $pycall:expr) => {
        fn $method_name(vm: &VM) {
            let tos = vm.pop_stack();
            let tos_1 = vm.pop_stack();

            let pyobj_tos = vm.get_pyobj_byaddr(tos);
            let pyobj_tos_1 = vm.get_pyobj_byaddr(tos_1);

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
                vm.push_onto_stack(tos_1);
                handle_load_attr(vm, $pycall);
                vm.push_onto_stack(tos);
                handle_function_call(vm, 1);
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
                    vm.decrease_refcount(tos); //decrease_refcount currently also deallocates if reaches 0
                }

                if refcount_tos_1 == 0 {
                    vm.decrease_refcount(tos_1);
                }

                if result.unwrap() {
                    vm.push_onto_stack(vm.builtin_type_addrs.true_val);
                } else {
                    vm.push_onto_stack(vm.builtin_type_addrs.false_val);
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
create_compare_operator!(handle_compare_greater_eq, a, b, a >= b, "__ge__");
create_compare_operator!(handle_compare_less, a, b, a < b, "__lt__");
create_compare_operator!(handle_compare_less_eq, a, b, a <= b, "__le__");
create_compare_operator!(handle_compare_equals, a, b, a == b, "__eq__");
create_compare_operator!(handle_compare_not_eq, a, b, a != b, "__ne__");

//Division is weird so we do it separately. It always results in a float result
fn handle_binary_truediv(vm: &VM) {
    let tos = vm.pop_stack();
    let tos_1 = vm.pop_stack();

    let pyobj_tos = vm.get_pyobj_byaddr(tos);
    let pyobj_tos_1 = vm.get_pyobj_byaddr(tos_1);

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
        vm.push_onto_stack(tos_1);
        handle_load_attr(vm, "__truediv__");
        vm.push_onto_stack(tos);
        handle_function_call(vm, 1);
    } else {
        //:GarbageCollector @TODO Proper garbage collection, this is perhaps not the right thing to do.

        if refcount_tos == 0 {
            vm.decrease_refcount(tos); //decrease_refcount currently also deallocates if reaches 0
        }
        if refcount_tos_1 == 0 {
            vm.decrease_refcount(tos_1);
        }

        let addr = match result {
            Some(f) => vm.allocate_type_byaddr_raw(
                vm.builtin_type_addrs.float,
                BuiltInTypeData::Float(Float(f)),
            ),
            _ => {
                panic!("unknown error")
            }
        };
        vm.push_onto_stack(addr);
    }
}

pub fn handle_load_name(vm: &VM, code_obj: &CodeObjectContext, name: usize) {
    
    match vm.get_local(name) {
        Some(addr) => vm.push_onto_stack(addr),
        None => match code_obj.code.names.get(name) {
            //@TODO shouldn't it load from the main module first? Or even better, the current module being executed?
            Some(name_str) => match vm.find_in_module(BUILTIN_MODULE, name_str) {
                Some(addr) => vm.push_onto_stack(addr),
                None => match vm.find_in_module(MAIN_MODULE, name_str) {
                    Some(addr) => vm.push_onto_stack(addr),
                    None => panic!("Could not find name {}", name_str),
                }
            },
            None => panic!("Could not find name")
        },
    }
    
}

pub fn handle_store_name(vm: &VM, name: usize) {
    if let Some(addr) = vm.get_local(name) {
        vm.decrease_refcount(addr);
    }
    let addr = vm.pop_stack();
    vm.increase_refcount(addr);
    vm.bind_local(name, addr)
}

//returns true if jumped
pub fn handle_jump_if_false_pop(vm: &VM, destination: usize) -> bool {
    let stack_top = vm.pop_stack();
    let raw_value = vm.get_raw_data_of_pyobj(stack_top);

    if let BuiltInTypeData::Int(x) = raw_value {
        let result = if *x == 0 {
            vm.set_pc(destination);
            true
        } else {
            false
        };
        vm.decrease_refcount(stack_top);
        return result;
    } else {
        let (as_boolean, _) = vm.call_method(stack_top, "__bool__", PositionalParameters::empty()).unwrap();
        let raw_value = vm.get_raw_data_of_pyobj(as_boolean).take_int();
        let result = if raw_value == 0 {
            vm.set_pc(destination);
            true
        } else {
            false
        };
        vm.decrease_refcount(stack_top);
        return result;
    }
}

pub fn handle_build_list(vm: &VM, size: usize) {
    let mut elements: Vec<MemoryAddress> = vec![];
    for _ in 0..size {
        elements.push(vm.pop_stack());
    }
    elements.reverse();

    //This bypasses the list.__new__ function
    let built_list = vm.allocate_type_byaddr_raw(
        vm.builtin_type_addrs.list,
        BuiltInTypeData::List(elements),
    );

    vm.push_onto_stack(built_list);
}

pub fn handle_jump_unconditional(vm: &VM, destination: usize) {
    vm.set_pc(destination);
}

pub fn handle_store_attr(vm: &VM, code: &CodeObjectContext, attr_name: usize) {
    let obj = vm.pop_stack();
    let value = vm.pop_stack();
    let name = &code.code.names[attr_name];
    vm.set_attribute(obj, name, value);
    vm.increase_refcount(obj);
    vm.increase_refcount(value);
}

pub fn execute_next_instruction(vm: &VM, code: &CodeObjectContext) {
    let mut advance_pc = true;
    let instruction = code.code.instructions.get(vm.get_pc()).unwrap();
    //println!(">> {:?} {:?} at {:?}", vm.get_pc(), instruction, code.code.objname);
    //vm.print_stack();
    match instruction {
        Instruction::LoadConst(c) => handle_load_const(vm, code, *c),
        Instruction::CallFunction { number_arguments } => handle_function_call(vm, *number_arguments),
        Instruction::LoadName(name) => handle_load_name(vm, code, *name),
        Instruction::LoadGlobal(name) => handle_load_global(vm, code, *name),
        Instruction::LoadAttr(name) => handle_load_attr(vm, name),
        Instruction::StoreName(name) => handle_store_name(vm, *name),
        Instruction::BinaryAdd => handle_binary_add(vm),
        Instruction::BinaryModulus => handle_binary_mod(vm),
        Instruction::BinarySubtract => handle_binary_sub(vm),
        Instruction::BinaryMultiply => handle_binary_mul(vm),
        Instruction::CompareLessThan => handle_compare_less(vm),
        Instruction::CompareLessEquals => handle_compare_less_eq(vm),
        Instruction::CompareGreaterThan => handle_compare_greater(vm),
        Instruction::CompareGreaterEquals => handle_compare_greater_eq(vm),
        Instruction::CompareEquals => handle_compare_equals(vm),
        Instruction::CompareNotEquals => handle_compare_not_eq(vm),
        Instruction::BinaryTrueDivision => handle_binary_truediv(vm),
        Instruction::JumpIfFalseAndPopStack(destination) => {
            advance_pc = !handle_jump_if_false_pop(vm, *destination)
        }
        Instruction::BuildList { number_elements } => {
            handle_build_list(vm, *number_elements)
        }
        Instruction::JumpUnconditional(destination) => {
            handle_jump_unconditional(vm, *destination);
            advance_pc = false;
        }
        Instruction::MakeFunction(has_default_params) => {
            let name_addr = vm.pop_stack();
            let codeobj_addr = vm.pop_stack();

            let qualname = vm.get_pyobj_byaddr(name_addr).try_get_builtin().unwrap().take_string().clone();
            let codeobj = vm.get_pyobj_byaddr(codeobj_addr).try_get_builtin().unwrap().take_code_object().clone();
            


            let function_addr = if *has_default_params {
                //When there are default params, we will do something sneaky
                //the previous instructions will actually build a list of positional arguments
                //allowing the function to be called passing a variable numer of arguments.

                //So yes, the top stack parameter is a list of memory addresses, a python list,
                //containing the positional parameters.

                //We'll have to calculate how many of the default parameters we will use 
                //at the time the function is called.

                //@TODO remember to add a check in the parser: default arguments must be the last parameters of the function.

                let default_params = vm.pop_stack();
                let as_list = vm.get_raw_data_of_pyobj(default_params).take_list();

                vm.allocate_user_defined_function(codeobj, qualname.clone(), as_list.to_vec())
            } else {
                vm.allocate_user_defined_function(codeobj, qualname.clone(), vec![])
            };
            vm.add_to_module(MAIN_MODULE, qualname.as_str(), function_addr);
            vm.push_onto_stack(function_addr);
        }
        Instruction::MakeClass => {
            let name_addr = vm.pop_stack();
            let codeobj_addr = vm.pop_stack();

            let class_name = vm.get_pyobj_byaddr(name_addr).try_get_builtin().unwrap().take_string().clone();

            let class_code = vm.get_pyobj_byaddr(codeobj_addr).try_get_builtin().unwrap().take_code_object().clone();
                    
            vm.new_stack_frame(&class_name);
            
            //execute the class code
            execute_code_object(vm, &class_code);

            let popped_stack_frame = vm.pop_stack_frame();

            let mut namespace = std::collections::BTreeMap::<String, MemoryAddress>::new();
            
            //and observe what changed in the current stack frame namespace 
            let namespace_values = popped_stack_frame.local_namespace;
            for (index, name) in class_code.code.names.iter().enumerate() {
                //Insert the value as-is in the namespace
                namespace.insert(name.clone(), namespace_values[index]);
            }

            let type_addr = vm.create_type(MAIN_MODULE, &class_name.clone(), None);

            //Registers the regular functions on the type, even those that take the self parameter
            //They will be accessed using `ClassName.function_name`
            for (key, value) in namespace.iter() {
                //println!("Registering method addr {} on type {}", key, class_name);
                vm.register_method_addr_on_type(type_addr, key, *value);
            }

            vm.register_type_unbounded_func(type_addr, "__new__", move |method_vm: &VM, call_params: CallParams| -> MemoryAddress {
                let instance = method_vm.allocate_type_byaddr_raw(type_addr, BuiltInTypeData::ClassInstance);
                method_vm.increase_refcount(instance);
                method_vm.increase_refcount(instance);

                method_vm.call_method(instance, "__init__", call_params.params);
                
                return instance;
            });

            vm.push_onto_stack(type_addr);
        }
        Instruction::PopTop => {
            vm.pop_stack();
        }
        Instruction::ReturnValue => {
            let top = vm.top_stack();
            //increase counter because it is being used by the current function
            vm.increase_refcount(top);
            let instructions_len = code.code.instructions.len();
            vm.set_pc(instructions_len);
        }
        Instruction::StoreAttr(attr_name) => {
            handle_store_attr(vm, code, *attr_name);
        }
        Instruction::IndexAccess => {
            let index_value = vm.pop_stack();
            let indexed_value = vm.pop_stack();

            //for now we only accept integer indexing on lists and strings
            let index_int = vm.get_raw_data_of_pyobj(index_value).take_int();
            let list = vm.get_raw_data_of_pyobj(indexed_value).take_list();

            vm.push_onto_stack(list[index_int as usize]);
        }
        Instruction::Raise => {
            let exception_value = vm.pop_stack();
            vm.raise_exception(exception_value);
        }
        Instruction::ForIter(end_ptr) => {
            //TOS is the iterator object
            let iterator = vm.top_stack();

            //this assumes the iterator is on the top of the call already
            let (next, popped_frame) = vm.call_method(iterator, "__next__", PositionalParameters::empty()).unwrap();
            
            //This effectivelly catches the exception. This is weird in python: why 
            //use an ***exception*** to stop iteration? Makes no sense!
            if let Some(exception_addr) = popped_frame.exception {
                if exception_addr == vm.special_values[&SpecialValue::StopIterationType] {
                    vm.set_pc(*end_ptr);
                    advance_pc = false;
                }
            } else {
                vm.push_onto_stack(next);   
            }

            //unimplemented!();
        }
        _ => {
            panic!("Unsupported instruction: {:?}", instruction);
        }
    }
    
    if let Some(_) = vm.get_current_exception() {
        //if an exception happened, then finish execution immediately, push None on stack
        vm.push_onto_stack(vm.special_values[&SpecialValue::NoneValue]);
        let instructions_len = code.code.instructions.len();
        vm.set_pc(instructions_len);
        advance_pc = false;
    }


    //println!(">> {:?} Executed", vm.get_pc());
    if advance_pc {
        vm.jump_pc(1);
    }
}

pub fn execute_code_object(vm: &VM, code: &CodeObjectContext) {
    loop {
        if vm.get_pc() >= code.code.instructions.len() {
            return;
        }
       
        execute_next_instruction(vm, &code);
    }
}


fn register_codeobj_consts(vm: &VM, codeobj: &CodeObject) -> CodeObjectContext {
    let mut consts = vec![];
    for c in codeobj.consts.iter() {
        let memaddr = get_const_memaddr(vm, c);
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

pub fn execute_program(vm: &mut VM, program: Program) {
    //print_codeobj(&program.code_objects[0], None);

    let main_code = program.code_objects.iter().find(|x| x.main).unwrap();
    let main_codeobj_ctx = register_codeobj_consts(vm, main_code);
     
    execute_code_object(vm, &main_codeobj_ctx);
}