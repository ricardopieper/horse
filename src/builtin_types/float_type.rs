use crate::runtime::*;

macro_rules! create_compare_function {
    ($name:tt, $param_a:tt, $param_b:tt, $compare:expr) => {
        fn $name(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
            
            let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);
            let self_data = interpreter.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
            
            return match other_type_name {
               "bool" | "int" => {
                    let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    let $param_a = *self_data;
                    let $param_b = *other_int as f64;
                    let result_compare = $compare;
                    let result_as_int: i128 = if result_compare {1} else {0};
                   
                    interpreter.allocate_type_byname_raw("bool", Box::new(result_as_int))
                },
                "float" => {
                    let other_float = interpreter.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    let $param_a = *self_data;
                    let $param_b = *other_float;
                    let result_compare = $compare;
                    let result_as_int: i128 = if result_compare {1} else {0};
                    interpreter.allocate_type_byname_raw("bool", Box::new(result_as_int))
                },
                _ => {
                    interpreter.special_values[&SpecialValue::NotImplementedValue]
                }
            };
        }
    }
}

macro_rules! create_binop_function {
    ($name:tt, $param_a:tt, $param_b:tt, $binop:expr) => {
        fn $name(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
        
            check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
            
            let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);
            let self_data = interpreter.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
            
            return match other_type_name {
                "int" => {
                    let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    let $param_a = *self_data;
                    let $param_b = *other_int as f64;
                    interpreter.allocate_type_byname_raw("float", Box::new($binop))
                },
                "float" => {
                    let other_float = interpreter.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    let $param_a = *self_data;
                    let $param_b = *other_float;
                    interpreter.allocate_type_byname_raw("float", Box::new($binop))
                },
                _ => {
                    interpreter.special_values[&SpecialValue::NotImplementedValue]
                }
            };
        }
    }
}

macro_rules! create_unary_function {
    ($name:tt, $param_a:tt, $func:expr) => {
        fn $name(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
            let $param_a = *self_data;
            interpreter.allocate_type_byname_raw("float", Box::new($func))
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

fn to_boolean(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = interpreter.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
    if *self_data == 0.0 {
        interpreter.allocate_type_byname_raw("bool", Box::new(0))
    } else {
        interpreter.allocate_type_byname_raw("bool", Box::new(1))
    }
}

fn to_float(_interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    return params.bound_pyobj.unwrap();
}

fn to_int(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = *interpreter.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
    interpreter.allocate_type_byname_raw("int", Box::new(self_data as i128))
}

fn to_str(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {    
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = *interpreter.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
    let formatted = format!("{:?}", self_data);
    interpreter.allocate_type_byname_raw("str", Box::new(formatted))
}

fn repr(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = *interpreter.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
    let formatted = format!("{:?}", self_data);
    interpreter.allocate_type_byname_raw("str", Box::new(formatted))
}

pub fn register_float_type(interpreter: &Interpreter) -> MemoryAddress {
    let float_type = interpreter.create_type(BUILTIN_MODULE, "float", None);

    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__eq__", equals);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__gt__", greater_than);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__ge__", greater_equals);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__lt__", less_than);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__le__", less_equals);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__ne__", not_equals);

    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__add__", add);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__mod__", modulus);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__sub__", sub);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__mul__", mul);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__truediv__", truediv);

    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__neg__", negation);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__pos__", positive);

    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__bool__", to_boolean);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__int__", to_int);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__float__", to_float);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__str__", to_str);
    interpreter.register_bounded_func(BUILTIN_MODULE, "float",  "__repr__", repr);

    return float_type;
}

