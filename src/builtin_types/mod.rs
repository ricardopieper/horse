use crate::runtime::Runtime;

pub mod int_type;
pub mod float_type;
pub mod boolean_type;
pub mod string_type;
pub mod builtin_math;
pub mod builtin_io;

pub fn register_builtins(runtime: &mut Runtime) {
    int_type::register_int_type(runtime);
    float_type::register_float_type(runtime);
    builtin_math::register_builtin_functions(runtime);
    builtin_io::register_builtin_functions(runtime);
    boolean_type::register_boolean_type(runtime);
    string_type::register_string_type(runtime);
}