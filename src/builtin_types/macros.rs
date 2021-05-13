macro_rules! check_builtin_func_params {
    ($name:expr, $expected:expr, $received:expr) => {
        if $expected != $received {
            panic!(
                "{}() expected {} arguments, got {}",
                $name, $expected, $received
            );
        }
    };
}
