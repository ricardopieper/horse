use crate::runtime::*;
use crate::memory::*;
use crate::datamodel::*;

fn create_print_fn(runtime: &Runtime) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |runtime, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
            let str_call_result = runtime
                .call_method(params.params[0], "__str__", &[])
                .unwrap();
            let str_raw = runtime.get_raw_data_of_pyobj(str_call_result);
            match str_raw {
                BuiltInTypeData::String(s) => {
                    println!("{}", s);
                }
                _ => {
                    panic!("__str__ returned something else than string");
                }
            }
            return runtime.special_values[&SpecialValue::NoneValue];
        }),
    };
    return runtime.create_callable_pyobj(func, Some("print".to_string()));
}

fn create_printstack_fn(runtime: &Runtime) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |runtime, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
            runtime.print_stack();
            return runtime.special_values[&SpecialValue::NoneValue];
        }),
    };
    return runtime.create_callable_pyobj(func, Some("printstack".to_string()));
}
fn create_traceback_fn(runtime: &Runtime) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |runtime, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
            runtime.print_traceback();
            return runtime.special_values[&SpecialValue::NoneValue];
        }),
    };
    return runtime.create_callable_pyobj(func, Some("traceback".to_string()));
}
fn create_len_fn(runtime: &Runtime) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |runtime, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 1, params.params.len());
            let str_call_result = runtime
                .call_method(params.params[0], "__len__", &[])
                .unwrap();
            return str_call_result;
        }),
    };
    return runtime.create_callable_pyobj(func, Some("len".to_string()));
}

pub fn register_builtin_functions(runtime: &mut Runtime) {
    let print_fn = create_print_fn(runtime);
    let printstack_fn = create_printstack_fn(runtime);
    let traceback_fn = create_traceback_fn(runtime);
    let create_len_fn = create_len_fn(runtime);
    runtime.add_to_module(BUILTIN_MODULE, "print", print_fn);
    runtime.add_to_module(BUILTIN_MODULE, "printstack", printstack_fn);
    runtime.add_to_module(BUILTIN_MODULE, "traceback", traceback_fn);
    runtime.add_to_module(BUILTIN_MODULE, "len", create_len_fn);
}
