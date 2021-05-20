use crate::commons::float::Float;
use crate::runtime::runtime::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;


fn create_concat(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_string();
    let other_type_name = runtime.get_pyobj_type_name(call_params.params[0]);

    if other_type_name == "str" {
        let other_str = runtime
            .get_raw_data_of_pyobj(call_params.params[0])
            .take_string();
        let new_str = format!("{}{}", self_data, other_str);
        runtime.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(new_str))
    } else {
        panic!(
            "can only concatenate str (not \"{}\") to str, {:?}",
            other_type_name,
            unsafe {&*call_params.params[0]}
        );
    }
}

fn create_eq(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = runtime.get_raw_data_of_pyobj(call_params.bound_pyobj);
    let other_type_name = runtime.get_pyobj_type_name(call_params.params[0]);

    if other_type_name == "str" {
        let other_str = runtime.get_raw_data_of_pyobj(call_params.params[0]);
        if self_data == other_str {
            return runtime.builtin_type_addrs.true_val;
        } else {
            return runtime.builtin_type_addrs.false_val;
        }
    } else {
        return runtime.builtin_type_addrs.false_val;
    }
}

fn create_neq(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = runtime.get_raw_data_of_pyobj(call_params.bound_pyobj);
    let other_type_name = runtime.get_pyobj_type_name(call_params.params[0]);

    if other_type_name == "str" {
        let other_str = runtime.get_raw_data_of_pyobj(call_params.params[0]);

        if self_data == other_str {
            return runtime.builtin_type_addrs.false_val;
        } else {
            return runtime.builtin_type_addrs.true_val;
        }
    } else {
        return runtime.builtin_type_addrs.false_val;
    }
}

fn create_to_int(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_string();
    let as_int = self_data.parse::<i128>().unwrap();
    runtime.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(as_int))
}

fn create_to_float(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_string();
    let as_float = self_data.parse::<f64>().unwrap();
    runtime.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(as_float)))
}

fn create_to_str(_runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    return call_params.bound_pyobj;
}

fn create_repr(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_string()
        .clone();
    runtime.allocate_builtin_type_byname_raw(
        "str",
        BuiltInTypeData::String(format!("\'{}\'", self_data)),
    )
}

fn create_new(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    if params.params.len() == 0 {
        return runtime
            .allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("")));
    } else {
        check_builtin_func_params!("str", 1, params.params.len());
        let call_params = params.as_method();
        //try call the __str__ method on the parameter
        let string_call = runtime.call_method(call_params.params[0], "__str__", &[]);
        match string_call {
            Some(addr) => addr,
            None => panic!("Object passed to str does not have __str__ method"),
        }
    }
}

macro_rules! create_transform_function {
    ($name:tt, $param_a:tt, $func:expr) => {
        fn $name(runtime: &Runtime, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
            let call_params = params.as_method();
            let self_data = runtime
                .get_raw_data_of_pyobj(call_params.bound_pyobj)
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

    runtime.register_type_unbounded_func(string_type, "__new__", create_new);
    runtime.register_bounded_func_on_addr(string_type, "__add__", create_concat);
    runtime.register_bounded_func_on_addr(string_type, "__eq__", create_eq);
    runtime.register_bounded_func_on_addr(string_type, "__neq__", create_neq);
    runtime.register_bounded_func_on_addr(string_type, "__int__", create_to_int);
    runtime.register_bounded_func_on_addr(string_type, "__float__", create_to_float);
    runtime.register_bounded_func_on_addr(string_type, "__repr__", create_repr);
    runtime.register_bounded_func_on_addr(string_type, "__str__", create_to_str);
    runtime.register_bounded_func_on_addr(string_type, "lower", str_lower);
    runtime.register_bounded_func_on_addr(string_type, "upper", str_upper);
    runtime.builtin_type_addrs.string = string_type;

    return string_type;
}
