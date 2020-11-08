use crate::runtime::Interpreter;

pub mod int_type;
pub mod float_type;
pub mod boolean_type;
pub mod string_type;
pub mod builtin_math;

pub fn register_builtins(interpreter: &Interpreter) {
    int_type::register_int_type(interpreter);
    float_type::register_float_type(interpreter);
    builtin_math::register_builtin_functions(interpreter);
    boolean_type::register_boolean_type(interpreter);
    string_type::register_string_type(interpreter);
}