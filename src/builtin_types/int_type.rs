use crate::runtime::*;

macro_rules! create_compare_function {
    ($name:tt, $param_a:tt, $param_b:tt, $compare:expr) => {
        fn $name(runtime: &Runtime, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.as_ref().unwrap().as_str(), 1, params.params.len());
                    
            let other_type_name = runtime.get_pyobj_type_name(params.params[0]);
            let self_data = runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            
            return match other_type_name {
            "bool" | "int" => {
                    let other_int = runtime.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    let $param_a = self_data;
                    let $param_b = other_int;
                    if $compare {
                        return runtime.special_values[&SpecialValue::TrueValue];
                    } else {
                        return runtime.special_values[&SpecialValue::FalseValue];
                    }
                },
                "float" => {
                    let other_float = runtime.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    let $param_a = *self_data as f64;
                    let $param_b = *other_float;
                    if $compare {
                        return runtime.special_values[&SpecialValue::TrueValue];
                    } else {
                        return runtime.special_values[&SpecialValue::FalseValue];
                    }
                },
                _ => {
                    runtime.special_values[&SpecialValue::NotImplementedValue]
                }
            };
        }
    }
}

macro_rules! create_binop_function {
    ($name:tt, $param_a:tt, $param_b:tt, $binop:expr) => {
        fn $name(runtime: &Runtime, params: CallParams) -> MemoryAddress {
        
            check_builtin_func_params!(params.func_name.as_ref().unwrap().as_str(), 1, params.params.len());
            
            let other_type_name = runtime.get_pyobj_type_name(params.params[0]);
            let self_data = runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            
            return match other_type_name {
                "int" => {
                    let other_int = runtime.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    let $param_a = self_data;
                    let $param_b = other_int;
                    runtime.allocate_type_byname_raw("int", Box::new($binop))
                },
                "float" => {
                    let other_float = runtime.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    let $param_a = *self_data as f64;
                    let $param_b = *other_float;
                    runtime.allocate_type_byname_raw("float", Box::new($binop))
                },
                _ => {
                    runtime.special_values[&SpecialValue::NotImplementedValue]
                }
            };
        }
    }
}

macro_rules! create_unary_function {
    ($name:tt, $param_a:tt, $func:expr) => {
        fn $name(runtime: &Runtime, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            let $param_a = *self_data;
            runtime.allocate_type_byname_raw("int", Box::new($func))            
        }
    }
}

create_compare_function!(greater_than, a, b, a > b);
create_compare_function!(less_than, a, b, a < b);
create_compare_function!(equals, a, b, a == b);
create_compare_function!(less_equals, a, b, a <= b);
create_compare_function!(greater_equals, a, b, a <= b);
create_compare_function!(not_equals, a, b, a != b);

create_binop_function!(add, a, b, a + b);
create_binop_function!(modulus, a, b, a % b);
create_binop_function!(sub, a, b, a - b);
create_binop_function!(mul, a, b, a * b);

fn truediv(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.as_ref().unwrap().as_str(), 1, params.params.len());
            
    let other_type_name = runtime.get_pyobj_type_name(params.params[0]);
    let self_data = runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    
    return match other_type_name {
        "int" => {
            let other_int = runtime.get_raw_data_of_pyobj::<i128>(params.params[0]);
            runtime.allocate_type_byname_raw("float", Box::new(*self_data as f64 / *other_int as f64))
        },
        "float" => {
            let other_float = runtime.get_raw_data_of_pyobj::<f64>(params.params[0]);
            runtime.allocate_type_byname_raw("float", Box::new(*self_data as f64 / *other_float))
        },
        _ => {
            runtime.special_values[&SpecialValue::NotImplementedValue]
        }
    };
}

create_unary_function!(negation, a, a * -1);
create_unary_function!(positive, a, a);

fn int(_runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    //no-op
    return params.bound_pyobj.unwrap();
}

fn float(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = *runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    runtime.allocate_type_byname_raw("float", Box::new(self_data as f64))
}


fn to_str(runtime: & Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = *runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    runtime.allocate_type_byname_raw("str", Box::new(self_data.to_string()))
}

fn repr(runtime: & Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = *runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    runtime.allocate_type_byname_raw("str", Box::new(self_data.to_string()))
}

fn to_boolean(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = runtime.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
    if *self_data == 1 {
        return runtime.special_values[&SpecialValue::TrueValue];
    } else {
        return runtime.special_values[&SpecialValue::FalseValue];
    }
}


pub fn register_int_type(runtime: &Runtime) -> MemoryAddress {
    let int_type = runtime.create_type(BUILTIN_MODULE, "int", None);
    
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__eq__", equals);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__gt__", greater_than);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__ge__", greater_equals);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__lt__", less_than);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__le__", less_equals);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__ne__", not_equals);

    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__add__", add);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__mod__", modulus);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__sub__", sub);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__mul__", mul);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__truediv__", truediv);

    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__neg__", negation);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__pos__", positive);

    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__bool__", to_boolean);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__int__", int);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__float__", float);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__str__", to_str);
    runtime.register_bounded_func(BUILTIN_MODULE, "int",  "__repr__", repr);
   
    return int_type;
}