use crate::commons::float::Float;
use crate::runtime::vm::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;

const AND_STR: &'static str = "__and__";
const OR_STR: &'static str = "__or__";
const XOR_STR: &'static str = "__xor__";

fn and_method(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    let other_type_addr = vm.get_pyobj_type_addr(call_params.params[0]);
    let boolean_type = vm.builtin_type_addrs.boolean;
    if other_type_addr == boolean_type {
        let other_int = vm.get_raw_data_of_pyobj(call_params.params[0]).take_int();

        let self_as_rust_boolean = if self_data == 0 { false } else { true };
        let other_as_rust_boolean = if other_int == 0 { false } else { true };
        if self_as_rust_boolean && other_as_rust_boolean {
            return vm.builtin_type_addrs.true_val;
        } else {
            return vm.builtin_type_addrs.false_val;
        }
    } else {
        if let Some((addr, _)) = vm.call_method(call_params.params[0], "__bool__", PositionalParameters::empty()) {
            //call the method again, but the argument is another boolean
            let (bool_value_addr, _) = vm
                .call_method(call_params.bound_pyobj, AND_STR, PositionalParameters::single(addr))
                .unwrap();
            let bool_result = vm.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return call_params.params[0];
            } else {
                return call_params.bound_pyobj;
            }
        }

        if let Some((addr, _)) = vm.call_method(call_params.params[0], "__len__", PositionalParameters::empty()) {
            //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
            let (bool_value_addr, _) = vm
                .call_method(call_params.bound_pyobj, AND_STR, PositionalParameters::single(addr))
                .unwrap();
            let bool_result = vm.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return call_params.params[0];
            } else {
                return call_params.bound_pyobj;
            }
        }
        return vm.special_values[&SpecialValue::NotImplementedValue];
    }
}

fn or_method(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    let other_type_addr = vm.get_pyobj_type_addr(call_params.params[0]);
    let boolean_type = vm.builtin_type_addrs.boolean;
    if other_type_addr == boolean_type {
        let other_int = vm.get_raw_data_of_pyobj(call_params.params[0]).take_int();

        let self_as_rust_boolean = if self_data == 0 { false } else { true };
        let other_as_rust_boolean = if other_int == 0 { false } else { true };
        if self_as_rust_boolean || other_as_rust_boolean {
            return vm.builtin_type_addrs.true_val;
        } else {
            return vm.builtin_type_addrs.false_val;
        }
    } else {
        if let Some((addr, _)) = vm.call_method(call_params.params[0], "__bool__", PositionalParameters::empty()) {
            //call the method again, but the argument is another boolean
            let (bool_value_addr, _) = vm
                .call_method(call_params.bound_pyobj, OR_STR, PositionalParameters::single(addr))
                .unwrap();
            let bool_result = vm.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return call_params.bound_pyobj;
            } else {
                return call_params.params[0];
            }
        }

        if let Some((addr, _)) = vm.call_method(call_params.params[0], "__len__", PositionalParameters::empty()) {
            //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
            let (bool_value_addr, _) = vm
                .call_method(call_params.bound_pyobj, OR_STR, PositionalParameters::single(addr))
                .unwrap();
            let bool_result = vm.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return call_params.bound_pyobj;
            } else {
                return call_params.params[0];
            }
        }
        return vm.special_values[&SpecialValue::NotImplementedValue];
    }
}

fn xor_method(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    let other_type_addr = vm.get_pyobj_type_addr(call_params.params[0]);
    let boolean_type = vm.builtin_type_addrs.boolean;
    if other_type_addr == boolean_type {
        let other_int = vm.get_raw_data_of_pyobj(call_params.params[0]).take_int();

        let self_as_rust_boolean = if self_data == 0 { false } else { true };
        let other_as_rust_boolean = if other_int == 0 { false } else { true };
        if self_as_rust_boolean ^ other_as_rust_boolean {
            return vm.builtin_type_addrs.true_val;
        } else {
            return vm.builtin_type_addrs.false_val;
        }
    } else {
        if let Some((addr, _)) = vm.call_method(call_params.params[0], "__bool__", PositionalParameters::empty()) {
            //call the method again, but the argument is another boolean
            let (bool_value_addr, _) = vm
                .call_method(call_params.bound_pyobj, XOR_STR, PositionalParameters::single(addr))
                .unwrap();
            let bool_result = vm.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return call_params.bound_pyobj;
            } else {
                return call_params.params[0];
            }
        }

        if let Some((addr, _)) = vm.call_method(call_params.params[0], "__len__", PositionalParameters::empty()) {
            //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
            let (bool_value_addr, _) = vm
                .call_method(call_params.bound_pyobj, XOR_STR, PositionalParameters::single(addr))
                .unwrap();
            let bool_result = vm.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return call_params.bound_pyobj;
            } else {
                return call_params.params[0];
            }
        }
        return vm.special_values[&SpecialValue::NotImplementedValue];
    }
}

fn not_method(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();

    if self_data == 0 {
        return vm.builtin_type_addrs.true_val;
    } else {
        return vm.builtin_type_addrs.false_val;
    }
}

fn to_str(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    if self_data == 0 {
        vm.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("False")))
    } else {
        vm.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("True")))
    }
}

fn repr(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    if self_data == 0 {
        vm.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("False")))
    } else {
        vm.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("True")))
    }
}

fn to_boolean(_vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());

    //no-op
    return call_params.bound_pyobj;
}

fn to_int(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    if self_data == 0 {
        vm.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(0 as i128))
    } else {
        vm.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(1 as i128))
    }
}

fn to_float(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    if self_data == 0 {
        vm.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(0.0 as f64)))
    } else {
        vm.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(1.0 as f64)))
    }
}

macro_rules! create_unary_function {
    ($name:tt, $param_a:tt, $func:expr) => {
        fn $name(vm: &VM, params: CallParams) -> MemoryAddress {
            let call_params = params.as_method();
            check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
            let self_data = vm
                .get_raw_data_of_pyobj(call_params.bound_pyobj)
                .take_int();
            let $param_a = self_data;
            vm.allocate_type_byaddr_raw(
                vm.builtin_type_addrs.int,
                BuiltInTypeData::Int($func),
            )
        }
    };
}
create_unary_function!(negation, a, if a == 1 { 0 } else { 1 });

pub fn register_boolean_type(vm: &mut VM) -> MemoryAddress {
    //bool inherits from int

    let int_supertype = vm
        .find_in_module(BUILTIN_MODULE, "int")
        .expect("int type not found");
    let boolean_type = vm.create_type(BUILTIN_MODULE, "bool", Some(int_supertype));

    vm.register_bounded_func(BUILTIN_MODULE, "bool", "__and__", and_method);
    vm.register_bounded_func(BUILTIN_MODULE, "bool", "__or__", or_method);
    vm.register_bounded_func(BUILTIN_MODULE, "bool", "__xor__", xor_method);
    vm.register_bounded_func(BUILTIN_MODULE, "bool", "__not__", not_method);
    vm.register_bounded_func(BUILTIN_MODULE, "bool", "__neg__", negation);

    vm.register_bounded_func(BUILTIN_MODULE, "bool", "__bool__", to_boolean);
    vm.register_bounded_func(BUILTIN_MODULE, "bool", "__str__", to_str);
    vm.register_bounded_func(BUILTIN_MODULE, "bool", "__repr__", repr);
    vm.register_bounded_func(BUILTIN_MODULE, "bool", "__int__", to_int);
    vm.register_bounded_func(BUILTIN_MODULE, "bool", "__float__", to_float);

    let true_value =
        vm.allocate_builtin_type_byname_raw("bool", BuiltInTypeData::Int(1 as i128));
    let false_value =
        vm.allocate_builtin_type_byname_raw("bool", BuiltInTypeData::Int(0 as i128));
    vm.builtin_type_addrs.true_val = true_value;
    vm.builtin_type_addrs.false_val = false_value;
    vm.make_const(true_value);
    vm.make_const(false_value);

    vm.builtin_type_addrs.boolean = boolean_type;

    return boolean_type;
}
