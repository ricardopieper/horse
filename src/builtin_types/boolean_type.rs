use crate::runtime::*;

const AND_STR : &'static str = "__and__";
const OR_STR : &'static str = "__or__";
const XOR_STR : &'static str = "__xor__";


fn and_method(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
    
    let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);            
    match other_type_name {
        "bool" => {
            let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);

            let self_as_rust_boolean = if *self_data == 0 {false} else {true};
            let other_as_rust_boolean = if *other_int == 0 {false} else {true};
            if self_as_rust_boolean && other_as_rust_boolean {
                return interpreter.special_values[&SpecialValue::TrueValue];
            } else {
                return interpreter.special_values[&SpecialValue::FalseValue];
            }
        },
        _ => {
            
            if let Some(addr) = interpreter.call_method(params.params[0], "__bool__", vec![]) {
                //call the method again, but the argument is another boolean
                let bool_value_addr = interpreter.call_method(params.bound_pyobj.unwrap(), AND_STR, vec![addr]).unwrap();
                let bool_result = interpreter.get_raw_data_of_pyobj::<i128>(bool_value_addr);
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
                if *bool_result == 1 {
                    return params.params[0];
                } else {
                    return params.bound_pyobj.unwrap()
                }
            }
            
            return interpreter.special_values[&SpecialValue::NotImplementedValue];
        }
    };
}

fn or_method(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
    
    let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);            
    match other_type_name {
        "bool" => {
            let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);

            let self_as_rust_boolean = if *self_data == 0 {false} else {true};
            let other_as_rust_boolean = if *other_int == 0 {false} else {true};
            if self_as_rust_boolean || other_as_rust_boolean {
                return interpreter.special_values[&SpecialValue::TrueValue];
            } else {
                return interpreter.special_values[&SpecialValue::FalseValue];
            }
        },
        _ => {
            
            if let Some(addr) = interpreter.call_method(params.params[0], "__bool__", vec![]) {
                //call the method again, but the argument is another boolean
                let bool_value_addr = interpreter.call_method(params.bound_pyobj.unwrap(), OR_STR, vec![addr]).unwrap();
                let bool_result = interpreter.get_raw_data_of_pyobj::<i128>(bool_value_addr);
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
                if *bool_result == 1 {
                    return params.bound_pyobj.unwrap();
                } else {
                    return params.params[0];
                }
            }
            
            
            return interpreter.special_values[&SpecialValue::NotImplementedValue];
        }
    };
       
}

fn xor_method(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
    
    let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);            
    match other_type_name {
        "bool" => {
            let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);

            let self_as_rust_boolean = if *self_data == 0 {false} else {true};
            let other_as_rust_boolean = if *other_int == 0 {false} else {true};
            if self_as_rust_boolean ^ other_as_rust_boolean {
                return interpreter.special_values[&SpecialValue::TrueValue];
            } else {
                return interpreter.special_values[&SpecialValue::FalseValue];
            }
        },
        _ => {
            
            if let Some(addr) = interpreter.call_method(params.params[0], "__bool__", vec![]) {
                //call the method again, but the argument is another boolean
                let bool_value_addr = interpreter.call_method(params.bound_pyobj.unwrap(), XOR_STR, vec![addr]).unwrap();
                let bool_result = interpreter.get_raw_data_of_pyobj::<i128>(bool_value_addr);
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
                if *bool_result == 1 {
                    return params.bound_pyobj.unwrap();
                } else {
                    return params.params[0];
                }
            }
            
            
            return interpreter.special_values[&SpecialValue::NotImplementedValue];
        }
    };
}

fn not_method(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = *interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());

    if self_data == 0 {
        return interpreter.special_values[&SpecialValue::TrueValue];
    } else {
        return interpreter.special_values[&SpecialValue::FalseValue];
    }
}

fn to_str(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    let self_data = *interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    if self_data == 0 {
        interpreter.allocate_type_byname_raw("str", Box::new(String::from("False")))
    } else {
        interpreter.allocate_type_byname_raw("str", Box::new(String::from("True")))
    }
}

fn repr(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = *interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    if self_data == 0 {
        interpreter.allocate_type_byname_raw("str", Box::new(String::from("False")))
    } else {
        interpreter.allocate_type_byname_raw("str", Box::new(String::from("True")))
    }
}

fn to_boolean(_interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    //no-op
    return params.bound_pyobj.unwrap();
}

fn to_int(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    if *self_data == 0 {
        interpreter.allocate_type_byname_raw("int", Box::new(0 as i128))
    } else {
        interpreter.allocate_type_byname_raw("int", Box::new(1 as i128))
    }
}


fn to_float(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    if *self_data == 0 {
        interpreter.allocate_type_byname_raw("float", Box::new(0.0 as f64))
    } else {
        interpreter.allocate_type_byname_raw("float", Box::new(1.0 as f64))
    }
}

pub fn register_boolean_type(interpreter: &mut Interpreter) -> MemoryAddress {
    //bool inherits from int
    
    let int_supertype = interpreter.find_in_module(BUILTIN_MODULE, "int").expect("int type not found");
    let float_type = interpreter.create_type(BUILTIN_MODULE, "bool", Some(int_supertype));

    interpreter.register_bounded_func(BUILTIN_MODULE, "bool", "__and__", and_method);
    interpreter.register_bounded_func(BUILTIN_MODULE, "bool", "__or__", or_method);
    interpreter.register_bounded_func(BUILTIN_MODULE, "bool", "__xor__", xor_method);
    interpreter.register_bounded_func(BUILTIN_MODULE, "bool", "__not__", not_method);

    interpreter.register_bounded_func(BUILTIN_MODULE, "bool", "__bool__", to_boolean);
    interpreter.register_bounded_func(BUILTIN_MODULE, "bool", "__str__", to_str);
    interpreter.register_bounded_func(BUILTIN_MODULE, "bool", "__repr__", repr);
    interpreter.register_bounded_func(BUILTIN_MODULE, "bool", "__int__", to_int);
    interpreter.register_bounded_func(BUILTIN_MODULE, "bool", "__float__", to_float);
    
    let true_value = interpreter.allocate_type_byname_raw("bool", Box::new(1 as i128));
    let false_value = interpreter.allocate_type_byname_raw("bool", Box::new(0 as i128));

    interpreter.special_values.insert(SpecialValue::TrueValue, true_value);
    interpreter.special_values.insert(SpecialValue::FalseValue, false_value);

    interpreter.freeze_in_memory(true_value);
    interpreter.freeze_in_memory(false_value);

    return float_type;
}

