use crate::runtime::*;

fn create_concat(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
    let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
    let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);

    if other_type_name == "str" {
        let other_str = interpreter.get_raw_data_of_pyobj::<String>(params.params[0]);
        let new_str = format!("{}{}", self_data, other_str);
        interpreter.allocate_type_byname_raw("str", Box::new(new_str))
    } else {
        panic!("can only concatenate str (not \"{}\") to str", other_type_name);
    }
}

fn create_eq(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
    let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
    let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);

    if other_type_name == "str" {
        let other_str = interpreter.get_raw_data_of_pyobj::<String>(params.params[0]);
        if self_data == other_str {
            return interpreter.special_values[&SpecialValue::TrueValue];
        } else {
            return interpreter.special_values[&SpecialValue::FalseValue];
        }
    } else {
        return interpreter.special_values[&SpecialValue::FalseValue];
    }
}

fn create_neq(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 1, params.params.len());
    let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
    let other_type_name = interpreter.get_pyobj_type_name(params.params[0]);

    if other_type_name == "str" {
        let other_str = interpreter.get_raw_data_of_pyobj::<String>(params.params[0]);

        if self_data == other_str {
            return interpreter.special_values[&SpecialValue::FalseValue];
        } else {
            return interpreter.special_values[&SpecialValue::TrueValue];
        }
    } else {
        return interpreter.special_values[&SpecialValue::FalseValue];
    }
}

fn create_to_int(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
    let as_int = self_data.parse::<i128>().unwrap();
    interpreter.allocate_type_byname_raw("int", Box::new(as_int))
}

fn create_to_float(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
    let as_int = self_data.parse::<f64>().unwrap();
    interpreter.allocate_type_byname_raw("float", Box::new(as_int))
}

fn create_to_str(_interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    return params.bound_pyobj.unwrap();
}

fn create_repr(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
    let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
    interpreter.allocate_type_byname_raw("str", Box::new(format!("\'{}\'", self_data)))
}

fn create_new(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
    if params.params.len() == 0 {
        return interpreter.allocate_type_byname_raw("str", Box::new(String::from("")));
    } else {
        check_builtin_func_params!("str", 1, params.params.len());

        //try call the __str__ method on the parameter
        let string_call = interpreter.call_method(params.params[0], "__str__", vec![]);
        match string_call {
            Some(addr) => addr,
            None => panic!("Object passed to str does not have __str__ method")
        }
    }
}

macro_rules! create_transform_function {
    ($name:tt, $param_a:tt, $func:expr) => {
        fn $name(interpreter: &Interpreter, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap().as_str(), 0, params.params.len());
            let self_data = interpreter.get_raw_data_of_pyobj::<String>(params.bound_pyobj.unwrap());
            let $param_a = self_data;
            let transformed = $func; 
            interpreter.allocate_type_byname_raw("str", Box::new(transformed))
        }
    }
}

create_transform_function!(str_lower, a, a.to_lowercase());
create_transform_function!(str_upper, a, a.to_uppercase());


pub fn register_string_type(interpreter: &Interpreter) -> MemoryAddress {
    let string_type = interpreter.create_type(BUILTIN_MODULE, "str", None);

    interpreter.register_bounded_func(BUILTIN_MODULE, "str",  "__add__", create_concat);
    interpreter.register_bounded_func(BUILTIN_MODULE, "str",  "__eq__", create_eq);
    interpreter.register_bounded_func(BUILTIN_MODULE, "str",  "__neq__", create_neq);
    interpreter.register_bounded_func(BUILTIN_MODULE, "str",  "__int__", create_to_int);
    interpreter.register_bounded_func(BUILTIN_MODULE, "str",  "__float__", create_to_float);
    interpreter.register_bounded_func(BUILTIN_MODULE, "str",  "__repr__",create_repr);
    interpreter.register_bounded_func(BUILTIN_MODULE, "str",  "__str__", create_to_str);
    interpreter.register_bounded_func(BUILTIN_MODULE, "str",  "__new__", create_new);
    interpreter.register_bounded_func(BUILTIN_MODULE, "str",  "lower", str_lower);
    interpreter.register_bounded_func(BUILTIN_MODULE, "str",  "upper", str_upper);

    return string_type;
}

