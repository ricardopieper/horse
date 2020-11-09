use crate::runtime::*;
use std::collections::HashMap;

const AND_STR : &'static str = "__and__";
const OR_STR : &'static str = "__or__";
const XOR_STR : &'static str = "__xor__";
const NOT_STR : &'static str = "__not__";


fn create_and_method(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 1, params.params.len());
            
            let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);            
            match other_type_name {
                "bool" => {
                    let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);

                    let self_as_rust_boolean = if *self_data == 0 {false} else {true};
                    let other_as_rust_boolean = if *other_int == 0 {false} else {true};
                    let bool_result = self_as_rust_boolean && other_as_rust_boolean;
                    let result_as_int: i128 = if bool_result {1} else {0};

                    let result = interpreter.allocate_type_byname_raw("bool", Box::new(result_as_int));
                    return result;
                },
                _ => {
                    
                    if let Some(addr) = interpreter.call_method(params.params[0], "__bool__", vec![]) {
                        //call the method again, but the argument is another boolean
                        let bool_value_addr = interpreter.call_method(params.bound_pyobj.unwrap(), AND_STR, vec![addr]).unwrap();
                        let bool_result = interpreter.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                        interpreter.memory.deallocate(bool_value_addr);
                        interpreter.memory.deallocate(addr);
                        if *bool_result == 1 {
                            return params.params[0];
                        } else {
                            return params.bound_pyobj.unwrap()
                        }
                    }

                    if let Some(addr) = interpreter.call_method(params.params[0], "__len__", vec![]) {
                        //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
                        let bool_value_addr = interpreter.call_method(params.bound_pyobj.unwrap(), AND_STR, vec![addr]).unwrap();
                        let bool_result = interpreter.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                        interpreter.memory.deallocate(bool_value_addr);
                        interpreter.memory.deallocate(addr);
                        if *bool_result == 1 {
                            return params.params[0];
                        } else {
                            return params.bound_pyobj.unwrap()
                        }
                    }
                    
                    
                    return interpreter.special_values.not_implemented_value;
                }
            };
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some(AND_STR.to_string()));

    methods.insert(AND_STR.to_string(), func_addr);
}

fn create_or_method(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 1, params.params.len());
            
            let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);            
            match other_type_name {
                "bool" => {
                    let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);

                    let self_as_rust_boolean = if *self_data == 0 {false} else {true};
                    let other_as_rust_boolean = if *other_int == 0 {false} else {true};
                    let bool_result = self_as_rust_boolean || other_as_rust_boolean;
                    let result_as_int: i128 = if bool_result {1} else {0};

                    return interpreter.allocate_type_byname_raw("bool", Box::new(result_as_int));
                },
                _ => {
                    
                    if let Some(addr) = interpreter.call_method(params.params[0], "__bool__", vec![]) {
                        //call the method again, but the argument is another boolean
                        let bool_value_addr = interpreter.call_method(params.bound_pyobj.unwrap(), OR_STR, vec![addr]).unwrap();
                        let bool_result = interpreter.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                        interpreter.memory.deallocate(bool_value_addr);
                        interpreter.memory.deallocate(addr);
                        if *bool_result == 1 {
                            return params.bound_pyobj.unwrap();
                        } else {
                            return params.params[0];
                        }
                    }

                    if let Some(addr) = interpreter.call_method(params.params[0], "__len__", vec![]) {
                        //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
                        let bool_value_addr = interpreter.call_method(params.bound_pyobj.unwrap(), OR_STR, vec![addr]).unwrap();
                        let bool_result = interpreter.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                        interpreter.memory.deallocate(bool_value_addr);
                        interpreter.memory.deallocate(addr);
                        if *bool_result == 1 {
                            return params.bound_pyobj.unwrap();
                        } else {
                            return params.params[0];
                        }
                    }
                    
                    
                    return interpreter.special_values.not_implemented_value;
                }
            };
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some(OR_STR.to_string()));

    methods.insert(OR_STR.to_string(), func_addr);
}

fn create_xor_method(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 1, params.params.len());
            
            let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);            
            match other_type_name {
                "bool" => {
                    let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);

                    let self_as_rust_boolean = if *self_data == 0 {false} else {true};
                    let other_as_rust_boolean = if *other_int == 0 {false} else {true};
                    let bool_result = self_as_rust_boolean ^ other_as_rust_boolean;
                    let result_as_int: i128 = if bool_result {1} else {0};

                    return interpreter.allocate_type_byname_raw("bool", Box::new(result_as_int));
                },
                _ => {
                    
                    if let Some(addr) = interpreter.call_method(params.params[0], "__bool__", vec![]) {
                        //call the method again, but the argument is another boolean
                        let bool_value_addr = interpreter.call_method(params.bound_pyobj.unwrap(), XOR_STR, vec![addr]).unwrap();
                        let bool_result = interpreter.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                        interpreter.memory.deallocate(bool_value_addr);
                        interpreter.memory.deallocate(addr);
                        if *bool_result == 1 {
                            return params.bound_pyobj.unwrap();
                        } else {
                            return params.params[0];
                        }
                    }

                    if let Some(addr) = interpreter.call_method(params.params[0], "__len__", vec![]) {
                        //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
                        let bool_value_addr = interpreter.call_method(params.bound_pyobj.unwrap(), XOR_STR, vec![addr]).unwrap();
                        let bool_result = interpreter.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                        interpreter.memory.deallocate(bool_value_addr);
                        interpreter.memory.deallocate(addr);
                        if *bool_result == 1 {
                            return params.bound_pyobj.unwrap();
                        } else {
                            return params.params[0];
                        }
                    }
                    
                    
                    return interpreter.special_values.not_implemented_value;
                }
            };
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some(XOR_STR.to_string()));

    methods.insert(XOR_STR.to_string(), func_addr);
}

fn create_not_method(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            
            let self_data = *interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
        
            return interpreter.allocate_type_byname_raw("bool", Box::new(if self_data == 0 { 1 as i128} else { 0 as i128 }));
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some(NOT_STR.to_string()));

    methods.insert(NOT_STR.to_string(), func_addr);
}

fn create_to_str(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = *interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            if self_data == 0 {
                interpreter.allocate_type_byname_raw("str", Box::new(String::from("False")))
            } else {
                interpreter.allocate_type_byname_raw("str", Box::new(String::from("True")))
            }
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__str__".to_string()));
    methods.insert("__str__".to_string(), func_addr);
}

fn create_repr(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = *interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            if self_data == 0 {
                interpreter.allocate_type_byname_raw("str", Box::new(String::from("False")))
            } else {
                interpreter.allocate_type_byname_raw("str", Box::new(String::from("True")))
            }
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__repr__".to_string()));
    methods.insert("__repr__".to_string(), func_addr);
}



fn create_to_boolean(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |_interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            return params.bound_pyobj.unwrap();
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__bool__".to_string()));
    methods.insert("__bool__".to_string(), func_addr);
}

fn create_to_int(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            if *self_data == 0 {
                interpreter.allocate_type_byname_raw("int", Box::new(0 as i128))
            } else {
                interpreter.allocate_type_byname_raw("int", Box::new(1 as i128))
            }
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__int__".to_string()));
    methods.insert("__int__".to_string(), func_addr);
}


fn create_to_float(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            if *self_data == 0 {
                interpreter.allocate_type_byname_raw("float", Box::new(0.0 as f64))
            } else {
                interpreter.allocate_type_byname_raw("float", Box::new(1.0 as f64))
            }
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__float__".to_string()));
    methods.insert("__float__".to_string(), func_addr);
}

pub fn register_boolean_type(interpreter: &Interpreter) -> MemoryAddress {
    let mut methods = HashMap::new();

    create_and_method(interpreter, &mut methods);
    create_or_method(interpreter, &mut methods);
    create_xor_method(interpreter, &mut methods);
    create_not_method(interpreter, &mut methods);

    create_to_boolean(interpreter, &mut methods);
    create_to_str(interpreter, &mut methods);
    create_repr(interpreter, &mut methods);
    create_to_int(interpreter, &mut methods);
    create_to_float(interpreter, &mut methods);

    
    //bool inherits from int
    
    let int_supertype = interpreter.find_in_module(BUILTIN_MODULE, "int").expect("int type not found");
    return interpreter.create_type(BUILTIN_MODULE, "bool", methods, HashMap::new(), Some(int_supertype));
}

