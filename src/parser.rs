use crate::lexer::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    IntegerValue(i128),
    FloatValue(Float),
    BinaryOperation(Box<Expr>, Operator, Box<Expr>),
    Parenthesized(Box<Expr>)
}

impl Expr {
    fn new_int(i: i128) -> Box<Self> {
        Box::new(Self::IntegerValue(i))
    }
    fn new_float(f: Float) -> Box<Self> {
        Box::new(Self::FloatValue(f))
    }
}

impl From<f64> for Box<Expr> {
    fn from(w: f64) -> Self {
        Expr::new_float(w.into())
    }
}

impl From<i128> for Box<Expr> {
    fn from(w: i128) -> Self {
        Expr::new_int(w.into())
    }
}

fn precedence(o: Operator) -> u32 {
    match o {
        Operator::Multiply => 100,
        Operator::Divide => 100,
        _ => 1
    }
}

pub fn parse(tokens: Vec<Token>) -> Expr {

    let mut operator_stack: Vec<Operator> = vec![];
    let mut operand_stack: Vec<Expr> = vec![];

    for tok in tokens {
        let mut was_operand = false;
        match tok {
            Token::LiteralInteger(i) => {
                operand_stack.push(Expr::IntegerValue(i));
                was_operand = true;
            },
            Token::LiteralFloat(f) => {
                operand_stack.push(Expr::FloatValue(f));
                was_operand = true;
            },
            Token::Operator(o) => {
                operator_stack.push(o)
            }
        }

        if was_operand {
            if operand_stack.len() > 1 && !operator_stack.is_empty() {

                let rhs_root = operand_stack.pop().unwrap();
                let lhs_root = operand_stack.pop().unwrap();
                let op = operator_stack.pop().unwrap();

                let mut bin_op = Expr::BinaryOperation(
                    Box::new(lhs_root.clone()), 
                    op, 
                    Box::new(rhs_root.clone()));
                
                if let Expr::BinaryOperation(lhs_down, op_down, rhs_down) = &lhs_root {

                    let precedence_down = precedence(*op_down);
                    let precedence_root = precedence(op);
                    
                    if precedence_root > precedence_down {
                        bin_op = Expr::BinaryOperation(
                            lhs_down.clone(),
                            *op_down,
                            Box::new(
                                Expr::BinaryOperation(
                                    rhs_down.clone(), 
                                    op, 
                                    Box::new(rhs_root.clone())))
                        );
                    }
                } 
                
                if let Expr::BinaryOperation(lhs_down, op_down, rhs_down) = &rhs_root {

                    let precedence_down = precedence(*op_down);
                    let precedence_root = precedence(op);
                    
                    if precedence_root > precedence_down {
                        bin_op = Expr::BinaryOperation(
                            lhs_down.clone(),
                            *op_down,
                            Box::new(
                                Expr::BinaryOperation(
                                    rhs_down.clone(), 
                                    op, 
                                    Box::new(lhs_root.clone())))
                        );
                    }
                } 
                
                operand_stack.push(bin_op);
            }
        }
    }

    if !operator_stack.is_empty() {
        panic!("Unparsed operators: {:?}, operands = {:?}", operator_stack, operand_stack);
    }

    if operand_stack.len() > 1 {
        panic!("Unparsed operands: {:?}", operand_stack);
    }

    if operand_stack.is_empty() {
        panic!("Empty operand stack, didn't parse anything");
    }

    return operand_stack.pop().unwrap()
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_1_plus_1()  {
        let result = parse(vec![
            Token::LiteralInteger(1), 
            Token::Operator(Operator::Plus), 
            Token::LiteralInteger(1)]);
        
        let expected = Expr::BinaryOperation(
            1.into(),
            Operator::Plus,
            1.into(),
        );

        assert_eq!(result, expected)
    }

    #[test]
    fn parse_float_10_plus_integer_1()  {
        let result = parse(vec![
            Token::LiteralFloat(10.0.into()), 
            Token::Operator(Operator::Plus), 
            Token::LiteralInteger(1)]);
      
         
        let expected = Expr::BinaryOperation(
            10.0.into(),
            Operator::Plus,
            1.into(),
        );

        assert_eq!(result, expected);
    }


    #[test]
    fn parse_float_3_values_2_ops() {
        let result = parse(vec![ 
            Token::LiteralInteger(1), 
            Token::Operator(Operator::Plus), 
            Token::LiteralInteger(2),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(3)]);
      
        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(
                1.into(),
                Operator::Plus,
                2.into(),
            )), 
            Operator::Plus,
            3.into());

        assert_eq!(result, expected);
    }


    #[test]
    fn parse_multiplication() {
        let result = parse(vec![ 
            Token::LiteralInteger(1), 
            Token::Operator(Operator::Multiply), 
            Token::LiteralInteger(2)]);
              
        let expected = Expr::BinaryOperation(
            1.into(),
            Operator::Multiply,
            2.into(),
        );

        assert_eq!(result, expected);
    }


    #[test]
    fn parse_multiplication_2() {
        let result = parse(vec![ 
            Token::LiteralInteger(1), 
            Token::Operator(Operator::Multiply), 
            Token::LiteralInteger(2),
            Token::Operator(Operator::Multiply), 
            Token::LiteralInteger(3)]);

        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(
                1.into(),
                Operator::Multiply,
                2.into(),
            )), 
            Operator::Multiply,
            3.into());

        assert_eq!(result, expected);
    }
    

    #[test]
    fn parse_mul_rhs_precedence() {
        let result = parse(vec![ 
            Token::LiteralInteger(1), 
            Token::Operator(Operator::Plus), 
            Token::LiteralInteger(2),
            Token::Operator(Operator::Multiply), 
            Token::LiteralInteger(3)]);

        let expected = Expr::BinaryOperation(
            1.into(),
            Operator::Plus,
            Box::new(Expr::BinaryOperation(
                2.into(),
                Operator::Multiply,
                3.into()
            )), 
           );

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_div_rhs_precedence() {
        let result = parse(vec![ 
            Token::LiteralInteger(1), 
            Token::Operator(Operator::Plus), 
            Token::LiteralInteger(2),
            Token::Operator(Operator::Divide), 
            Token::LiteralInteger(3)]);

        let expected = Expr::BinaryOperation(
            1.into(),
            Operator::Plus,
            Box::new(Expr::BinaryOperation(
                2.into(),
                Operator::Divide,
                3.into()
            )), 
           );

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_mul_lhs_precedence() {
        let result = parse(vec![ 
            Token::LiteralInteger(1), 
            Token::Operator(Operator::Multiply), 
            Token::LiteralInteger(2),
            Token::Operator(Operator::Plus), 
            Token::LiteralInteger(3)]);

        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(
                1.into(),
                Operator::Multiply,
                2.into()
            )), 
            Operator::Plus,
            3.into(),
        );

        assert_eq!(expected, result);
    }
    
    #[test]
    fn parse_complex_precedence() {
        //1 + 2 * 3 * 4 + 5
        let result = parse(vec![ 
            Token::LiteralInteger(1), 
            Token::Operator(Operator::Plus), 
            Token::LiteralInteger(2),
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(3),
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(4),
            Token::Operator(Operator::Plus), 
            Token::LiteralInteger(5)]);

        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(
                1.into(), 
                Operator::Plus, 
                Box::new(Expr::BinaryOperation(
                    Box::new(Expr::BinaryOperation(
                        2.into(), 
                        Operator::Multiply, 
                        3.into()
                    )), 
                    Operator::Multiply, 
                    4.into()
                ))
            )),
            Operator::Plus, 
            5.into()
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_more_complex_precedence() {

        //1 + 2 * 3 + 4 * 5 / 6 + 7 * 8

        let result = parse(vec![ 
            Token::LiteralInteger(1), 
            Token::Operator(Operator::Plus), 
            Token::LiteralInteger(2),
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(3),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(4),
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(5),
            Token::Operator(Operator::Divide),
            Token::LiteralInteger(6),
            Token::Operator(Operator::Plus), 
            Token::LiteralInteger(7),
            Token::Operator(Operator::Multiply), 
            Token::LiteralInteger(8)]);

        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(
                Box::new(Expr::BinaryOperation(
                    1.into(), 
                    Operator::Plus, 
                    Box::new(Expr::BinaryOperation(
                        2.into(), 
                        Operator::Multiply, 
                        3.into()
                    ))
                )), 
                Operator::Plus, 
                Box::new(Expr::BinaryOperation(
                    Box::new(Expr::BinaryOperation(
                        4.into(), 
                        Operator::Multiply, 
                        5.into()
                    )), 
                    Operator::Divide, 
                    6.into()
                ))
            )), 
            Operator::Plus, 
            Box::new(Expr::BinaryOperation(
                7.into(), 
                Operator::Multiply, 
                8.into()
            ))
        );

        assert_eq!(expected, result);
    }
}