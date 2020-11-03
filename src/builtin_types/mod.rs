use crate::runtime::Interpreter;

pub mod int_type;
pub mod float_type;
pub mod builtin_math;

pub fn register_builtins(interpreter: &Interpreter) {
    int_type::register_int_type(interpreter);
    float_type::register_float_type(interpreter);
    builtin_math::register_builtin_functions(interpreter);
}