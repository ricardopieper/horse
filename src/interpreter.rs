use crate::lexer::*;
use crate::parser::*;
use std::any::Any;

pub fn eval(expr: Expr) -> Box<dyn Any>
{
    match expr {
        Expr::IntegerValue(i) => {
            Box::new(i)
        },
        Expr::FloatValue(Float(f)) => {
            Box::new(f)
        },
        Expr::BinaryOperation(lhs, op, rhs) => {
            let mut integer_ops = true;
           

            let mut lhs_result: Box<dyn Any> = eval(*lhs);
            let mut rhs_result: Box<dyn Any> = eval(*rhs);

            //do i need to cast lhs to float?
            if lhs_result.is::<i128>() {
                if rhs_result.is::<f64>() {
                    lhs_result = Box::new(*lhs_result.downcast_ref::<i128>().unwrap() as f64);
                    integer_ops = false;
                }
            }

            //do i need to cast lhs to float?
            if rhs_result.is::<i128>() {
                if lhs_result.is::<f64>() {
                    rhs_result = Box::new(*rhs_result.downcast_ref::<i128>().unwrap() as f64);
                    integer_ops = false;
                }
            }

            match (integer_ops, op) {
                (true, Operator::Plus) => {
                    let lval = lhs_result.downcast_ref::<i128>().unwrap();
                    let rval = rhs_result.downcast_ref::<i128>().unwrap();
                    return Box::new(lval + rval);
                },
                (false, Operator::Plus) => {
                    let lval = lhs_result.downcast_ref::<f64>().unwrap();
                    let rval = rhs_result.downcast_ref::<f64>().unwrap();
                    return Box::new(lval + rval);
                }

                
                (true, Operator::Multiply) => {
                    let lval = lhs_result.downcast_ref::<i128>().unwrap();
                    let rval = rhs_result.downcast_ref::<i128>().unwrap();
                    return Box::new(lval * rval);
                },
                (false, Operator::Multiply) => {
                    let lval = lhs_result.downcast_ref::<f64>().unwrap();
                    let rval = rhs_result.downcast_ref::<f64>().unwrap();
                    return Box::new(lval * rval);
                }


                (true, Operator::Divide) => {
                    let lval = lhs_result.downcast_ref::<i128>().unwrap();
                    let rval = rhs_result.downcast_ref::<i128>().unwrap();
                    return Box::new(lval / rval);
                },
                (false, Operator::Divide) => {
                    let lval = lhs_result.downcast_ref::<f64>().unwrap();
                    let rval = rhs_result.downcast_ref::<f64>().unwrap();
                    return Box::new(lval / rval);
                }


                (true, Operator::Minus) => {
                    let lval = lhs_result.downcast_ref::<i128>().unwrap();
                    let rval = rhs_result.downcast_ref::<i128>().unwrap();
                    return Box::new(lval - rval);
                },
                (false, Operator::Minus) => {
                    let lval = lhs_result.downcast_ref::<f64>().unwrap();
                    let rval = rhs_result.downcast_ref::<f64>().unwrap();
                    return Box::new(lval - rval);
                },

                _ => {
                    unimplemented!();
                }
            }
        },
        _ => {
            unimplemented!();
        }
    }
}
