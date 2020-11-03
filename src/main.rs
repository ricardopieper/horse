mod lexer;
mod parser;
mod runtime;
mod builtin_types;
mod bytecode;
use std::io;
use std::io::Write;

fn main() {
    println!("Slowpython 0.0.1 (rustc {})", rustc_version_runtime::version());
    println!("No help, copyright or licensing commands available. You're on your own.");

    loop {
        let mut input = String::new();
        print!(">>> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        if input == "\n" {
            continue;
        }
        if input == "exit\n" {
            return;
        }

        let tokens = lexer::tokenize(input.as_str());
        let ast = parser::parse(tokens.unwrap());
        let bytecode = bytecode::compiler::compile(&ast);
        let interpreter = runtime::Interpreter::new();
        builtin_types::register_builtins(&interpreter);
        bytecode::instructions::execute_instructions(&interpreter, bytecode);
        
        let result_addr = interpreter.pop_stack();
        
        if let Some(f) = interpreter.get_raw_data_of_pyobj_opt::<f64>(result_addr) {
            println!("{:?}", f)
        }
        if let Some(i) = interpreter.get_raw_data_of_pyobj_opt::<i128>(result_addr) {
            println!("{:?}", i)
        }
    }
}
