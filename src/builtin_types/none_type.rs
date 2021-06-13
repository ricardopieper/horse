
use crate::runtime::vm::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;
fn to_str(vm: &VM, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
 
    vm.allocate_type_byaddr_raw(
        vm.builtin_type_addrs.string,
        BuiltInTypeData::String("None".into()),
    )
}
fn to_boolean(vm: &VM, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    return vm.builtin_type_addrs.false_val;
}

fn equals(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_pyobj = vm.get_pyobj_byaddr(call_params.params[0]);

    match self_pyobj.structure {
        PyObjectStructure::None => vm.builtin_type_addrs.true_val,
        _ => vm.builtin_type_addrs.false_val
    }
}

pub fn register_none_type_methods(vm: &mut VM) {
    let none_type_addr = vm.special_values[&SpecialValue::NoneType];

    vm.register_bounded_func_on_addr(none_type_addr, "__str__", to_str);
    vm.register_bounded_func_on_addr(none_type_addr, "__eq__", equals);
    vm.register_bounded_func_on_addr(none_type_addr, "__bool__", to_boolean);
}
