use crate::runtime::vm::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;


fn concat(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj_mut(call_params.bound_pyobj)
        .take_list()
        .clone();
    let other_data = vm.get_raw_data_of_pyobj(call_params.params[0]);

    match other_data {
        BuiltInTypeData::List(values) => {
            let mut result = vec![];
            result.extend(self_data);
            result.extend(values.iter().cloned());
            return vm.allocate_type_byaddr_raw(
                vm.builtin_type_addrs.list,
                BuiltInTypeData::List(result),
            );
        }
        _ => {
            let other_type_name = vm.get_pyobj_type_name(call_params.params[0]);
            panic!(
                "can only concatenate list (not \"{}\") to list",
                other_type_name
            );
        }
    }
}

fn extend(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let other_data = vm.get_raw_data_of_pyobj(call_params.params[0]);
    match other_data {
        BuiltInTypeData::List(values) => {
            let cloned = values.clone();
            let self_data = vm
                .get_raw_data_of_pyobj_mut(call_params.bound_pyobj)
                .take_list_mut();
            (*self_data).extend(cloned);
            return call_params.bound_pyobj;
        }
        _ => {
            let other_type_name = vm.get_pyobj_type_name(call_params.params[0]);
            panic!(
                "horse only supports extending from list (not \"{}\") for now",
                other_type_name
            );
        }
    }
}

fn append(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj_mut(call_params.bound_pyobj)
        .take_list_mut();
    self_data.push(call_params.params[0]);
    return call_params.bound_pyobj;
}

fn equals(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let this_list = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_list();
    let other_data = vm.get_raw_data_of_pyobj(call_params.params[0]);

    match other_data {
        BuiltInTypeData::List(other_list) => {
            if this_list.len() != other_list.len() {
                return vm.builtin_type_addrs.false_val;
            }
            let mut list_equals = true;
            for ptr_self in this_list.iter() {
                for ptr_other in other_list.iter() {
                    if ptr_self == ptr_other {
                        continue;
                    }
                    let result = vm.call_method(*ptr_self, "__eq__", PositionalParameters::single(*ptr_other));
                    match result {
                        Some((eq_result, _)) => {
                            if eq_result == vm.builtin_type_addrs.false_val {
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
                return vm.builtin_type_addrs.false_val;
            } else {
                return vm.builtin_type_addrs.true_val;
            }
        }
        _ => {
            return vm.builtin_type_addrs.false_val;
        }
    }
}

fn not_equals(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let result = vm.call_method(call_params.bound_pyobj, "__eq__", PositionalParameters::single(call_params.params[0]));
    match result {
        Some((eq_result, _)) => {
            if eq_result == vm.builtin_type_addrs.false_val {
                return vm.builtin_type_addrs.true_val;
            } else {
                return vm.builtin_type_addrs.false_val;
            }
        }
        None => {
            return vm.builtin_type_addrs.false_val;
        }
    }
}

fn repr(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let this_list = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_list();
    let mut buffer = String::from("[");

    let all_reprs: Vec<String> = this_list
        .iter()
        .map(|ptr_self| {
            let (as_string, _) = vm.call_method(*ptr_self, "__repr__", PositionalParameters::empty()).unwrap();
            return vm
                .get_pyobj_byaddr(as_string)
                .try_get_builtin()
                .unwrap()
                .take_string()
                .clone();
        })
        .collect();

    buffer = buffer + all_reprs.join(",").as_str();
    buffer.push(']');

    vm.allocate_type_byaddr_raw(
        vm.builtin_type_addrs.string,
        BuiltInTypeData::String(buffer),
    )
}

fn to_str(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let this_list = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_list();
    let mut buffer = String::from("[");

    let all_reprs: Vec<String> = this_list
        .iter()
        .map(|ptr_self| {
            let (as_string, _) = vm.call_method(*ptr_self, "__repr__", PositionalParameters::empty()).unwrap();
            return vm
                .get_raw_data_of_pyobj(as_string)
                .take_string()
                .clone();
        })
        .collect();

    buffer = buffer + all_reprs.join(", ").as_str();
    buffer.push(']');

    vm.allocate_type_byaddr_raw(
        vm.builtin_type_addrs.string,
        BuiltInTypeData::String(buffer),
    )
}

fn len(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let this_list = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_list();
    let list_len = this_list.len();
    vm.allocate_type_byaddr_raw(
        vm.builtin_type_addrs.int,
        BuiltInTypeData::Int(list_len as i128),
    )
}


fn iter(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    //construct a list_iterator
    //find it in builtin module
    let iterator_class = vm.find_in_module(MAIN_MODULE, "list_iterator").expect("list_iterator type not found");
    let new = vm.try_load_function_addr(iterator_class); //this is the __new__ method, try_load_function_addr automatically gets the __new__ function
    let (result, _) = vm.run_function(PositionalParameters::single(call_params.bound_pyobj), new, None);
    return result;
}

fn getitem(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let this_list = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_list();
    
    let index = vm.get_raw_data_of_pyobj(call_params.params[0]).take_int();

    if index as usize >= this_list.len() {
        let exception = vm.allocate_type_byaddr_raw(vm.builtin_type_addrs.index_err, BuiltInTypeData::String("list index out of range".into()));
        vm.raise_exception(exception);
        return exception;
    } else {
        let value_at_index = this_list[index as usize];
        return value_at_index
    }

}

fn create_new(vm: &VM, params: CallParams) -> MemoryAddress {
    if params.params.len() == 0 {
        return vm
            .allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("")));
    } else {
        if params.params.len() == 0 {
            return vm.allocate_type_byaddr_raw(
                vm.builtin_type_addrs.list,
                BuiltInTypeData::List(vec![]),
            );
        } else {
            check_builtin_func_params!("list", 1, params.params.len());
            let iterator_call = vm.call_method(params.params.params[0], "__iter__", PositionalParameters::empty());
            let mut results = vec![];
            loop {
                match iterator_call {
                    Some((addr, _)) => {
                        //start consuming the iterator
                        let (result, frame) = vm.call_method(addr, "__next__", PositionalParameters::empty()).unwrap();
                        if frame.exception.is_some() {
                            break;
                        } else {
                            results.push(result);
                        }
                    },
                    None => panic!("Object passed to list does not have __iter__ method"),
                }
            }   
            return vm.allocate_type_byaddr_raw(
                vm.builtin_type_addrs.list,
                BuiltInTypeData::List(results),
            );
        }
       
    }
}


pub fn register_list_type(vm: &mut VM) -> MemoryAddress {
    let list_type = vm.create_type(BUILTIN_MODULE, "list", None);

    vm.register_type_unbounded_func(list_type, "__new__", create_new);

    vm.register_bounded_func(BUILTIN_MODULE, "list", "__add__", concat);
    vm.register_bounded_func(BUILTIN_MODULE, "list", "__eq__", equals);
    vm.register_bounded_func(BUILTIN_MODULE, "list", "__neq__", not_equals);
    vm.register_bounded_func(BUILTIN_MODULE, "list", "__repr__", repr);
    vm.register_bounded_func(BUILTIN_MODULE, "list", "__str__", to_str);
    vm.register_bounded_func(BUILTIN_MODULE, "list", "__len__", len);
    vm.register_bounded_func(BUILTIN_MODULE, "list", "__getitem__", getitem);
    vm.register_bounded_func(BUILTIN_MODULE, "list", "__iter__", iter);
    vm.register_bounded_func(BUILTIN_MODULE, "list", "append", append);
    vm.register_bounded_func(BUILTIN_MODULE, "list", "extend", extend);
    vm.builtin_type_addrs.list = list_type;
    return list_type;
}
