use crate::runtime::*;

fn create_print_fn(runtime: & Runtime) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |runtime, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
            let str_call_result = runtime.call_method(params.params[0], "__str__", &[]).unwrap();
            let str_raw = runtime.get_raw_data_of_pyobj::<String>(str_call_result);  
            //do not remove println 
            println!("{}", str_raw); 
            return runtime.special_values[&SpecialValue::NoneValue];
        })
    };
    return runtime.create_callable_pyobj(func, Some("print".to_string()));
}

pub fn register_builtin_functions(runtime: &Runtime) {
    runtime.add_to_module(BUILTIN_MODULE, "print", create_print_fn(runtime));
}

