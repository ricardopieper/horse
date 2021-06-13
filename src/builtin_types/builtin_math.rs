use crate::commons::float::Float;
use crate::runtime::vm::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;


fn create_function_1arg<FFloat>(
    vm: &VM,
    name: &str,
    op_float: FFloat,
) -> MemoryAddress
where
    FFloat: Fn(f64) -> f64 + 'static,
{
    let func = PyCallable {
        code: Box::new(move |vm, params| -> MemoryAddress {
            let call_params = params.as_function();
            check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
            let value_type_name = vm.get_pyobj_type_name(call_params.params[0]);
            let other_value = vm.get_raw_data_of_pyobj(call_params.params[0]);

            return match value_type_name {
                "int" => {
                    let parameter = other_value.take_int();
                    vm.allocate_builtin_type_byname_raw(
                        "float",
                        BuiltInTypeData::Float(Float((op_float)(parameter as f64))),
                    )
                }
                "float" => {
                    let other_float = other_value.take_float();
                    vm.allocate_builtin_type_byname_raw(
                        "float",
                        BuiltInTypeData::Float(Float((op_float)(other_float))),
                    )
                }
                _ => vm.special_values[&SpecialValue::NotImplementedValue],
            };
        }),
    };
    return vm.create_unbounded_callable_pyobj(func, Some(name.to_string()));
}

pub fn register_builtin_functions(vm: &mut VM) {
    let sin = create_function_1arg(vm, "sin", |f| f.sin());
    let cos = create_function_1arg(vm, "cos", |f| f.cos());
    let tanh = create_function_1arg(vm, "tanh", |f| f.tanh());

    vm.add_to_module(BUILTIN_MODULE, "sin", sin);
    vm.add_to_module(BUILTIN_MODULE, "cos", cos);
    vm.add_to_module(BUILTIN_MODULE, "tanh", tanh);
}
