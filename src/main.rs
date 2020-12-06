mod lexer;
mod parser;
#[macro_use]
mod runtime;
mod builtin_types;
mod bytecode;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs;

fn main() {
    let mut interpreter = runtime::Interpreter::new();
    builtin_types::register_builtins(&mut interpreter);
    
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        let input = fs::read_to_string(args[1].clone()).expect(&format!("Could not read file {}", args[1]));
        let tokens = lexer::tokenize(input.as_str());
        let ast = parser::parse_ast(tokens.unwrap());
        let bytecode = bytecode::compiler::compile(ast);
        bytecode::instructions::execute_instructions(&interpreter, bytecode);

        /*
        let memory = interpreter.memory.memory.into_inner();
        for (index, data) in memory.into_iter().enumerate() {
            let cell = data.data.into_inner();

            if let Some(integer) = cell.downcast_ref::<i128>() {
                println!("Data at index {} is a raw int {:?}", index, integer);
            }
            else if let Some(float) = cell.downcast_ref::<f64>() {
                println!("Data at index {} is a raw float {:?}", index, float);
            }
            else if let Some(boolean) = cell.downcast_ref::<bool>() {
                println!("Data at index {} is a raw bool {:?}", index, boolean);
            }
            else if let Some(py_obj) = cell.downcast_ref::<runtime::PyObject>() {
                println!("Data at index {} is a PyObject {:?}", index, py_obj);
            }
            else {
                println!("Data at index {} is unknown, might be a function lambda", index);
            }
        }*/

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
