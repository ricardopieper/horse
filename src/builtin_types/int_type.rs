use crate::commons::float::Float;
use crate::runtime::vm::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;


macro_rules! create_compare_function {
    ($name:tt, $param_a:tt, $param_b:tt, $compare:expr) => {
        fn $name(vm: &VM, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.as_ref().unwrap(), 2, params.params.len());
            let call_params = params.as_method();
            let other_type_addr = vm.get_pyobj_type_addr(call_params.params[0]);
            let self_data = vm
                .get_raw_data_of_pyobj(call_params.bound_pyobj)
                .take_int();
            let type_addr = &vm.builtin_type_addrs;

            if other_type_addr == type_addr.boolean || other_type_addr == type_addr.int {
                let other_int = vm.get_raw_data_of_pyobj(call_params.params[0]).take_int();
                let $param_a = self_data;
                let $param_b = other_int;
                if $compare {
                    return vm.builtin_type_addrs.true_val;
                } else {
                    return vm.builtin_type_addrs.false_val;
                }
            } else if other_type_addr == type_addr.float {
                let other_float = vm.get_raw_data_of_pyobj(call_params.params[0]).take_float();
                let $param_a = self_data as f64;
                let $param_b = other_float;
                if $compare {
                    return vm.builtin_type_addrs.true_val;
                } else {
                    return vm.builtin_type_addrs.false_val;
                }
            } else {
                return vm.builtin_type_addrs.false_val;
            }
        }
    };
}

macro_rules! create_binop_function {
    ($name:tt, $param_a:tt, $param_b:tt, $binop:expr) => {
        fn $name(vm: &VM, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.as_ref().unwrap(), 2, params.params.len());
            let call_params = params.as_method();
            let other_data = vm.get_raw_data_of_pyobj(call_params.params[0]);
            let other_type_addr = vm.get_pyobj_type_addr(call_params.params[0]);
            let self_data = vm
                .get_raw_data_of_pyobj(call_params.bound_pyobj)
                .take_int();
            if other_type_addr == vm.builtin_type_addrs.int {
                let other_int = other_data.take_int();
                let $param_a = self_data;
                let $param_b = other_int;
                vm.allocate_type_byaddr_raw(
                    vm.builtin_type_addrs.int,
                    BuiltInTypeData::Int($binop),
                )
            } else if other_type_addr == vm.builtin_type_addrs.float {
                let other_float = other_data.take_float();
                let $param_a = self_data as f64;
                let $param_b = other_float;
                vm.allocate_type_byaddr_raw(
                    vm.builtin_type_addrs.float,
                    BuiltInTypeData::Float(Float($binop)),
                )
            } else {
                return vm.special_values[&SpecialValue::NotImplementedValue];
            }
        }
    };
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

create_compare_function!(greater_than, a, b, a > b);
create_compare_function!(less_than, a, b, a < b);
create_compare_function!(equals, a, b, a == b);
create_compare_function!(less_equals, a, b, a <= b);
create_compare_function!(greater_equals, a, b, a <= b);
create_compare_function!(not_equals, a, b, a != b);

create_binop_function!(add, a, b, a + b);
create_binop_function!(modulus, a, b, a % b);
create_binop_function!(sub, a, b, a - b);
create_binop_function!(mul, a, b, a * b);

fn truediv(vm: &VM, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.as_ref().unwrap(), 2, params.params.len());
    let call_params = params.as_method();
    let other_type_name = vm.get_pyobj_type_name(call_params.params[0]);
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();

    return match other_type_name {
        "int" => {
            let other_int = vm.get_raw_data_of_pyobj(call_params.params[0]).take_int();
            vm.allocate_type_byaddr_raw(
                vm.builtin_type_addrs.float,
                BuiltInTypeData::Float(Float(self_data as f64 / other_int as f64)),
            )
        }
        "float" => {
            let other_float = vm.get_raw_data_of_pyobj(call_params.params[0]).take_float();
            vm.allocate_type_byaddr_raw(
                vm.builtin_type_addrs.float,
                BuiltInTypeData::Float(Float(self_data as f64 / other_float)),
            )
        }
        _ => vm.special_values[&SpecialValue::NotImplementedValue],
    };
}

create_unary_function!(negation, a, a * -1);
create_unary_function!(positive, a, a);

fn int(_vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    //no-op
    return call_params.bound_pyobj;
}

fn float(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    vm.allocate_type_byaddr_raw(
        vm.builtin_type_addrs.float,
        BuiltInTypeData::Float(Float(self_data as f64)),
    )
}

fn to_str(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    vm.allocate_type_byaddr_raw(
        vm.builtin_type_addrs.string,
        BuiltInTypeData::String(self_data.to_string()),
    )
}

fn repr(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    vm.allocate_type_byaddr_raw(
        vm.builtin_type_addrs.string,
        BuiltInTypeData::String(self_data.to_string()),
    )
}

fn to_boolean(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_int();
    if self_data == 1 {
        return vm.builtin_type_addrs.true_val;
    } else {
        return vm.builtin_type_addrs.false_val;
    }
}

pub fn register_int_type(vm: &mut VM) -> MemoryAddress {
    let int_type = vm.create_type(BUILTIN_MODULE, "int", None);

    vm.register_bounded_func(BUILTIN_MODULE, "int", "__eq__", equals);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__gt__", greater_than);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__ge__", greater_equals);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__lt__", less_than);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__le__", less_equals);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__ne__", not_equals);

    vm.register_bounded_func(BUILTIN_MODULE, "int", "__add__", add);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__mod__", modulus);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__sub__", sub);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__mul__", mul);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__truediv__", truediv);

    vm.register_bounded_func(BUILTIN_MODULE, "int", "__neg__", negation);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__pos__", positive);

    vm.register_bounded_func(BUILTIN_MODULE, "int", "__bool__", to_boolean);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__int__", int);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__float__", float);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__str__", to_str);
    vm.register_bounded_func(BUILTIN_MODULE, "int", "__repr__", repr);

    vm.builtin_type_addrs.int = int_type;

    return int_type;
}
