use crate::runtime::runtime::*;

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

pub fn register_builtins(runtime: &mut Runtime) {
    int_type::register_int_type(runtime);
    float_type::register_float_type(runtime);
    builtin_math::register_builtin_functions(runtime);
    builtin_functions::register_builtin_functions(runtime);
    boolean_type::register_boolean_type(runtime);
    string_type::register_string_type(runtime);
    list_type::register_list_type(runtime);
    index_error::register_indexerr_type(runtime);
    code_object::register_codeobject_type(runtime);
    none_type::register_none_type_methods(runtime);
    loader::run_loader(runtime);
}
