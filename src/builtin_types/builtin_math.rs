use crate::runtime::*;

fn create_function_1arg<FFloat>(runtime: & Runtime, name: &str, 
    op_float: FFloat) -> MemoryAddress where FFloat: Fn(f64) -> f64 + 'static {
    let func = PyCallable {
        code: Box::new(move |runtime, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
            let value_type_name = runtime.get_pyobj_type_name(params.params[0]);
            
            return match value_type_name {
                "int" => {
                    let parameter = runtime.get_raw_data_of_pyobj::<i128>(params.params[0]);
                    runtime.allocate_type_byname_raw("float", Box::new((op_float)(*parameter as f64)))
                },
                "float" => {
                    let other_float = runtime.get_raw_data_of_pyobj::<f64>(params.params[0]);
                    runtime.allocate_type_byname_raw("float", Box::new((op_float)(*other_float)))
                },
                _ => {
                    runtime.special_values[&SpecialValue::NotImplementedValue]
                }
            };
        })
    };
    return runtime.create_callable_pyobj(func, Some(name.to_string()));
}


fn create_function_2arg<FFloat>(runtime: & Runtime, name: &str, 
    op_float: FFloat) -> MemoryAddress where FFloat: Fn(f64, f64) -> f64 + 'static {
    let func = PyCallable {
        code: Box::new(move |runtime, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 2, params.params.len());
          
            let float1 = runtime.get_raw_data_of_pyobj::<f64>(params.params[0]);
            let float2 = runtime.get_raw_data_of_pyobj::<f64>(params.params[1]);
            runtime.allocate_type_byname_raw("float", Box::new((op_float)(*float1, *float2)))
        
        })
    };
    return runtime.create_callable_pyobj(func, Some(name.to_string()));
}

pub fn register_builtin_functions(runtime: &Runtime) {
    runtime.add_to_module(BUILTIN_MODULE, "sin", create_function_1arg(runtime, "sin", |f| f.sin()));
    runtime.add_to_module(BUILTIN_MODULE, "cos", create_function_1arg(runtime, "cos", |f| f.cos()));
    runtime.add_to_module(BUILTIN_MODULE, "tanh", create_function_1arg(runtime, "tanh", |f| f.tanh()));
    runtime.add_to_module(BUILTIN_MODULE, "test", create_function_2arg(runtime, "test", |x, y| x / y));
}

