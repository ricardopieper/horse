use crate::runtime::*;
use std::collections::HashMap;


fn create_compare_fn<FInt, FFloat>(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>,
    name: &str,
    op_int: FInt, op_float: FFloat) where FInt: Fn(i128, i128) -> bool + 'static ,  FFloat: Fn(f64, f64) -> bool + 'static {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
        
            check_builtin_func_params(params.func_name.unwrap().as_str(), 1, params.params.len());
            
            let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);
            let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            
            return match other_type_name {
               "bool" | "int" => {
                    let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    let result_compare = (op_int)(*self_data, *other_int);
                    let result_as_int: i128 = if result_compare {1} else {0};
                    interpreter.allocate_type_byname_raw("bool", Box::new(result_as_int))
                },
                "float" => {
                    let other_float = interpreter.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    let result_compare = (op_float)(*self_data as f64, *other_float as f64);
                    let result_as_int: i128 = if result_compare {1} else {0};
                    interpreter.allocate_type_byname_raw("bool", Box::new(result_as_int))
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


fn create_binop_fn<FInt, FFloat>(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>,
    name: &str,
    op_int: FInt, op_float: FFloat) where FInt: Fn(i128, i128) -> i128 + 'static ,  FFloat: Fn(f64, f64) -> f64 + 'static {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
        
            check_builtin_func_params(params.func_name.unwrap().as_str(), 1, params.params.len());
            
            let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);
            let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            
            return match other_type_name {
                "int" => {
                    let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    interpreter.allocate_type_byname_raw("int", Box::new((op_int)( *self_data, *other_int)))
                },
                "float" => {
                    let other_float = interpreter.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    interpreter.allocate_type_byname_raw("float", Box::new((op_float)(*self_data as f64, *other_float)))
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


fn create_div(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>,
    name: &str) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
        
            check_builtin_func_params(params.func_name.unwrap().as_str(), 1, params.params.len());
            
            let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);
            let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            
            return match other_type_name {
                "int" => {
                    let other_int = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    interpreter.allocate_type_byname_raw("float", Box::new(*self_data as f64 / *other_int as f64))
                },
                "float" => {
                    let other_float = interpreter.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    interpreter.allocate_type_byname_raw("float", Box::new(*self_data as f64 / *other_float))
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

fn create_unary_fn<FInt>(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>,
    name: &str,
    op_int: FInt) where FInt: Fn(i128) -> i128 + 'static {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            interpreter.allocate_type_byname_raw("int", Box::new((op_int)(*self_data)))            
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some(name.to_string()));

    methods.insert(name.to_string(), func_addr);
}


fn create_to_int(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |_interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            return params.bound_pyobj.unwrap();
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__int__".to_string()));
    methods.insert("__int__".to_string(), func_addr);
}


fn create_to_float(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = *interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            interpreter.allocate_type_byname_raw("float", Box::new(self_data as f64))
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__float__".to_string()));
    methods.insert("__float__".to_string(), func_addr);
}


fn create_to_str(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = *interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            interpreter.allocate_type_byname_raw("str", Box::new(self_data.to_string()))
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
            interpreter.allocate_type_byname_raw("str", Box::new(self_data.to_string()))
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__repr__".to_string()));
    methods.insert("__repr__".to_string(), func_addr);
}



fn create_to_boolean(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<i128>(params.bound_pyobj.unwrap());
            if *self_data == 0 {
                interpreter.allocate_type_byname_raw("bool", Box::new(0))
            } else {
                interpreter.allocate_type_byname_raw("bool", Box::new(1))
            }
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__bool__".to_string()));
    methods.insert("__bool__".to_string(), func_addr);
}


macro_rules! add_fn {
    ($interpreter:expr, $methods:expr, $name:expr, $binfunc:expr) => {
        create_binop_fn($interpreter, &mut $methods, $name, $binfunc, $binfunc)
    };
}


macro_rules! add_comparable {
    ($interpreter:expr, $methods:expr, $name:expr, $binfunc:expr) => {
        create_compare_fn($interpreter, &mut $methods, $name, $binfunc, $binfunc)
    };
}

pub fn register_int_type(interpreter: &Interpreter) -> MemoryAddress {
    let mut methods = HashMap::new();
    add_fn!(interpreter, methods, "__add__", |a, b| a + b);
    add_fn!(interpreter, methods, "__mod__", |a, b| a % b);
    add_fn!(interpreter, methods, "__sub__", |a, b| a - b);
    add_fn!(interpreter, methods, "__mul__", |a, b| a * b);
    add_fn!(interpreter, methods, "__truediv__", |a, b| a / b);
    
    create_div(interpreter, &mut methods, "__truediv__");
    
    create_to_boolean(interpreter, &mut methods);
    create_to_int(interpreter, &mut methods);
    create_to_float(interpreter, &mut methods);
    create_to_str(interpreter, &mut methods);
    create_repr(interpreter, &mut methods);
    
    create_unary_fn(interpreter, &mut methods, "__neg__", |a| a * -1);
    create_unary_fn(interpreter, &mut methods, "__pos__", |a| a);

    add_comparable!(interpreter, methods, "__eq__", |a, b| a == b);
    add_comparable!(interpreter, methods, "__gt__", |a, b| a > b);
    add_comparable!(interpreter, methods, "__ge__", |a, b| a >= b);
    add_comparable!(interpreter, methods, "__lt__", |a, b| a < b);
    add_comparable!(interpreter, methods, "__le__", |a, b| a <= b);
    add_comparable!(interpreter, methods, "__ne__", |a, b| a != b);

   
    return interpreter.create_type(BUILTIN_MODULE, "int", methods, HashMap::new(), None);
}

