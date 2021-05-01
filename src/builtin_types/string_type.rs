use crate::float::Float;
use crate::runtime::*;

fn create_concat(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_string();
    let other_type_name = runtime.get_pyobj_type_name(params.params[0]);

    if other_type_name == "str" {
        let other_str = runtime
            .get_raw_data_of_pyobj(params.params[0])
            .take_string();
        let new_str = format!("{}{}", self_data, other_str);
        runtime.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(new_str))
    } else {
        panic!(
            "can only concatenate str (not \"{}\") to str",
            other_type_name
        );
    }
}

fn create_eq(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
    let self_data = runtime.get_raw_data_of_pyobj(params.bound_pyobj.unwrap());
    let other_type_name = runtime.get_pyobj_type_name(params.params[0]);

    if other_type_name == "str" {
        let other_str = runtime.get_raw_data_of_pyobj(params.params[0]);
        if self_data == other_str {
            return runtime.builtin_type_addrs.true_val;
        } else {
            return runtime.builtin_type_addrs.false_val;
        }
    } else {
        return runtime.builtin_type_addrs.false_val;
    }
}

fn create_neq(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
    let self_data = runtime.get_raw_data_of_pyobj(params.bound_pyobj.unwrap());
    let other_type_name = runtime.get_pyobj_type_name(params.params[0]);

    if other_type_name == "str" {
        let other_str = runtime.get_raw_data_of_pyobj(params.params[0]);

        if self_data == other_str {
            return runtime.builtin_type_addrs.false_val;
        } else {
            return runtime.builtin_type_addrs.true_val;
        }
    } else {
        return runtime.builtin_type_addrs.false_val;
    }
}

fn create_to_int(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_string();
    let as_int = self_data.parse::<i128>().unwrap();
    runtime.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(as_int))
}

fn create_to_float(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_string();
    let as_float = self_data.parse::<f64>().unwrap();
    runtime.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(as_float)))
}

fn create_to_str(_runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    return params.bound_pyobj.unwrap();
}

fn create_repr(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_string()
        .clone();
    runtime.allocate_builtin_type_byname_raw(
        "str",
        BuiltInTypeData::String(format!("\'{}\'", self_data)),
    )
}

fn create_new(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    if params.params.len() == 0 {
        return runtime
            .allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("")));
    } else {
        check_builtin_func_params!("str", 1, params.params.len());

        //try call the __str__ method on the parameter
        let string_call = runtime.call_method(params.params[0], "__str__", &[]);
        match string_call {
            Some(addr) => addr,
            None => panic!("Object passed to str does not have __str__ method"),
        }
    }
}

macro_rules! create_transform_function {
    ($name:tt, $param_a:tt, $func:expr) => {
        fn $name(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
            let self_data = runtime
                .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
                .take_string();
            let $param_a = self_data;
            let transformed = $func;
            runtime.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(transformed))
        }
    };
}

create_transform_function!(str_lower, a, a.to_lowercase());
create_transform_function!(str_upper, a, a.to_uppercase());

pub fn register_string_type(runtime: &mut Runtime) -> MemoryAddress {
    let string_type = runtime.create_type(BUILTIN_MODULE, "str", None);

    runtime.register_bounded_func(BUILTIN_MODULE, "str", "__add__", create_concat);
    runtime.register_bounded_func(BUILTIN_MODULE, "str", "__eq__", create_eq);
    runtime.register_bounded_func(BUILTIN_MODULE, "str", "__neq__", create_neq);
    runtime.register_bounded_func(BUILTIN_MODULE, "str", "__int__", create_to_int);
    runtime.register_bounded_func(BUILTIN_MODULE, "str", "__float__", create_to_float);
    runtime.register_bounded_func(BUILTIN_MODULE, "str", "__repr__", create_repr);
    runtime.register_bounded_func(BUILTIN_MODULE, "str", "__str__", create_to_str);
    runtime.register_bounded_func(BUILTIN_MODULE, "str", "__new__", create_new);
    runtime.register_bounded_func(BUILTIN_MODULE, "str", "lower", str_lower);
    runtime.register_bounded_func(BUILTIN_MODULE, "str", "upper", str_upper);
    runtime.builtin_type_addrs.string = string_type;
    return string_type;
}
