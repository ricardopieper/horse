use crate::lexer::*;
use crate::parser::*;
use std::any::Any;
use std::fmt::Debug;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
enum InterpreterError {
    NumberConversionError,
    OperationOnUncoercedTypes,
    UnaryOperatorUnsupported(Operator)
}

#[derive(Copy, Clone, Debug)]
pub enum Number {
    Integer(i128),
    Float(f64),
}

impl Number {
    fn from_dyn(num: Box<dyn Any>) -> Result<Number, InterpreterError> {
        if num.is::<i128>() {
            return Ok(Number::Integer(*num.downcast_ref::<i128>().unwrap()));
        }
        if num.is::<f64>() {
            return Ok(Number::Float(*num.downcast_ref::<f64>().unwrap()));
        }
        return Err(InterpreterError::NumberConversionError);
    }

    fn coerce_types(n1: &Number, n2: &Number) -> (Number, Number) {
        match (n1, n2) {
            (Number::Integer(i), Number::Float(f)) => (Number::Float(*i as f64), Number::Float(*f)),
            (Number::Float(f), Number::Integer(i)) => (Number::Float(*f), Number::Float(*i as f64)),
            _ => (*n1, *n2),
        }
    }

    fn run_operation(
        n1: &Number,
        n2: &Number,
        op: Operator,
    ) -> Result<Box<dyn Any>, InterpreterError> {
        if let Operator::Divide = op {
            match (n1, n2) {
                (Number::Float(f1), Number::Float(f2)) => {
                    Ok(Box::new(Number::float_op(*f1, *f2, op)))
                }
                (Number::Integer(f1), Number::Integer(f2)) => {
                    Ok(Box::new(Number::float_op(*f1 as f64, *f2 as f64, op)))
                }
                _ => Err(InterpreterError::OperationOnUncoercedTypes),
            }
        } else {
            match (n1, n2) {
                (Number::Float(f1), Number::Float(f2)) => {
                    Ok(Box::new(Number::float_op(*f1, *f2, op)))
                }
                (Number::Integer(f1), Number::Integer(f2)) => {
                    Ok(Box::new(Number::integer_op(*f1, *f2, op)))
                }
                _ => Err(InterpreterError::OperationOnUncoercedTypes),
            }
        }
    }

    fn run_unary_operation(
        number: &Number,
        op: Operator
    ) -> Result<Box<dyn Any>, InterpreterError> {

        match (op, number) {
            (Operator::Plus, Number::Float(f)) => Ok(Box::new(*f)),
            (Operator::Plus, Number::Integer(i)) => Ok(Box::new(*i)),

            (Operator::Minus, Number::Float(f)) => Ok(Box::new(*f * -1.0)),
            (Operator::Minus, Number::Integer(i)) =>  Ok(Box::new(*i * -1)),
             
            _ => Err(InterpreterError::UnaryOperatorUnsupported(op))
        }

    }


    fn float_op(lhs: f64, rhs: f64, op: Operator) -> f64 {
        match op {
            Operator::Plus => lhs + rhs,
            Operator::Multiply => lhs * rhs,
            Operator::Divide => lhs / rhs,
            Operator::Minus => lhs - rhs,
            _ => unimplemented!(),
        }
    }

    fn integer_op(lhs: i128, rhs: i128, op: Operator) -> i128 {
        match op {
            Operator::Plus => lhs + rhs,
            Operator::Multiply => lhs * rhs,
            Operator::Divide => lhs / rhs,
            Operator::Minus => lhs - rhs,
            _ => unimplemented!(),
        }
    }
}

struct PyFunctionInvocationParams {
    named_params: HashMap<String, Box<dyn Any>>
    //kargs: Vec<>
}

struct PyFunction {
    name: String,
    param_names: Vec<String>,
    body: Box<dyn FnMut(PyFunctionInvocationParams) -> Box<dyn Any>>
}

impl PyFunction {
    fn apply(&mut self, params: PyFunctionInvocationParams) -> Box<dyn Any> {
        return (self.body)(params);
    }

    fn make_float_function<F>(name: String, param: String, fun: F) -> PyFunction where F: Fn(f64) -> f64 + 'static {
        PyFunction {
            name: name,
            param_names: vec![param],
            body: Box::new(move |mut params| -> Box<dyn Any> {
                let x = params.named_params.remove(&String::from("x")).unwrap();
                let x_number = Number::from_dyn(x);

                let as_float = match x_number.unwrap()  {
                    Number::Float(f) => f,
                    Number::Integer(i) => i as f64
                };
                
                return Box::new(fun(as_float))
            })
        }
    }

    fn make_float_to_int_function<F>(name: String, param: String, fun: F) -> PyFunction where F: Fn(f64) -> i128 + 'static {
        PyFunction {
            name: name,
            param_names: vec![param],
            body: Box::new(move |mut params| -> Box<dyn Any> {
                let x = params.named_params.remove(&String::from("x")).unwrap();
                let x_number = Number::from_dyn(x);

                let as_float = match x_number.unwrap()  {
                    Number::Float(f) => f,
                    Number::Integer(i) => i as f64
                };
                
                return Box::new(fun(as_float))
            })
        }
    }
}

pub fn eval_float(expr: Expr) -> f64 {
    let result = eval(expr);

    if let Some(f) = result.downcast_ref::<f64>() {
        return *f;
    }
    if let Some(i) = result.downcast_ref::<i128>() {
        return *i as f64;
    }

    panic!("not a number");
}

pub fn eval(expr: Expr) -> Box<dyn Any> {

    let mut all_functions = HashMap::new();

    let sin_f = PyFunction::make_float_function(String::from("sin"), String::from("x"), |x| x.sin());
    let cos_f = PyFunction::make_float_function(String::from("cos"), String::from("x"), |x| x.cos());
    let tan_f = PyFunction::make_float_function(String::from("tan"), String::from("x"), |x| x.tan());
    let float_f = PyFunction::make_float_function(String::from("float"), String::from("x"), |x| x);
    let int_f = PyFunction::make_float_to_int_function(String::from("int"), String::from("x"), |x| x as i128);

    for f in vec![sin_f, cos_f, tan_f, float_f, int_f] {
        all_functions.insert(f.name.clone(), f);
    }

    match expr {
        Expr::IntegerValue(i) => Box::new(i),
        Expr::FloatValue(Float(f)) => Box::new(f),
        Expr::BinaryOperation(lhs, op, rhs) => {
            let lhs_result: Number = Number::from_dyn(eval(*lhs)).unwrap();
            let rhs_result: Number = Number::from_dyn(eval(*rhs)).unwrap();
            let (lhs, rhs) = Number::coerce_types(&lhs_result, &rhs_result);
            return Number::run_operation(&lhs, &rhs, op).unwrap();
        }
        Expr::UnaryExpression(op, rhs) => {
            let rhs_result: Number = Number::from_dyn(eval(*rhs)).unwrap();
            let result = Number::run_unary_operation(&rhs_result, op).unwrap();
            return result;
        },
        Expr::FunctionCall(fname, params) => {
            //setup order of params

            let mut evaluated_params = vec![];
            for p in params {
                evaluated_params.push(eval(p));
            }

            let mut param_function = HashMap::<String, Box<dyn Any>>::new();
            let function_definition = all_functions.get_mut(&fname).unwrap();

            for (p, name) in evaluated_params.into_iter().zip(function_definition.param_names.iter()) {
                param_function.insert(name.clone(), p);
            }

            let invoke = PyFunctionInvocationParams {
                named_params: param_function
            };

            return function_definition.apply(invoke);
        }
        _ => {
            unimplemented!();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn run_1_plus_1() {
        let tokens = tokenize("1 + 1").unwrap();
        let expr = parse(tokens);
        let result = eval_float(expr);
        
        assert_eq!(result, 2.0);
    }

    #[test]
    fn run_sin1() {
        let tokens = tokenize("sin(1)").unwrap();
        let expr = parse(tokens);
        let result = eval_float(expr);
        
        assert_eq!(result, (1.0_f64).sin());
    }

    #[test]
    fn run_complex_expr() {
        let tokens = tokenize("((sin(1) * -(cos(2.0001)) + tan(1e-2)))").unwrap();
        let expr = parse(tokens);
        let result = eval_float(expr);
        
        assert_eq!(result, (1.0_f64).sin() * -(2.0001_f64).cos() + (1e-2_f64).tan());
    }

    #[test]
    fn int_truncate_expr() {
        let tokens = tokenize("int(2.222333)").unwrap();
        let expr = parse(tokens);
        let result = eval_float(expr);
        
        assert_eq!(result, 2.0);
    }
}