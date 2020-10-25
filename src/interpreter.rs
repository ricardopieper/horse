use crate::lexer::*;
use crate::parser::*;
use std::any::Any;

#[derive(Copy, Clone, Debug)]
enum InterpreterError {
    NumberConversionError,
    OperationOnUncoercedTypes,
}

#[derive(Copy, Clone, Debug)]
enum Number {
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

pub fn eval(expr: Expr) -> Box<dyn Any> {
    match expr {
        Expr::IntegerValue(i) => Box::new(i),
        Expr::FloatValue(Float(f)) => Box::new(f),
        Expr::BinaryOperation(lhs, op, rhs) => {
            let lhs_result: Number = Number::from_dyn(eval(*lhs)).unwrap();
            let rhs_result: Number = Number::from_dyn(eval(*rhs)).unwrap();

            let (lhs, rhs) = Number::coerce_types(&lhs_result, &rhs_result);

            return Number::run_operation(&lhs, &rhs, op).unwrap();
        }
        _ => {
            unimplemented!();
        }
    }
}
