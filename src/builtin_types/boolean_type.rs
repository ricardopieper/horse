use crate::runtime::*;

const AND_STR : &'static str = "__and__";
const OR_STR : &'static str = "__or__";
const XOR_STR : &'static str = "__xor__";


fn and_method(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
    
    let self_data = runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    let other_type_name = runtime.get_pyobj_type_name(params.params[0]);            
    match other_type_name {
        "bool" => {
            let other_int = runtime.get_raw_data_of_pyobj::<i128>(params.params[0]);

            let self_as_rust_boolean = if *self_data == 0 {false} else {true};
            let other_as_rust_boolean = if *other_int == 0 {false} else {true};
            if self_as_rust_boolean && other_as_rust_boolean {
                return runtime.special_values[&SpecialValue::TrueValue];
            } else {
                return runtime.special_values[&SpecialValue::FalseValue];
            }
        },
        _ => {
            
            if let Some(addr) = runtime.call_method(params.params[0], "__bool__", vec![]) {
                //call the method again, but the argument is another boolean
                let bool_value_addr = runtime.call_method(params.bound_pyobj.unwrap(), AND_STR, vec![addr]).unwrap();
                let bool_result = runtime.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                if *bool_result == 1 {
                    return params.params[0];
                } else {
                    return params.bound_pyobj.unwrap()
                }
            }

            if let Some(addr) = runtime.call_method(params.params[0], "__len__", vec![]) {
                //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
                let bool_value_addr = runtime.call_method(params.bound_pyobj.unwrap(), AND_STR, vec![addr]).unwrap();
                let bool_result = runtime.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                if *bool_result == 1 {
                    return params.params[0];
                } else {
                    return params.bound_pyobj.unwrap()
                }
            }
            
            return runtime.special_values[&SpecialValue::NotImplementedValue];
        }
    };
}

fn or_method(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
    
    let self_data = runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    let other_type_name = runtime.get_pyobj_type_name(params.params[0]);            
    match other_type_name {
        "bool" => {
            let other_int = runtime.get_raw_data_of_pyobj::<i128>(params.params[0]);

            let self_as_rust_boolean = if *self_data == 0 {false} else {true};
            let other_as_rust_boolean = if *other_int == 0 {false} else {true};
            if self_as_rust_boolean || other_as_rust_boolean {
                return runtime.special_values[&SpecialValue::TrueValue];
            } else {
                return runtime.special_values[&SpecialValue::FalseValue];
            }
        },
        _ => {
            
            if let Some(addr) = runtime.call_method(params.params[0], "__bool__", vec![]) {
                //call the method again, but the argument is another boolean
                let bool_value_addr = runtime.call_method(params.bound_pyobj.unwrap(), OR_STR, vec![addr]).unwrap();
                let bool_result = runtime.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                if *bool_result == 1 {
                    return params.bound_pyobj.unwrap();
                } else {
                    return params.params[0];
                }
            }

            if let Some(addr) = runtime.call_method(params.params[0], "__len__", vec![]) {
                //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
                let bool_value_addr = runtime.call_method(params.bound_pyobj.unwrap(), OR_STR, vec![addr]).unwrap();
                let bool_result = runtime.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                if *bool_result == 1 {
                    return params.bound_pyobj.unwrap();
                } else {
                    return params.params[0];
                }
            }
            
            
            return runtime.special_values[&SpecialValue::NotImplementedValue];
        }
    };
       
}

fn xor_method(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
    
    let self_data = runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    let other_type_name = runtime.get_pyobj_type_name(params.params[0]);            
    match other_type_name {
        "bool" => {
            let other_int = runtime.get_raw_data_of_pyobj::<i128>(params.params[0]);

            let self_as_rust_boolean = if *self_data == 0 {false} else {true};
            let other_as_rust_boolean = if *other_int == 0 {false} else {true};
            if self_as_rust_boolean ^ other_as_rust_boolean {
                return runtime.special_values[&SpecialValue::TrueValue];
            } else {
                return runtime.special_values[&SpecialValue::FalseValue];
            }
        },
        _ => {
            
            if let Some(addr) = runtime.call_method(params.params[0], "__bool__", vec![]) {
                //call the method again, but the argument is another boolean
                let bool_value_addr = runtime.call_method(params.bound_pyobj.unwrap(), XOR_STR, vec![addr]).unwrap();
                let bool_result = runtime.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                if *bool_result == 1 {
                    return params.bound_pyobj.unwrap();
                } else {
                    return params.params[0];
                }
            }

            if let Some(addr) = runtime.call_method(params.params[0], "__len__", vec![]) {
                //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
                let bool_value_addr = runtime.call_method(params.bound_pyobj.unwrap(), XOR_STR, vec![addr]).unwrap();
                let bool_result = runtime.get_raw_data_of_pyobj::<i128>(bool_value_addr);
                if *bool_result == 1 {
                    return params.bound_pyobj.unwrap();
                } else {
                    return params.params[0];
                }
            }
            
            
            return runtime.special_values[&SpecialValue::NotImplementedValue];
        }
    };
}

fn not_method(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = *runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());

    if self_data == 0 {
        return runtime.special_values[&SpecialValue::TrueValue];
    } else {
        return runtime.special_values[&SpecialValue::FalseValue];
    }
}

fn to_str(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let self_data = *runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    if self_data == 0 {
        runtime.allocate_type_byname_raw("str", Box::new(String::from("False")))
    } else {
        runtime.allocate_type_byname_raw("str", Box::new(String::from("True")))
    }
}

fn repr(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = *runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    if self_data == 0 {
        runtime.allocate_type_byname_raw("str", Box::new(String::from("False")))
    } else {
        runtime.allocate_type_byname_raw("str", Box::new(String::from("True")))
    }
}

fn to_boolean(_runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    //no-op
    return params.bound_pyobj.unwrap();
}

fn to_int(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    if *self_data == 0 {
        runtime.allocate_type_byname_raw("int", Box::new(0 as i128))
    } else {
        runtime.allocate_type_byname_raw("int", Box::new(1 as i128))
    }
}


fn to_float(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    if *self_data == 0 {
        runtime.allocate_type_byname_raw("float", Box::new(0.0 as f64))
    } else {
        runtime.allocate_type_byname_raw("float", Box::new(1.0 as f64))
    }
}

pub fn register_boolean_type(runtime: &mut Runtime) -> MemoryAddress {
    //bool inherits from int
    
    let int_supertype = runtime.find_in_module(BUILTIN_MODULE, "int").expect("int type not found");
    let float_type = runtime.create_type(BUILTIN_MODULE, "bool", Some(int_supertype));

    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__and__", and_method);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__or__", or_method);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__xor__", xor_method);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__not__", not_method);

    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__bool__", to_boolean);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__str__", to_str);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__repr__", repr);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__int__", to_int);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__float__", to_float);
    
    let true_value = runtime.allocate_type_byname_raw("bool", Box::new(1 as i128));
    let false_value = runtime.allocate_type_byname_raw("bool", Box::new(0 as i128));

    runtime.special_values.insert(SpecialValue::TrueValue, true_value);
    runtime.special_values.insert(SpecialValue::FalseValue, false_value);

    runtime.make_const(true_value);
    runtime.make_const(false_value);

    return float_type;
}

