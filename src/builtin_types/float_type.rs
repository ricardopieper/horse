use crate::runtime::*;

macro_rules! create_compare_function {
    ($name:tt, $param_a:tt, $param_b:tt, $compare:expr) => {
        fn $name(runtime: &Runtime, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
            
            let other_type_name = runtime.get_pyobj_type_name(params.params[0]);
            let self_data = runtime.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
            
            return match other_type_name {
               "bool" | "int" => {
                    let other_int = runtime.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    let $param_a = *self_data;
                    let $param_b = *other_int as f64;
                    let result_compare = $compare;
                    let result_as_int: i128 = if result_compare {1} else {0};
                   
                    runtime.allocate_type_byname_raw("bool", Box::new(result_as_int))
                },
                "float" => {
                    let other_float = runtime.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    let $param_a = *self_data;
                    let $param_b = *other_float;
                    let result_compare = $compare;
                    let result_as_int: i128 = if result_compare {1} else {0};
                    runtime.allocate_type_byname_raw("bool", Box::new(result_as_int))
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
        
            check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
            
            let other_type_name = runtime.get_pyobj_type_name(params.params[0]);
            let self_data = runtime.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
            
            return match other_type_name {
                "int" => {
                    let other_int = runtime.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    let $param_a = *self_data;
                    let $param_b = *other_int as f64;
                    runtime.allocate_type_byname_raw("float", Box::new($binop))
                },
                "float" => {
                    let other_float = runtime.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    let $param_a = *self_data;
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
            check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
            let self_data = runtime.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
            let $param_a = *self_data;
            runtime.allocate_type_byname_raw("float", Box::new($func))
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
create_binop_function!(truediv, a, b, a / b);

create_unary_function!(negation, a, a * -1.0);
create_unary_function!(positive, a, a);

fn to_boolean(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = runtime.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
    if *self_data == 0.0 {
        runtime.allocate_type_byname_raw("bool", Box::new(0))
    } else {
        runtime.allocate_type_byname_raw("bool", Box::new(1))
    }
}

fn to_float(_runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    return params.bound_pyobj.unwrap();
}

fn to_int(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = *runtime.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
    runtime.allocate_type_byname_raw("int", Box::new(self_data as i128))
}

fn to_str(runtime: &Runtime, params: CallParams) -> MemoryAddress {    
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = *runtime.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
    let formatted = format!("{:?}", self_data);
    runtime.allocate_type_byname_raw("str", Box::new(formatted))
}

fn repr(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = *runtime.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
    let formatted = format!("{:?}", self_data);
    runtime.allocate_type_byname_raw("str", Box::new(formatted))
}

pub fn register_float_type(runtime: &mut Runtime) -> MemoryAddress {
    let float_type = runtime.create_type(BUILTIN_MODULE, "float", None);

    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__eq__", equals);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__gt__", greater_than);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__ge__", greater_equals);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__lt__", less_than);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__le__", less_equals);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__ne__", not_equals);

    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__add__", add);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__mod__", modulus);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__sub__", sub);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__mul__", mul);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__truediv__", truediv);

    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__neg__", negation);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__pos__", positive);

    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__bool__", to_boolean);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__int__", to_int);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__float__", to_float);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__str__", to_str);
    runtime.register_bounded_func(BUILTIN_MODULE, "float",  "__repr__", repr);

    runtime.builtin_type_addrs.float = float_type;

    return float_type;
}

