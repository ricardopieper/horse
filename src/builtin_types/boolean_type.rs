use crate::float::Float;
use crate::runtime::*;

const AND_STR: &'static str = "__and__";
const OR_STR: &'static str = "__or__";
const XOR_STR: &'static str = "__xor__";

fn and_method(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());

    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_int();
    let other_type_addr = runtime.get_pyobj_type_addr(params.params[0]);
    let boolean_type = runtime.builtin_type_addrs.boolean;
    if other_type_addr == boolean_type {
        let other_int = runtime.get_raw_data_of_pyobj(params.params[0]).take_int();

        let self_as_rust_boolean = if self_data == 0 { false } else { true };
        let other_as_rust_boolean = if other_int == 0 { false } else { true };
        if self_as_rust_boolean && other_as_rust_boolean {
            return runtime.builtin_type_addrs.true_val;
        } else {
            return runtime.builtin_type_addrs.false_val;
        }
    } else {
        if let Some(addr) = runtime.call_method(params.params[0], "__bool__", &[]) {
            //call the method again, but the argument is another boolean
            let bool_value_addr = runtime
                .call_method(params.bound_pyobj.unwrap(), AND_STR, &[addr])
                .unwrap();
            let bool_result = runtime.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return params.params[0];
            } else {
                return params.bound_pyobj.unwrap();
            }
        }

        if let Some(addr) = runtime.call_method(params.params[0], "__len__", &[]) {
            //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
            let bool_value_addr = runtime
                .call_method(params.bound_pyobj.unwrap(), AND_STR, &[addr])
                .unwrap();
            let bool_result = runtime.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return params.params[0];
            } else {
                return params.bound_pyobj.unwrap();
            }
        }
        return runtime.special_values[&SpecialValue::NotImplementedValue];
    }
}

fn or_method(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());

    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_int();
    let other_type_addr = runtime.get_pyobj_type_addr(params.params[0]);
    let boolean_type = runtime.builtin_type_addrs.boolean;
    if other_type_addr == boolean_type {
        let other_int = runtime.get_raw_data_of_pyobj(params.params[0]).take_int();

        let self_as_rust_boolean = if self_data == 0 { false } else { true };
        let other_as_rust_boolean = if other_int == 0 { false } else { true };
        if self_as_rust_boolean || other_as_rust_boolean {
            return runtime.builtin_type_addrs.true_val;
        } else {
            return runtime.builtin_type_addrs.false_val;
        }
    } else {
        if let Some(addr) = runtime.call_method(params.params[0], "__bool__", &[]) {
            //call the method again, but the argument is another boolean
            let bool_value_addr = runtime
                .call_method(params.bound_pyobj.unwrap(), OR_STR, &[addr])
                .unwrap();
            let bool_result = runtime.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return params.bound_pyobj.unwrap();
            } else {
                return params.params[0];
            }
        }

        if let Some(addr) = runtime.call_method(params.params[0], "__len__", &[]) {
            //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
            let bool_value_addr = runtime
                .call_method(params.bound_pyobj.unwrap(), OR_STR, &[addr])
                .unwrap();
            let bool_result = runtime.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return params.bound_pyobj.unwrap();
            } else {
                return params.params[0];
            }
        }
        return runtime.special_values[&SpecialValue::NotImplementedValue];
    }
}

fn xor_method(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());

    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_int();
    let other_type_addr = runtime.get_pyobj_type_addr(params.params[0]);
    let boolean_type = runtime.builtin_type_addrs.boolean;
    if other_type_addr == boolean_type {
        let other_int = runtime.get_raw_data_of_pyobj(params.params[0]).take_int();

        let self_as_rust_boolean = if self_data == 0 { false } else { true };
        let other_as_rust_boolean = if other_int == 0 { false } else { true };
        if self_as_rust_boolean ^ other_as_rust_boolean {
            return runtime.builtin_type_addrs.true_val;
        } else {
            return runtime.builtin_type_addrs.false_val;
        }
    } else {
        if let Some(addr) = runtime.call_method(params.params[0], "__bool__", &[]) {
            //call the method again, but the argument is another boolean
            let bool_value_addr = runtime
                .call_method(params.bound_pyobj.unwrap(), XOR_STR, &[addr])
                .unwrap();
            let bool_result = runtime.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return params.bound_pyobj.unwrap();
            } else {
                return params.params[0];
            }
        }

        if let Some(addr) = runtime.call_method(params.params[0], "__len__", &[]) {
            //call the method again, but the argument is the i128 __len__ value, which will be converted to boolean
            let bool_value_addr = runtime
                .call_method(params.bound_pyobj.unwrap(), XOR_STR, &[addr])
                .unwrap();
            let bool_result = runtime.get_raw_data_of_pyobj(bool_value_addr).take_int();
            if bool_result == 1 {
                return params.bound_pyobj.unwrap();
            } else {
                return params.params[0];
            }
        }
        return runtime.special_values[&SpecialValue::NotImplementedValue];
    }
}

fn not_method(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_int();

    if self_data == 0 {
        return runtime.builtin_type_addrs.true_val;
    } else {
        return runtime.builtin_type_addrs.false_val;
    }
}

fn to_str(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_int();
    if self_data == 0 {
        runtime
            .allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("False")))
    } else {
        runtime
            .allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("True")))
    }
}

fn repr(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_int();
    if self_data == 0 {
        runtime
            .allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("False")))
    } else {
        runtime
            .allocate_builtin_type_byname_raw("str", BuiltInTypeData::String(String::from("True")))
    }
}

fn to_boolean(_runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    //no-op
    return params.bound_pyobj.unwrap();
}

fn to_int(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_int();
    if self_data == 0 {
        runtime.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(0 as i128))
    } else {
        runtime.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(1 as i128))
    }
}

fn to_float(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
    check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
    let self_data = runtime
        .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
        .take_int();
    if self_data == 0 {
        runtime.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(0.0 as f64)))
    } else {
        runtime.allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(1.0 as f64)))
    }
}

macro_rules! create_unary_function {
    ($name:tt, $param_a:tt, $func:expr) => {
        fn $name(runtime: &mut Runtime, params: CallParams) -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
            let self_data = runtime
                .get_raw_data_of_pyobj(params.bound_pyobj.unwrap())
                .take_int();
            let $param_a = self_data;
            runtime.allocate_type_byaddr_raw(
                runtime.builtin_type_addrs.int,
                BuiltInTypeData::Int($func),
            )
        }
    };
}
create_unary_function!(negation, a, if a == 1 { 0 } else { 1 });

pub fn register_boolean_type(runtime: &mut Runtime) -> MemoryAddress {
    //bool inherits from int

    let int_supertype = runtime
        .find_in_module(BUILTIN_MODULE, "int")
        .expect("int type not found");
    let boolean_type = runtime.create_type(BUILTIN_MODULE, "bool", Some(int_supertype));

    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__and__", and_method);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__or__", or_method);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__xor__", xor_method);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__not__", not_method);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__neg__", negation);

    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__bool__", to_boolean);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__str__", to_str);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__repr__", repr);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__int__", to_int);
    runtime.register_bounded_func(BUILTIN_MODULE, "bool", "__float__", to_float);

    let true_value =
        runtime.allocate_builtin_type_byname_raw("bool", BuiltInTypeData::Int(1 as i128));
    let false_value =
        runtime.allocate_builtin_type_byname_raw("bool", BuiltInTypeData::Int(0 as i128));
    runtime.builtin_type_addrs.true_val = true_value;
    runtime.builtin_type_addrs.false_val = false_value;
    runtime.make_const(true_value);
    runtime.make_const(false_value);

    runtime.builtin_type_addrs.boolean = boolean_type;

    return boolean_type;
}
