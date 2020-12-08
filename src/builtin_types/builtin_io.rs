use crate::runtime::*;

fn create_print_fn(runtime: &mut Runtime) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |runtime, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
            let str_call_result = runtime.call_method(params.params[0], "__str__", &[]).unwrap();
            let str_raw = runtime.get_raw_data_of_pyobj(str_call_result); 
            match str_raw {
                BuiltInTypeData::String(s) => {
                    println!("{}", s); 
                },
                _ => {
                    panic!("__str__ returned something else than string");
                }
            }
            return runtime.special_values[&SpecialValue::NoneValue];
        })
    };
    return runtime.create_callable_pyobj(func, Some("print".to_string()));
}

pub fn register_builtin_functions(runtime: &mut Runtime) {
    let print_fn = create_print_fn(runtime);
    runtime.add_to_module(BUILTIN_MODULE, "print", print_fn);
}

