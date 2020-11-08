use crate::bytecode::instructions::*;
use crate::lexer::*;
use crate::parser::*;

type Instrs = Vec<Instruction>;

fn compile_expr(expr: Expr) -> Vec<Instruction> {
    match expr {
        Expr::IntegerValue(i) => vec![Instruction::LoadConst(Const::Integer(i))],
        Expr::FloatValue(Float(f)) => vec![Instruction::LoadConst(Const::Float(f))],
        Expr::BooleanValue(b) => vec![Instruction::LoadConst(Const::Boolean(b))],
        Expr::StringValue(s) => vec![Instruction::LoadConst(Const::String(s))],
        Expr::BinaryOperation(lhs, op, rhs) => {
            let mut load_method: Instrs = match op {
                Operator::And => vec![Instruction::LoadMethod(String::from("__and__"))],
                Operator::Or => vec![Instruction::LoadMethod(String::from("__or__"))],
                Operator::Xor => vec![Instruction::LoadMethod(String::from("__xor__"))],
                Operator::Plus => vec![Instruction::LoadMethod(String::from("__add__"))],
                Operator::Minus => vec![Instruction::LoadMethod(String::from("__sub__"))],
                Operator::Multiply => vec![Instruction::LoadMethod(String::from("__mul__"))],
                Operator::Divide => vec![Instruction::LoadMethod(String::from("__truediv__"))],
                _ => panic!("operator not implemented: {:?}", op),
            };

            let mut lhs_bytecode: Instrs = compile_expr(*lhs);
            let mut rhs_bytecode: Instrs = compile_expr(*rhs);

            let call = Instruction::CallMethod {
                number_arguments: 1,
            };

            let mut final_instructions = vec![];
            final_instructions.append(&mut lhs_bytecode);
            final_instructions.append(&mut load_method);
            final_instructions.append(&mut rhs_bytecode);
            final_instructions.push(call);

            return final_instructions;
        }
        Expr::UnaryExpression(op, rhs) => {
            let mut load_method: Instrs = match op {
                Operator::Plus => vec![Instruction::LoadMethod(String::from("__pos__"))],
                Operator::Not => vec![Instruction::LoadMethod(String::from("__not__"))],
                Operator::Minus => vec![Instruction::LoadMethod(String::from("__neg__"))],
                _ => panic!("operator not implemented: {:?}", op),
            };

            let mut rhs_bytecode: Instrs = compile_expr(*rhs);
            let call = Instruction::CallMethod {
                number_arguments: 0,
            };

            let mut final_instructions = vec![];
            final_instructions.append(&mut rhs_bytecode);
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
                final_instructions.append(&mut compile_expr(param_expr));
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

pub fn compile(ast: Vec<AST>) -> Vec<Instruction> {
    let mut all_instructions = vec![];
    for ast_item in ast {
        match ast_item {
            AST::Assign {
                variable_name,
                expression,
            } => {
                all_instructions.append(&mut compile_expr(expression));
                all_instructions.push(Instruction::StoreName(variable_name));
            }
            AST::StandaloneExpr(expr) => return compile_expr(expr),
        }
    }

    return all_instructions;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin_types::*;
    use crate::runtime::*;

    #[test]
    fn test_literal_int_1() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("1").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.pop_stack());
        assert_eq!(stack_value, 1);
    }

    #[test]
    fn test_literal_float_1() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("1.0").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, 1.0);
    }

    #[test]
    fn test_literal_boolean_true() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("True").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.pop_stack());
        assert_eq!(stack_value, 1);
    }

    #[test]
    fn test_literal_boolean_false() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("False").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.pop_stack());
        assert_eq!(stack_value, 0);
    }

    #[test]
    fn test_1_plus_1() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("1 + 1").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.pop_stack());
        assert_eq!(stack_value, 2);
    }

    #[test]
    fn test_1_times_float_3_5() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("1 + 3.5").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, 4.5);
    }

    #[test]
    fn test_neg() {
        //-(5.0 / 9.0)
        let expected_result = -(5.0_f64 / 9.0_f64);
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("-(5.0 / 9.0)").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_div_neg_mul() {
        //-(5.0 / 9.0) * 32)
        let expected_result = -(5.0_f64 / 9.0_f64) * 32.0_f64;
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("-(5.0 / 9.0) * 32.0").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_div_minus_div() {
        //(1 - (5.0 / 9.0))
        let expected_result = 1.0_f64 - (5.0_f64 / 9.0_f64);
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("1.0 - (5.0 / 9.0)").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_fahrenheit() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let expected_result = (-(5.0_f64 / 9.0_f64) * 32.0_f64) / (1.0_f64 - (5.0_f64 / 9.0_f64));
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("(-(5.0 / 9.0) * 32.0) / (1.0 - (5.0 / 9.0))").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_function_calls_with_complex_expr() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let expected_result = (-(5.0_f64 / 9.0_f64) * 32.0_f64).sin().cos()
            / (1.0_f64.cos() - (5.0_f64 / 9.0_f64)).tanh();
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens =
            tokenize("cos(sin(-(5.0 / 9.0) * 32.0)) / tanh(cos(1.0) - (5.0 / 9.0))").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_fcall() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let expected_result = 1.0_f64.sin();
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("sin(1.0)").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_fcall_2params() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let expected_result = 1.0_f64 / 2.0_f64;
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("test(1.0, 2.0)").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>(interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_bind_local() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("x = 1 + 2").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.top_stack());
        assert_eq!(stack_value, 3);
    }

    #[test]
    fn test_string_concat() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("\"abc\" + 'cde'").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = interpreter.get_raw_data_of_pyobj::<String>(interpreter.top_stack());
        assert_eq!(stack_value, "abccde");
    }

    #[test]
    fn boolean_and() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("True and False").unwrap();
        let expr = parse_ast(tokens);
        let bytecode = compile(expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>(interpreter.top_stack());
        assert_eq!(stack_value, 0);
    }
}
