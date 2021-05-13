use crate::runtime::runtime::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;


fn to_str(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_string()
        .clone();
    runtime.allocate_builtin_type_byname_raw(
        "str",
        BuiltInTypeData::String(format!("IndexError: {}", self_data)),
    )
}

fn repr(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_string()
        .clone();
    runtime.allocate_builtin_type_byname_raw(
        "str",
        BuiltInTypeData::String(format!("IndexError: {}", self_data)),
    )
}

pub fn register_indexerr_type(runtime: &mut Runtime) -> MemoryAddress {
    let index_err = runtime.create_type(BUILTIN_MODULE, "IndexError", None);
    runtime.register_bounded_func(BUILTIN_MODULE, "IndexError", "__str__", to_str);
    runtime.register_bounded_func(BUILTIN_MODULE, "IndexError", "__repr__", repr);
    runtime.builtin_type_addrs.index_err = index_err;
    return index_err;
}