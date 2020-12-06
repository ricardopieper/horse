use crate::bytecode::program::*;
use crate::lexer::*;
use crate::parser::*;

use std::collections::HashMap;

fn process_constval(constval: Const, const_map: &mut HashMap<Const, usize>) -> Vec<Instruction> {
   let loadconst_idx = if !const_map.contains_key(&constval) {
        let len = const_map.len();
        println!("Storing const {:?} with key {:?}", constval, len);
        const_map.insert(constval, len);
        len
    } else {
        *const_map.get(&constval).unwrap()
    };

    return vec![Instruction::LoadConst(loadconst_idx)];
}

fn compile_expr(expr: Expr, const_map: &mut HashMap<Const, usize>) -> Vec<Instruction> {
    match expr {
        Expr::IntegerValue(i) => {
            let constval = Const::Integer(i);
            return process_constval(constval, const_map);
        },
        Expr::FloatValue(f) => {
            let constval = Const::Float(f);
            return process_constval(constval, const_map);
        },
        Expr::BooleanValue(b) => {
            let constval = Const::Boolean(b);
            return process_constval(constval, const_map);
        },
        Expr::StringValue(s) => {
            let constval = Const::String(s);
            return process_constval(constval, const_map);
        },
        Expr::BinaryOperation(lhs, op, rhs) => {
            let mut load_method: Vec<Instruction> = match op {
                Operator::And => vec![Instruction::LoadMethod(String::from("__and__"))],
                Operator::Or => vec![Instruction::LoadMethod(String::from("__or__"))],
                Operator::Xor => vec![Instruction::LoadMethod(String::from("__xor__"))],
                Operator::Plus => vec![Instruction::LoadMethod(String::from("__add__"))],
                Operator::Mod => vec![Instruction::LoadMethod(String::from("__mod__"))],
                Operator::Minus => vec![Instruction::LoadMethod(String::from("__sub__"))],
                Operator::Multiply => vec![Instruction::LoadMethod(String::from("__mul__"))],
                Operator::Divide => vec![Instruction::LoadMethod(String::from("__truediv__"))],
                Operator::Equals => vec![Instruction::LoadMethod(String::from("__eq__"))],
                Operator::NotEquals => vec![Instruction::LoadMethod(String::from("__ne__"))],
                Operator::Greater => vec![Instruction::LoadMethod(String::from("__gt__"))],
                Operator::GreaterEquals => vec![Instruction::LoadMethod(String::from("__ge__"))],
                Operator::Less => vec![Instruction::LoadMethod(String::from("__lt__"))],
                Operator::LessEquals => vec![Instruction::LoadMethod(String::from("__le__"))],
                _ => panic!("operator not implemented: {:?}", op),
            };

            let mut lhs_program: Vec<Instruction> = compile_expr(*lhs, const_map);
            let mut rhs_program: Vec<Instruction> = compile_expr(*rhs, const_map);

            let call = Instruction::CallMethod {
                number_arguments: 1,
            };

            let mut final_instructions = vec![];
            final_instructions.append(&mut lhs_program);
            final_instructions.append(&mut load_method);
            final_instructions.append(&mut rhs_program);
            final_instructions.push(call);

            return final_instructions;
        }
        Expr::UnaryExpression(op, rhs) => {
            let mut load_method: Vec<Instruction> = match op {
                Operator::Plus => vec![Instruction::LoadMethod(String::from("__pos__"))],
                Operator::Not => vec![Instruction::LoadMethod(String::from("__not__"))],
                Operator::Minus => vec![Instruction::LoadMethod(String::from("__neg__"))],
                _ => panic!("operator not implemented: {:?}", op),
            };

            let mut rhs_program: Vec<Instruction> = compile_expr(*rhs, const_map);
            let call = Instruction::CallMethod {
                number_arguments: 0,
            };

            let mut final_instructions = vec![];
            final_instructions.append(&mut rhs_program);
            final_instructions.append(&mut load_method);
            final_instructions.push(call);

            return final_instructions;
        }
        Expr::FunctionCall(fname, params) => {
            //setup order of params

            let mut final_instructions = vec![];
            final_instructions.push(Instruction::LoadFunction(fname.to_string()));

            let len_params = params.len();
            for param_expr in params {
                final_instructions.append(&mut compile_expr(param_expr, const_map));
            }

            final_instructions.push(Instruction::CallFunction {
                number_arguments: len_params,
            });
            return final_instructions;
        }
        Expr::Variable(var_name) => vec![Instruction::LoadName(var_name)],
        Expr::Parenthesized(_) => panic!("Parenthesized expr should not leak to compiler"),
    }
}

struct ConstAndIndex {
    constval: Const,
    index: usize
}

pub fn compile(ast: Vec<AST>) -> Program {
    let mut const_values_and_indices = HashMap::new();
    let instructions = compile_ast(ast, 0, &mut const_values_and_indices);

    let mut vec_const = vec![];
    for (constval, index) in const_values_and_indices {
        vec_const.push(ConstAndIndex {
            constval,
            index
        })
    }
    vec_const.sort_unstable_by(|a, b| a.index.cmp(&b.index));

    Program {
        data: vec_const.into_iter().map(|x| x.constval).collect(),
        code: instructions
    }
}

pub fn compile_ast(ast: Vec<AST>, offset: usize, 
    const_map: &mut HashMap<Const, usize>) -> Vec<Instruction> {
    let mut all_instructions = vec![];
    
    for ast_item in ast {
        match ast_item {
            AST::Assign {
                variable_name,
                expression,
            } => {
                all_instructions.append(&mut compile_expr(expression, const_map));
                all_instructions.push(Instruction::StoreName(variable_name));
            }
            AST::StandaloneExpr(expr) => {
                all_instructions.append(&mut compile_expr(expr, const_map));
            }
            AST::IfStatement {
                true_branch,
                elifs: _,
                final_else,
            } => {
                let mut if_expr_compiled = compile_expr(true_branch.expression, const_map);
                all_instructions.append(&mut if_expr_compiled);

                //+1 is because there will be a instruction before
                //that will do the jump
                let offset_before_if = offset + all_instructions.len() + 1;

                let mut true_branch_compiled =
                    compile_ast(true_branch.statements, offset_before_if, const_map);
                //generate a jump to the code right after the true branch

                //if there is an else: statement, the true branch must jump to after the false branch
                if let Some(else_ast) = final_else {
                    //+1 because where will be a jump unconditional that is *part* of the true branch

                    let offset_after_true_branch =
                        offset_before_if + true_branch_compiled.len() + 1;
                    all_instructions.push(Instruction::JumpIfFalseAndPopStack(
                        offset_after_true_branch,
                    ));
                    all_instructions.append(&mut true_branch_compiled);

                    let mut false_branch_compiled = compile_ast(else_ast, offset_after_true_branch, const_map);

                    //+1 because there will be an instruction
                    //in the true branch that will jump to *after* the false branch
                    let offset_after_else =
                        offset_after_true_branch + false_branch_compiled.len() + 1;
                    all_instructions.push(Instruction::JumpUnconditional(offset_after_else));
                    all_instructions.append(&mut false_branch_compiled);
                } else {
                    let offset_after_true_branch = offset_before_if + true_branch_compiled.len();
                    all_instructions.push(Instruction::JumpIfFalseAndPopStack(
                        offset_after_true_branch,
                    ));
                    all_instructions.append(&mut true_branch_compiled);
                }
            }
            AST::WhileStatement { expression, body } => {
                let offset_before_while = all_instructions.len() + offset;
                let mut compiled_expr = compile_expr(expression, const_map);
                //+1 for the jump if false
                let offset_after_expr = all_instructions.len() + compiled_expr.len() + 1;
                let compiled_body = compile_ast(body, offset_after_expr, const_map);
                all_instructions.append(&mut compiled_expr);
                let offset_after_body = offset_after_expr + compiled_body.len() + 1;
                all_instructions.push(Instruction::JumpIfFalseAndPopStack(offset_after_body));

                let mut compiled_body_with_resolved_breaks: Vec<Instruction> = compiled_body
                    .into_iter()
                    .map(|instr| -> Instruction {
                        if let Instruction::UnresolvedBreak = instr {
                            Instruction::JumpUnconditional(offset_after_body)
                        } else {
                            instr
                        }
                    })
                    .collect();

                all_instructions.append(&mut compiled_body_with_resolved_breaks);
                all_instructions.push(Instruction::JumpUnconditional(offset_before_while));
            }
            AST::Break => {
                //In python there's something called a "block stack" and an opcode called POP_BLOCK
                //that makes this much easier, as well as a BREAK_LOOP instruction that uses block information
                //to break the current loop.
                //So Python really has a loooot of information about high-level language features even in the 
                //lower level layers...
                //But for me it's a more interesting problem to not use these instructions and just use plain jumps. 
                //However, when I find a break in the AST, I don't yet know what the program will look like,
                //and therefore I don't know where to jump. 
                //Perhaps other features such as generators, for comprehensions, etc really need blocks? I doubt it.
                all_instructions.push(Instruction::UnresolvedBreak);
            }
            //_ => panic!("Instruction not covered: {:?}", ast_item)
        }
    }
 
    return all_instructions;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin_types::*;
    use crate::runtime::*;
    use crate::bytecode::interpreter;

    #[test]
    fn while_statements() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize(
            "
x = 0
y = 0
while x < 10:
    y = y + 1
    x = x + 1
",
        )
        .unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let y_value = interpreter.get_local("y").unwrap();
        let raw_y = interpreter.get_raw_data_of_pyobj::<i128>(y_value);
        assert_eq!(*raw_y, 10);
    }

    #[test]
    fn while_statements_with_conditional_break() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize(
            "
x = 0
y = 0
while x < 10:
    y = y + 1
    x = x + 1
    if x == 5:
        break
",
        )
        .unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let y_value = interpreter.get_local("y").unwrap();
        let raw_y = interpreter.get_raw_data_of_pyobj::<i128>(y_value);
        assert_eq!(*raw_y, 5);
    }

    #[test]
    fn if_else_statements() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize(
            "
x = 999
y = 1
if x == 0:
    y = 2
else:
    y = 3
",
        )
        .unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let y_value = interpreter.get_local("y").unwrap();
        let raw_y = interpreter.get_raw_data_of_pyobj::<i128>(y_value);
        assert_eq!(*raw_y, 3);
    }

    #[test]
    fn test_literal_int_1() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("1").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.pop_stack());
        assert_eq!(stack_value, 1);
    }

    #[test]
    fn test_literal_float_1() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("1.0").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, 1.0);
    }

    #[test]
    fn test_literal_boolean_true() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("True").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.pop_stack());
        assert_eq!(stack_value, 1);
    }

    #[test]
    fn test_literal_boolean_false() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("False").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.pop_stack());
        assert_eq!(stack_value, 0);
    }

    #[test]
    fn test_1_plus_1() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("1 + 1").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.pop_stack());
        assert_eq!(stack_value, 2);
    }

    #[test]
    fn test_1_plus_float_3_5() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("1 + 3.5").unwrap();
        let expr = parse_ast(tokens);
        println!("AST: {:?}", expr);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, 4.5);
    }

    #[test]
    fn test_neg() {
        //-(5.0 / 9.0)
        let expected_result = -(5.0_f64 / 9.0_f64);
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("-(5.0 / 9.0)").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_div_neg_mul() {
        //-(5.0 / 9.0) * 32)
        let expected_result = -(5.0_f64 / 9.0_f64) * 32.0_f64;
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("-(5.0 / 9.0) * 32.0").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_div_minus_div() {
        //(1 - (5.0 / 9.0))
        let expected_result = 1.0_f64 - (5.0_f64 / 9.0_f64);
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("1.0 - (5.0 / 9.0)").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_fahrenheit() {
        let expected_result = (-(5.0_f64 / 9.0_f64) * 32.0_f64) / (1.0_f64 - (5.0_f64 / 9.0_f64));
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("(-(5.0 / 9.0) * 32.0) / (1.0 - (5.0 / 9.0))").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_function_calls_with_complex_expr() {
        let expected_result = (-(5.0_f64 / 9.0_f64) * 32.0_f64).sin().cos()
            / (1.0_f64.cos() - (5.0_f64 / 9.0_f64)).tanh();
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens =
            tokenize("cos(sin(-(5.0 / 9.0) * 32.0)) / tanh(cos(1.0) - (5.0 / 9.0))").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_fcall() {
        let expected_result = 1.0_f64.sin();
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("sin(1.0)").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_fcall_2params() {
        let expected_result = 1.0_f64 / 2.0_f64;
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("test(1.0, 2.0)").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_bind_local() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("x = 1 + 2").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.get_local("x").unwrap());
        assert_eq!(stack_value, 3);
    }

    #[test]
    fn test_string_concat() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("\"abc\" + 'cde'").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = interpreter.get_raw_data_of_pyobj::<String>(interpreter.top_stack());
        assert_eq!(stack_value, "abccde");
    }

    #[test]
    fn boolean_and() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let tokens = tokenize("True and False").unwrap();
        let expr = parse_ast(tokens);
        let program =  compile(expr);
        interpreter::execute_program(&interpreter, program);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.top_stack());
        assert_eq!(stack_value, 0);
    }
}
