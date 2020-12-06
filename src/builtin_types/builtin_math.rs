use crate::runtime::*;

fn create_function_1arg<FFloat>(interpreter: & Interpreter, name: &str, 
    op_float: FFloat) -> MemoryAddress where FFloat: Fn(f64) -> f64 + 'static {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
            let value_type_name = interpreter.get_pyobj_type_name(params.params[0]);
            
            return match value_type_name {
                "int" => {
                    let parameter = interpreter.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    interpreter.allocate_type_byname_raw("float", Box::new((op_float)(*parameter as f64)))
                },
                "float" => {
                    let other_float = interpreter.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    interpreter.allocate_type_byname_raw("float", Box::new((op_float)(*other_float)))
                },
                _ => {
                    interpreter.special_values[&SpecialValue::NotImplementedValue]
                }
            };
        })
    };
    return interpreter.create_callable_pyobj(func, Some(name.to_string()));
}


fn create_function_2arg<FFloat>(interpreter: & Interpreter, name: &str, 
    op_float: FFloat) -> MemoryAddress where FFloat: Fn(f64, f64) -> f64 + 'static {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap().as_str(), 2, params.params.len());
          
            let float1 = interpreter.get_raw_data_of_pyobj::<f64>(params.params[0]);
            let float2 = interpreter.get_raw_data_of_pyobj::<f64>(params.params[1]);
            interpreter.allocate_type_byname_raw("float", Box::new((op_float)(*float1, *float2)))
        
        })
    };
    return interpreter.create_callable_pyobj(func, Some(name.to_string()));
}

pub fn register_builtin_functions(interpreter: &Interpreter) {
    interpreter.add_to_module(BUILTIN_MODULE, "sin", create_function_1arg(interpreter, "sin", |f| f.sin()));
    interpreter.add_to_module(BUILTIN_MODULE, "cos", create_function_1arg(interpreter, "cos", |f| f.cos()));
    interpreter.add_to_module(BUILTIN_MODULE, "tanh", create_function_1arg(interpreter, "tanh", |f| f.tanh()));
    interpreter.add_to_module(BUILTIN_MODULE, "test", create_function_2arg(interpreter, "test", |x, y| x / y));
}

