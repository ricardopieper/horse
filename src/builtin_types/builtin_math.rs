use crate::float::Float;
use crate::runtime::*;
use crate::memory::*;
use crate::datamodel::*;


fn create_function_1arg<FFloat>(
    runtime: &Runtime,
    name: &str,
    op_float: FFloat,
) -> MemoryAddress
where
    FFloat: Fn(f64) -> f64 + 'static,
{
    let func = PyCallable {
        code: Box::new(move |runtime, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
            let value_type_name = runtime.get_pyobj_type_name(params.params[0]);
            let other_value = runtime.get_raw_data_of_pyobj(params.params[0]);

            return match value_type_name {
                "int" => {
                    let parameter = other_value.take_int();
                    runtime.allocate_builtin_type_byname_raw(
                        "float",
                        BuiltInTypeData::Float(Float((op_float)(parameter as f64))),
                    )
                }
                "float" => {
                    let other_float = other_value.take_float();
                    runtime.allocate_builtin_type_byname_raw(
                        "float",
                        BuiltInTypeData::Float(Float((op_float)(other_float))),
                    )
                }
                _ => runtime.special_values[&SpecialValue::NotImplementedValue],
            };
        }),
    };
    return runtime.create_callable_pyobj(func, Some(name.to_string()));
}

pub fn register_builtin_functions(runtime: &mut Runtime) {
    let sin = create_function_1arg(runtime, "sin", |f| f.sin());
    let cos = create_function_1arg(runtime, "cos", |f| f.cos());
    let tanh = create_function_1arg(runtime, "tanh", |f| f.tanh());

    runtime.add_to_module(BUILTIN_MODULE, "sin", sin);
    runtime.add_to_module(BUILTIN_MODULE, "cos", cos);
    runtime.add_to_module(BUILTIN_MODULE, "tanh", tanh);
}
