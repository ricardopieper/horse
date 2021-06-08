use crate::runtime::runtime::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;


fn concat(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj_mut(call_params.bound_pyobj)
        .take_list()
        .clone();
    let other_data = runtime.get_raw_data_of_pyobj(call_params.params[0]);

    match other_data {
        BuiltInTypeData::List(values) => {
            let mut result = vec![];
            result.extend(self_data);
            result.extend(values.iter().cloned());
            return runtime.allocate_type_byaddr_raw(
                runtime.builtin_type_addrs.list,
                BuiltInTypeData::List(result),
            );
        }
        _ => {
            let other_type_name = runtime.get_pyobj_type_name(call_params.params[0]);
            panic!(
                "can only concatenate list (not \"{}\") to list",
                other_type_name
            );
        }
    }
}

fn extend(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let other_data = runtime.get_raw_data_of_pyobj(call_params.params[0]);
    match other_data {
        BuiltInTypeData::List(values) => {
            let cloned = values.clone();
            let self_data = runtime
                .get_raw_data_of_pyobj_mut(call_params.bound_pyobj)
                .take_list_mut();
            (*self_data).extend(cloned);
            return call_params.bound_pyobj;
        }
        _ => {
            let other_type_name = runtime.get_pyobj_type_name(call_params.params[0]);
            panic!(
                "slowpython only supports extending from list (not \"{}\") for now",
                other_type_name
            );
        }
    }
}

fn append(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj_mut(call_params.bound_pyobj)
        .take_list_mut();
    self_data.push(call_params.params[0]);
    return call_params.bound_pyobj;
}

fn equals(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let this_list = runtime
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_list();
    let other_data = runtime.get_raw_data_of_pyobj(call_params.params[0]);

    match other_data {
        BuiltInTypeData::List(other_list) => {
            if this_list.len() != other_list.len() {
                return runtime.builtin_type_addrs.false_val;
            }
            let mut list_equals = true;
            for ptr_self in this_list.iter() {
                for ptr_other in other_list.iter() {
                    if ptr_self == ptr_other {
                        continue;
                    }
                    let result = runtime.call_method(*ptr_self, "__eq__", &[*ptr_other]);
                    match result {
                        Some((eq_result, _)) => {
                            if eq_result == runtime.builtin_type_addrs.false_val {
                                list_equals = false;
                                break;
                            }
                        }
                        None => {
                            list_equals = false;
                        }
                    }
                }
            }
            if list_equals {
                return runtime.builtin_type_addrs.false_val;
            } else {
                return runtime.builtin_type_addrs.true_val;
            }
        }
        _ => {
            return runtime.builtin_type_addrs.false_val;
        }
    }
}

fn not_equals(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let result = runtime.call_method(call_params.bound_pyobj, "__eq__", &[call_params.params[0]]);
    match result {
        Some((eq_result, _)) => {
            if eq_result == runtime.builtin_type_addrs.false_val {
                return runtime.builtin_type_addrs.true_val;
            } else {
                return runtime.builtin_type_addrs.false_val;
            }
        }
        None => {
            return runtime.builtin_type_addrs.false_val;
        }
    }
}

fn repr(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let this_list = runtime
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_list();
    let mut buffer = String::from("[");

    let all_reprs: Vec<String> = this_list
        .iter()
        .map(|ptr_self| {
            let (as_string, _) = runtime.call_method(*ptr_self, "__repr__", &[]).unwrap();
            return runtime
                .get_pyobj_byaddr(as_string)
                .try_get_builtin()
                .unwrap()
                .take_string()
                .clone();
        })
        .collect();

    buffer = buffer + all_reprs.join(",").as_str();
    buffer.push(']');

    runtime.allocate_type_byaddr_raw(
        runtime.builtin_type_addrs.string,
        BuiltInTypeData::String(buffer),
    )
}

fn to_str(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let this_list = runtime
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_list();
    let mut buffer = String::from("[");

    let all_reprs: Vec<String> = this_list
        .iter()
        .map(|ptr_self| {
            let (as_string, _) = runtime.call_method(*ptr_self, "__repr__", &[]).unwrap();
            return runtime
                .get_raw_data_of_pyobj(as_string)
                .take_string()
                .clone();
        })
        .collect();

    buffer = buffer + all_reprs.join(", ").as_str();
    buffer.push(']');

    runtime.allocate_type_byaddr_raw(
        runtime.builtin_type_addrs.string,
        BuiltInTypeData::String(buffer),
    )
}

fn len(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let this_list = runtime
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_list();
    let list_len = this_list.len();
    runtime.allocate_type_byaddr_raw(
        runtime.builtin_type_addrs.int,
        BuiltInTypeData::Int(list_len as i128),
    )
}


fn iter(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    //construct a list_iterator
    //find it in builtin module
    let iterator_class = runtime.find_in_module(MAIN_MODULE, "list_iterator").unwrap();
    let new = runtime.try_load_function_addr(iterator_class); //this is the __new__ method, try_load_function_addr automatically gets the __new__ function
    let (result, _) = runtime.run_function(&mut vec![call_params.bound_pyobj], new, None);
    return result;
}

fn getitem(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let this_list = runtime
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_list();
    
    let index = runtime.get_raw_data_of_pyobj(call_params.params[0]).take_int();

    if index as usize >= this_list.len() {
        let exception = runtime.allocate_type_byaddr_raw(runtime.builtin_type_addrs.index_err, BuiltInTypeData::String("list index out of range".into()));
        runtime.raise_exception(exception);
        return exception;
    } else {
        let value_at_index = this_list[index as usize];
        return value_at_index
    }

}

pub fn register_list_type(runtime: &mut Runtime) -> MemoryAddress {
    let list_type = runtime.create_type(BUILTIN_MODULE, "list", None);

    runtime.register_bounded_func(BUILTIN_MODULE, "list", "__add__", concat);
    runtime.register_bounded_func(BUILTIN_MODULE, "list", "__eq__", equals);
    runtime.register_bounded_func(BUILTIN_MODULE, "list", "__neq__", not_equals);
    runtime.register_bounded_func(BUILTIN_MODULE, "list", "__repr__", repr);
    runtime.register_bounded_func(BUILTIN_MODULE, "list", "__str__", to_str);
    runtime.register_bounded_func(BUILTIN_MODULE, "list", "__len__", len);
    runtime.register_bounded_func(BUILTIN_MODULE, "list", "__getitem__", getitem);
    runtime.register_bounded_func(BUILTIN_MODULE, "list", "__iter__", iter);
    runtime.register_bounded_func(BUILTIN_MODULE, "list", "append", append);
    runtime.register_bounded_func(BUILTIN_MODULE, "list", "extend", extend);
    runtime.builtin_type_addrs.list = list_type;
    return list_type;
}
