use crate::runtime::*;
use std::collections::HashMap;

fn create_concat(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 1, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
            let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);

            if other_type_name == "str" {
                let other_str = interpreter.get_raw_data_of_pyobj::<String>(params.params[0]);
                let new_str = format!("{}{}", self_data, other_str);
                interpreter.allocate_type_byname_raw("str", Box::new(new_str))
            } else {
                panic!("can only concatenate str (not \"{}\") to str", other_type_name);
            }
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__add__".to_string()));
    methods.insert("__add__".to_string(), func_addr);
}


fn create_to_int(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
            let as_int = self_data.parse::<i128>().unwrap();
            interpreter.allocate_type_byname_raw("int", Box::new(as_int))
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__int__".to_string()));
    methods.insert("__int__".to_string(), func_addr);
}


fn create_to_float(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
            let as_int = self_data.parse::<f64>().unwrap();
            interpreter.allocate_type_byname_raw("float", Box::new(as_int))
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__float__".to_string()));
    methods.insert("__float__".to_string(), func_addr);
}


fn create_to_str(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |_interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            return params.bound_pyobj.unwrap();
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__str__".to_string()));
    methods.insert("__str__".to_string(), func_addr);
}


fn create_repr(interpreter: & Interpreter, methods: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {
            check_builtin_func_params(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
            interpreter.allocate_type_byname_raw("str", Box::new(format!("\'{}\'", self_data)))
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__repr__".to_string()));
    methods.insert("__repr__".to_string(), func_addr);
}

fn create_new(interpreter: & Interpreter, functions: &mut HashMap<String, MemoryAddress>) {
    let func = PyCallable {
        code: Box::new(move |interpreter, params| -> MemoryAddress {

            if params.params.len() == 0 {
                return interpreter.allocate_type_byname_raw("str", Box::new(String::from("")));
            } else {
                check_builtin_func_params("str", 1, params.params.len());

                //try call the __str__ method on the parameter
                let string_call = interpreter.call_method(params.params[0], "__str__", vec![]);
                match string_call {
                    Some(addr) => addr,
                    None => panic!("Object passed to str does not have __str__ method")
                }
            }
        })
    };
    let func_addr = interpreter.create_callable_pyobj(func, Some("__new__".to_string()));
    functions.insert("__new__".to_string(), func_addr);
}


pub fn register_string_type(interpreter: &Interpreter) -> MemoryAddress {
    let mut methods = HashMap::new();
    
    create_concat(interpreter, &mut methods);
    create_to_str(interpreter, &mut methods);
    create_to_int(interpreter, &mut methods);
    create_to_float(interpreter, &mut methods);
    create_repr(interpreter, &mut methods);

    let mut functions = HashMap::new();
    create_new(interpreter, &mut functions);

    let created_type = interpreter.create_type(BUILTIN_MODULE, "str", methods, functions, None);
    return created_type;
}

