
use crate::runtime::runtime::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;
fn to_str(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
 
    runtime.allocate_type_byaddr_raw(
        runtime.builtin_type_addrs.string,
        BuiltInTypeData::String("None".into()),
    )
}
fn to_boolean(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    return runtime.builtin_type_addrs.false_val;
}

fn equals(runtime: &Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
    let self_pyobj = runtime.get_pyobj_byaddr(params.params[0]);

    match self_pyobj.structure {
        PyObjectStructure::None => runtime.builtin_type_addrs.true_val,
        _ => runtime.builtin_type_addrs.false_val
    }
}

pub fn register_none_type_methods(runtime: &mut Runtime) {
    let none_type_addr = runtime.special_values[&SpecialValue::NoneType];

    runtime.register_bounded_func_on_addr(none_type_addr, "__str__", to_str);
    runtime.register_bounded_func_on_addr(none_type_addr, "__eq__", equals);
    runtime.register_bounded_func_on_addr(none_type_addr, "__bool__", to_boolean);
}
