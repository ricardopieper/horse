use crate::runtime::vm::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;

fn create_print_fn(vm: &VM) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |vm, params| -> MemoryAddress {
            let call_params = params.as_function();
            check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
            let (str_call_result, _) = vm
                .call_method(call_params.params[0], "__str__", PositionalParameters::empty())
                .unwrap();
            let str_raw = vm.get_raw_data_of_pyobj(str_call_result);
            match str_raw {
                BuiltInTypeData::String(s) => {
                    println!("{}", s);
                }
                _ => {
                    panic!("__str__ returned something else than string");
                }
            }
            return vm.special_values[&SpecialValue::NoneValue];
        }),
    };
    return vm.create_unbounded_callable_pyobj(func, Some("print".to_string()));
}

fn create_printstack_fn(vm: &VM) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |vm, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
            vm.print_stack();
            return vm.special_values[&SpecialValue::NoneValue];
        }),
    };
    return vm.create_unbounded_callable_pyobj(func, Some("printstack".to_string()));
}
fn create_traceback_fn(vm: &VM) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |vm, params| -> MemoryAddress {
            check_builtin_func_params!(params.func_name.unwrap(), 0, params.params.len());
            vm.print_traceback();
            return vm.special_values[&SpecialValue::NoneValue];
        }),
    };
    return vm.create_unbounded_callable_pyobj(func, Some("traceback".to_string()));
}
fn create_len_fn(vm: &VM) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |vm, params| -> MemoryAddress {
            let call_params = params.as_function();
            check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
            let (str_call_result, _) = vm
                .call_method(call_params.params[0], "__len__", PositionalParameters::empty())
                .unwrap();
            return str_call_result;
        }),
    };
    return vm.create_unbounded_callable_pyobj(func, Some("len".to_string()));
}

fn create_panic_fn(vm: &VM) -> MemoryAddress {
    let func = PyCallable {
        code: Box::new(move |vm, params| -> MemoryAddress {
            let call_params = params.as_function();
            check_builtin_func_params!(params.func_name.unwrap(), 1, call_params.params.len());
            let (str_call_result, _) = vm
                .call_method(call_params.params[0], "__str__", PositionalParameters::empty())
                .unwrap();
            let str_raw = vm.get_raw_data_of_pyobj(str_call_result);
            match str_raw {
                BuiltInTypeData::String(s) => {
                    panic!("{}", s);
                }
                _ => {
                    panic!("Program panicked with invalid argument passed to the panic function");
                }
            }
        }),
    };
    return vm.create_unbounded_callable_pyobj(func, Some("print".to_string()));
}

pub fn register_builtin_functions(vm: &mut VM) {
    let print_fn = create_print_fn(vm);
    let printstack_fn = create_printstack_fn(vm);
    let traceback_fn = create_traceback_fn(vm);
    let len_fn = create_len_fn(vm);
    let panic_fn = create_panic_fn(vm);
    vm.add_to_module(BUILTIN_MODULE, "print", print_fn);
    vm.add_to_module(BUILTIN_MODULE, "printstack", printstack_fn);
    vm.add_to_module(BUILTIN_MODULE, "traceback", traceback_fn);
    vm.add_to_module(BUILTIN_MODULE, "panic", panic_fn);
    vm.add_to_module(BUILTIN_MODULE, "len", len_fn);
}
