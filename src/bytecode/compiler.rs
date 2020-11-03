use crate::parser::*;
use crate::lexer::*;
use crate::bytecode::instructions::*;

type Instrs = Vec<Instruction>;

pub fn compile(expr: &Expr) -> Vec<Instruction> {
    match expr {
        Expr::IntegerValue(i) => vec![Instruction::LoadConst(Const::Integer(*i))],
        Expr::FloatValue(Float(f)) => vec![Instruction::LoadConst(Const::Float(*f))],
        Expr::BinaryOperation(lhs, op, rhs) => {

            let mut load_method: Instrs = match op {
                Operator::Plus => vec![Instruction::LoadMethod(String::from("__add__"))],
                Operator::Minus => vec![Instruction::LoadMethod(String::from("__sub__"))],
                Operator::Multiply => vec![Instruction::LoadMethod(String::from("__mul__"))],
                Operator::Divide => vec![Instruction::LoadMethod(String::from("__truediv__"))],
                _ => panic!("operator not implemented: {:?}", op)
            };

            let mut lhs_bytecode: Instrs = compile(lhs);
            let mut rhs_bytecode: Instrs = compile(rhs);

            let call = Instruction::CallMethod{ number_arguments: 1};
            
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
                Operator::Minus => vec![Instruction::LoadMethod(String::from("__neg__"))],
                _ => panic!("operator not implemented: {:?}", op)
            };

            let mut rhs_bytecode: Instrs = compile(rhs);
            let call = Instruction::CallMethod{ number_arguments: 0};

            let mut final_instructions = vec![];
            final_instructions.append(&mut rhs_bytecode);
            final_instructions.append(&mut load_method);
            final_instructions.push(call);

            return final_instructions;
        },
        Expr::FunctionCall(fname, params) => {
            //setup order of params
           
            let mut final_instructions = vec![];
            final_instructions.push(Instruction::LoadFunction(fname.to_string()));

            for param_expr in params.iter() {
                final_instructions.append(&mut compile(param_expr));
            }

            final_instructions.push(Instruction::CallFunction { number_arguments: params.len() });
            return final_instructions;
        }
        _ => {
            unimplemented!();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::*;
    use crate::builtin_types::*;

    #[test]
    fn test_1_plus_1() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("1 + 1").unwrap();
        let expr = parse(tokens);
        let bytecode = compile(&expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<i128>( interpreter.pop_stack());
        assert_eq!(stack_value, 2);
    }

    #[test]
    fn test_1_times_float_3_5() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("1 + 3.5").unwrap();
        let expr = parse(tokens);
        let bytecode = compile(&expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>( interpreter.pop_stack());
        assert_eq!(stack_value, 4.5);
    }


    #[test]
    fn test_neg() {
        //-(5.0 / 9.0)
        let expected_result = -(5.0_f64 / 9.0_f64);
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("-(5.0 / 9.0)").unwrap();
        let expr = parse(tokens);
        let bytecode = compile(&expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>( interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_div_neg_mul() {
        //-(5.0 / 9.0) * 32)
        let expected_result = -(5.0_f64 / 9.0_f64) * 32.0_f64;
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("-(5.0 / 9.0) * 32.0").unwrap();
        let expr = parse(tokens);
        let bytecode = compile(&expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>( interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_div_minus_div() {
        //(1 - (5.0 / 9.0))
        let expected_result = 1.0_f64 - (5.0_f64 / 9.0_f64);
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("1.0 - (5.0 / 9.0)").unwrap();
        let expr = parse(tokens);
        let bytecode = compile(&expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>( interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_fahrenheit() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let expected_result = (-(5.0_f64 / 9.0_f64) * 32.0_f64) / (1.0_f64 - (5.0_f64 / 9.0_f64));
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("(-(5.0 / 9.0) * 32.0) / (1.0 - (5.0 / 9.0))").unwrap();
        let expr = parse(tokens);
        let bytecode = compile(&expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>( interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_function_calls_with_complex_expr() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let expected_result = (-(5.0_f64 / 9.0_f64) * 32.0_f64).sin().cos() / (1.0_f64.cos() - (5.0_f64 / 9.0_f64)).tanh();
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("cos(sin(-(5.0 / 9.0) * 32.0)) / tanh(cos(1.0) - (5.0 / 9.0))").unwrap();
        let expr = parse(tokens);
        let bytecode = compile(&expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>( interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_fcall() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let expected_result = 1.0_f64.sin();
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("sin(1.0)").unwrap();
        let expr = parse(tokens);
        let bytecode = compile(&expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>( interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }

    #[test]
    fn test_fcall_2params() {
        //(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))
        let expected_result = 1.0_f64 / 2.0_f64;
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let tokens = tokenize("test(1.0, 2.0)").unwrap();
        let expr = parse(tokens);
        let bytecode = compile(&expr);
        execute_instructions(&interpreter, bytecode);
        let stack_value = *interpreter.get_raw_data_of_pyobj::<f64>( interpreter.pop_stack());
        assert_eq!(stack_value, expected_result);
    }
}