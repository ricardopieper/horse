mod interpreter;
mod lexer;
mod parser;
use std::io;
use std::io::Write;

fn main() {
    println!("Slowpython 0.0.1 (rustc 1.44.1 (c7087fe00 2020-06-17))");
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
        let result = interpreter::eval(ast);

        if let Some(f) = result.downcast_ref::<f64>() {
            println!("{:?}", f)
        }
        if let Some(i) = result.downcast_ref::<i128>() {
            println!("{:?}", i)
        }
    }
}
