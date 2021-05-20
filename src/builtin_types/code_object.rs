use crate::runtime::runtime::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;

fn get_bytecode(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = runtime.get_function_bytecode(call_params.bound_pyobj);

    let mut bytecode_repr = String::from("");

    for data in self_data {
        bytecode_repr.push_str(&format!("{:?}\n", data));
    }

    runtime.allocate_builtin_type_byname_raw(
        "str",
        BuiltInTypeData::String(bytecode_repr),
    )
}

pub fn register_codeobject_type(runtime: &mut Runtime) -> MemoryAddress {
    let codeobject_type = runtime.create_type(BUILTIN_MODULE, "code object", None);
    runtime.register_bounded_func(BUILTIN_MODULE, "code object", "__bytecode__", get_bytecode);
    runtime.builtin_type_addrs.code_object = codeobject_type;
    return codeobject_type;
}