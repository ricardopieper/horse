use crate::runtime::vm::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;


fn to_str(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_string()
        .clone();
    vm.allocate_builtin_type_byname_raw(
        "str",
        BuiltInTypeData::String(format!("IndexError: {}", self_data)),
    )
}

fn repr(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_string()
        .clone();
    vm.allocate_builtin_type_byname_raw(
        "str",
        BuiltInTypeData::String(format!("IndexError: {}", self_data)),
    )
}

pub fn register_indexerr_type(vm: &mut VM) -> MemoryAddress {
    let index_err = vm.create_type(BUILTIN_MODULE, "IndexError", None);
    vm.register_bounded_func(BUILTIN_MODULE, "IndexError", "__str__", to_str);
    vm.register_bounded_func(BUILTIN_MODULE, "IndexError", "__repr__", repr);
    vm.builtin_type_addrs.index_err = index_err;
    return index_err;
}