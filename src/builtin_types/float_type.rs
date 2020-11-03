use crate::runtime::*;
use std::collections::HashMap;

fn create_binop_fn<FFloat>(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>,
    name: &str,
    op_float: FFloat) where FFloat: Fn(f64, f64) -> f64 + 'static {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 1, params.params.len());
            
            let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);
            let self_data = interpreter.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
            
            return match other_type_name {
                "int" => {
                    let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    interpreter.allocate_type_byname_raw("float", Box::new((op_float)( *self_data, *other_int as f64)))
                },
                "float" => {
                    let other_float = interpreter.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    interpreter.allocate_type_byname_raw("float", Box::new((op_float)(*self_data, *other_float)))
                },
                _ => {
                    interpreter.special_values.not_implemented_value
                }
            };
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some(name.to_string()));

    methods.insert(name.to_string(), func_addr);
}


fn create_unary_fn<FFloat>(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>,
    name: &str,
    op_float: FFloat) where FFloat: Fn(f64) -> f64 + 'static {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<f64>(params.bound_pyobj.unwrap());
            interpreter.allocate_type_byname_raw("float", Box::new((op_float)(*self_data)))        
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some(name.to_string()));

    methods.insert(name.to_string(), func_addr);
}


macro_rules! add_fn {
    ($interpreter:expr, $methods:expr, $name:expr, $binfunc:expr) => {
        create_binop_fn($interpreter, &mut $methods, $name, $binfunc)
    };
}

pub fn register_float_type(interpreter: &Interpreter) -> MemoryAddress {
    let mut methods = HashMap::new();
    add_fn!(interpreter, methods, "__add__", |a, b| a + b);
    add_fn!(interpreter, methods, "__sub__", |a, b| a - b);
    add_fn!(interpreter, methods, "__mul__", |a, b| a * b);
    add_fn!(interpreter, methods, "__truediv__", |a, b| a / b);
    create_unary_fn(interpreter, &mut methods, "__neg__", |a| a * -1.0);
    create_unary_fn(interpreter, &mut methods, "__pos__", |a| a);
    return interpreter.create_type(BUILTIN_MODULE, "float", methods, HashMap::new());
}

