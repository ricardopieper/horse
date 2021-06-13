use crate::commons::float::Float;
use crate::runtime::vm::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;


macro_rules! create_compare_function {
    ($name:tt, $param_a:tt, $param_b:tt, $compare:expr) => {
        fn $name(vm: &VM, params: CallParams) -> MemoryAddress {
            let call_params = params.as_method();
            check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
            let other_type_name = vm.get_pyobj_type_name(call_params.params[0]);
            let self_data = vm
                .get_raw_data_of_pyobj(call_params.bound_pyobj)
                .take_float();
            return match other_type_name {
                "bool" | "int" => {
                    let other_int = vm.get_raw_data_of_pyobj(call_params.params[0]).take_int();
                    let $param_a = self_data;
                    let $param_b = other_int as f64;
                    let result_compare = $compare;
                    let result_as_int: i128 = if result_compare { 1 } else { 0 };

                    vm.allocate_builtin_type_byname_raw(
                        "bool",
                        BuiltInTypeData::Int(result_as_int),
                    )
                }
                "float" => {
                    let other_float = vm.get_raw_data_of_pyobj(call_params.params[0]).take_float();
                    let $param_a = self_data;
                    let $param_b = other_float;
                    let result_compare = $compare;
                    let result_as_int: i128 = if result_compare { 1 } else { 0 };
                    vm.allocate_builtin_type_byname_raw(
                        "bool",
                        BuiltInTypeData::Int(result_as_int),
                    )
                }
                _ => vm.special_values[&SpecialValue::NotImplementedValue],
            };
        }
    };
}

macro_rules! create_binop_function {
    ($name:tt, $param_a:tt, $param_b:tt, $binop:expr) => {
        fn $name(vm: &VM, params: CallParams) -> MemoryAddress {
            let call_params = params.as_method();
            check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
            let other_type_name = vm.get_pyobj_type_name(call_params.params[0]);
            let self_data = vm
                .get_raw_data_of_pyobj(call_params.bound_pyobj)
                .take_float();
            return match other_type_name {
                "int" => {
                    let other_int = vm.get_raw_data_of_pyobj(call_params.params[0]).take_int();
                    let $param_a = self_data;
                    let $param_b = other_int as f64;
                    vm.allocate_builtin_type_byname_raw(
                        "float",
                        BuiltInTypeData::Float(Float($binop)),
                    )
                }
                "float" => {
                    let other_float = vm.get_raw_data_of_pyobj(call_params.params[0]).take_float();
                    let $param_a = self_data;
                    let $param_b = other_float;
                    vm.allocate_builtin_type_byname_raw(
                        "float",
                        BuiltInTypeData::Float(Float($binop)),
                    )
                }
                _ => vm.special_values[&SpecialValue::NotImplementedValue],
            };
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
                .take_float();
            let $param_a = self_data;
            vm.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float($func)))
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
create_binop_function!(truediv, a, b, a / b);

create_unary_function!(negation, a, a * -1.0);
create_unary_function!(positive, a, a);

fn to_boolean(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_float();
    if self_data == 0.0 {
        vm.allocate_builtin_type_byname_raw("bool", BuiltInTypeData::Int(0))
    } else {
        vm.allocate_builtin_type_byname_raw("bool", BuiltInTypeData::Int(1))
    }
}

fn to_float(_vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    return call_params.bound_pyobj;
}

fn to_int(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_float();
    vm.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(self_data as i128))
}

fn to_str(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_float();
    let formatted = format!("{:?}", self_data);
    vm.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(formatted))
}

fn repr(vm: &VM, params: CallParams) -> MemoryAddress {
    let call_params = params.as_method();
    check_builtin_func_params!(params.func_name.unwrap(), 0, call_params.params.len());
    let self_data = vm
        .get_raw_data_of_pyobj(call_params.bound_pyobj)
        .take_float();
    let formatted = format!("{:?}", self_data);
    vm.allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(formatted))
}

pub fn register_float_type(vm: &mut VM) -> MemoryAddress {
    let float_type = vm.create_type(BUILTIN_MODULE, "float", None);

    vm.register_bounded_func(BUILTIN_MODULE, "float", "__eq__", equals);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__gt__", greater_than);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__ge__", greater_equals);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__lt__", less_than);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__le__", less_equals);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__ne__", not_equals);

    vm.register_bounded_func(BUILTIN_MODULE, "float", "__add__", add);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__mod__", modulus);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__sub__", sub);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__mul__", mul);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__truediv__", truediv);

    vm.register_bounded_func(BUILTIN_MODULE, "float", "__neg__", negation);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__pos__", positive);

    vm.register_bounded_func(BUILTIN_MODULE, "float", "__bool__", to_boolean);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__int__", to_int);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__float__", to_float);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__str__", to_str);
    vm.register_bounded_func(BUILTIN_MODULE, "float", "__repr__", repr);

    vm.builtin_type_addrs.float = float_type;

    return float_type;
}
