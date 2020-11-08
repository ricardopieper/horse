use crate::runtime::*;

fn create_print_fn(interpreter: & Interpreter) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 1, params.params.len());
            let str_call_result = interpreter.call_method(params.params[0], "__str__", vec![]).unwrap();
            let str_raw = interpreter.get_raw_data_of_pyobj::<String>(str_call_result);   
            println!("{}", str_raw); 
            return interpreter.special_values.none_value;
        })
    };
    return interpreter.create_callable_pyobj(func, Some("print".to_string()));
}

pub fn register_builtin_functions(interpreter: &Interpreter) {
    interpreter.add_to_module(BUILTIN_MODULE, "print", create_print_fn(interpreter));
}

