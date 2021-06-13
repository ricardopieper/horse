use crate::runtime::vm::*;
use crate::ast::lexer;
use crate::ast::parser;
use crate::bytecode::compiler::*;
use crate::runtime::interpreter;

pub fn run_loader(vm: &mut VM) {

    //bool inherits from int
    for entry in std::fs::read_dir("./stdlib/__builtins__").unwrap() {
        let dir = entry.unwrap();
        //println!("Loading source {:?}", dir.path());
        let source = std::fs::read_to_string(dir.path());
        let tokens = lexer::tokenize(&source.unwrap()).unwrap();
        let expr = parser::parse_ast(tokens);
        let program = compile(expr);
        interpreter::execute_program(vm, program);
        vm.clear_stacks();
    }
    
}
