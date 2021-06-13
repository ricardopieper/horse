use crate::runtime::vm::*;

#[macro_use]
pub mod macros;
pub mod boolean_type;
pub mod builtin_functions;
pub mod builtin_math;
pub mod float_type;
pub mod int_type;
pub mod list_type;
pub mod string_type;
pub mod index_error;
pub mod code_object;
pub mod loader;
pub mod none_type;

pub fn register_builtins(vm: &mut VM) {
    int_type::register_int_type(vm);
    float_type::register_float_type(vm);
    builtin_math::register_builtin_functions(vm);
    builtin_functions::register_builtin_functions(vm);
    boolean_type::register_boolean_type(vm);
    string_type::register_string_type(vm);
    list_type::register_list_type(vm);
    index_error::register_indexerr_type(vm);
    code_object::register_codeobject_type(vm);
    none_type::register_none_type_methods(vm);
}
