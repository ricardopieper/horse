mod lexer;
mod parser;
mod runtime;
mod builtin_types;
mod bytecode;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs;

fn main() {
    let interpreter = runtime::Interpreter::new();
    builtin_types::register_builtins(&interpreter);
    
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        let input = fs::read_to_string(args[1].clone()).expect(&format!("Could not read file {}", args[1]));
        let tokens = lexer::tokenize(input.as_str());
        let ast = parser::parse_ast(tokens.unwrap());
        let bytecode = bytecode::compiler::compile(ast);
        bytecode::instructions::execute_instructions(&interpreter, bytecode);
        return;
    }

    println!("Slowpython 0.0.1 (rustc {})", rustc_version_runtime::version());
    println!("No help, copyright or licensing commands available. You're on your own.");
       
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">>> ");
        match readline {
            Ok(input) => {
                rl.add_history_entry(input.as_str());
                if input == "\n" {
                    continue;
                }
                if input == "exit\n" {
                    return;
                }
                let tokens = lexer::tokenize(input.as_str());
                let ast = parser::parse_ast(tokens.unwrap());
                let bytecode = bytecode::compiler::compile(ast);
                bytecode::instructions::execute_instructions(&interpreter, bytecode);
                
                let result_addr = interpreter.top_stack();
                
                let result_string = interpreter.call_method(result_addr, "__repr__", vec![]);
                match result_string {
                    None => {},
                    Some(addr) => {
                        let pyobj_str = interpreter.get_raw_data_of_pyobj_opt::<String>(addr).unwrap();
                        println!("{}", pyobj_str);
                    }
                }
                
                interpreter.set_pc(0);

            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
