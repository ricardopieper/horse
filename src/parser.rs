use crate::lexer::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    IntegerValue(i128),
    FloatValue(Float),
    FunctionCall(String, Vec<Expr>),
    Variable(String),
    BinaryOperation(Box<Expr>, Operator, Box<Expr>),
    Parenthesized(Box<Expr>),
    UnaryExpression(Operator, Box<Expr>)
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
        _ => 1,
    }
}

fn clean_parens(expr: Expr) -> Expr {
    match expr {
        Expr::Parenthesized(e) => clean_parens(*e),
        Expr::UnaryExpression(op, e) => Expr::UnaryExpression(op, Box::new(clean_parens(*e))),
        Expr::BinaryOperation(left, op, right) => {
            let left_clean = Box::new(clean_parens(*left));
            let right_clean = Box::new(clean_parens(*right));
            Expr::BinaryOperation(left_clean, op, right_clean)
        }
        _ => expr,
    }
}

pub fn parse(tokens: Vec<Token>) -> Expr {
    let mut list_expr_result = parse_comma_sep_list_expr(tokens);
    if list_expr_result.remaining_tokens.len() != 0 {
        panic!(
            "There are remaining tokens! ${:?}",
            list_expr_result.remaining_tokens
        );
    }
    if list_expr_result.resulting_expr_list.len() == 0 {
        panic!(
            "Nothing was parsed from expr ${:?}",
            list_expr_result.remaining_tokens
        );
    }

    return list_expr_result.resulting_expr_list.swap_remove(0);
}

struct ParseListExpressionResult {
    remaining_tokens: Vec<Token>,
    resulting_expr_list: Vec<Expr>,
}

fn parse_comma_sep_list_expr(tokens: Vec<Token>) -> ParseListExpressionResult {
    let mut expressions = vec![];
    let mut current_tokens = tokens;
    let mut final_remaining_tokens = vec![];
    loop {
        let current_length = current_tokens.len();
        let parse_result = parse_expr(current_tokens);
        if parse_result.remaining_tokens.len() == current_length {
            break;
        }

        let remaining_current_length = parse_result.remaining_tokens.len();
        current_tokens = parse_result.remaining_tokens;
        expressions.push(parse_result.resulting_expr);
        if remaining_current_length == 0 {
            break;
        }
        if let Token::Comma = current_tokens[0] {
            current_tokens.remove(0);
            continue;
        } else {
            final_remaining_tokens = current_tokens;
            break;
        }
    }

    ParseListExpressionResult {
        resulting_expr_list: expressions,
        remaining_tokens: final_remaining_tokens,
    }
}

struct ParseExpressionResult {
    remaining_tokens: Vec<Token>,
    resulting_expr: Expr,
}

//Parses a single expression
fn parse_expr(tokens: Vec<Token>) -> ParseExpressionResult {

    
    let mut token_queue = VecDeque::from(tokens);
    let mut operator_stack: Vec<Operator> = vec![];
    let mut operand_stack: Vec<Expr> = vec![];

    loop {
        if token_queue.is_empty() {
            break;
        }
        let mut was_operand = false;
        let mut not_part_of_expr = false;
        //if there is an open paren, we collect all the tokens for this open paren
        //and parse the sub-expression recursively
        {
            let tok = token_queue.pop_front().unwrap();
            match tok {
                Token::Operator(Operator::OpenParen) => {
                    let mut between_parens = vec![];
                    let mut close_parens_found = 0;
                    let mut open_parens_found = 1; //1 for the tok
                    loop {
                        let next_token = token_queue.pop_front();
                        if !next_token.is_some() {
                            break;
                        }
                        let next_token = next_token.unwrap();

                        if let Token::Operator(Operator::OpenParen) = next_token {
                            open_parens_found = open_parens_found + 1;
                        }

                        if let Token::Operator(Operator::CloseParen) = next_token {
                            close_parens_found = close_parens_found + 1;
                            if open_parens_found == close_parens_found {
                                break;
                            }
                        }
                        between_parens.push(next_token);
                    }

                    if open_parens_found != close_parens_found {
                        panic!("Mismatched parens!");
                    }

                    let parsed_subexpr = parse(between_parens);

                    operand_stack.push(Expr::Parenthesized(Box::new(parsed_subexpr)));
                    was_operand = true;
                }
                Token::Identifier(identifier_str) => {
                    //if we have an identifier now,
                    //then peek the next token to see if is a open paren.
                    //if it is a open paren, then we are parsing a function.
                    //Otherwise, consider that this is simply a variable
                    let next_token = token_queue.front();

                    if let Some(Token::Operator(Operator::OpenParen)) = next_token {
                        //when we parse a funcion, we need to parse its arguments as well.
                        //however, the list of arguments is a list of expressions, and those expressions might be
                        //function calls as well.
                        //like fcall(fcall(fcall(1, fcall(2)), fcall2(3, fcall())))....)
                        //Unlike parenthesized expressions, in this case I cannot just fetch everything between
                        //the parens because there are commas separating the arguments. I can't also fetch all
                        //the tokens until I find a comma, because i would have a list of tokens containig
                        //[fcall(fcall(fcall(1,] as my list of tokens to parse.
                        //I will need to parse lists of expressions for other stuff as well, like array items and tuple items.
                        //Perhaps a better strategy is to make it a core function of the parser: Parse list of expressions instead of just a single expr.
                        //And we need the parse function to be more tolerant of tokens outside of expressions: if it finds something that doesn't look
                        //like it's part of an expression, then maybe we should just understand that the expression has been finished.
                        let cur_token = token_queue.pop_front();
                        if !cur_token.is_some() {
                            break;
                        }
                        let cur_token = cur_token.unwrap(); //guaranteed to be OpenParen
                        assert_eq!(cur_token, Token::Operator(Operator::OpenParen));

                        if let Some(Token::Operator(Operator::CloseParen)) = token_queue.front() {
                            token_queue.pop_front();
                            operand_stack.push(Expr::FunctionCall(identifier_str, vec![]));
                        } else {
                            let list_of_exprs = parse_comma_sep_list_expr(Vec::from(token_queue));

                            operand_stack.push(Expr::FunctionCall(
                                identifier_str,
                                list_of_exprs.resulting_expr_list,
                            ));
                            token_queue = VecDeque::from(list_of_exprs.remaining_tokens);
                            assert_eq!(
                                token_queue.front(),
                                (Some(Token::Operator(Operator::CloseParen)).as_ref())
                            );
                            token_queue.pop_front();
                        }
                    } else {
                        operand_stack.push(Expr::Variable(identifier_str));
                    }
                    was_operand = true;
                }
                Token::LiteralInteger(i) => {
                    operand_stack.push(Expr::IntegerValue(i));
                    was_operand = true;
                }
                Token::LiteralFloat(f) => {
                    operand_stack.push(Expr::FloatValue(f));
                    was_operand = true;
                }
                Token::Operator(Operator::CloseParen) => {
                    not_part_of_expr = true;
                    token_queue.push_front(tok);
                }
                Token::Operator(o) => {
                    operator_stack.push(o)
                },
                _ => {
                    not_part_of_expr = true;
                    token_queue.push_front(tok);
                }
            }
        }
        if not_part_of_expr {
            break;
        }

        //-(5.0 / 9.0) * 32
       
        if was_operand {
            //base case: there is only an operator and an operand, like "-1"
            if operand_stack.len() == 1 && operator_stack.len() == 1 {
                let last_operand = operand_stack.pop().unwrap();
                let op = operator_stack.pop().unwrap();
                operand_stack.push(
                    Expr::UnaryExpression(op, Box::new(last_operand)));
                
            }
            //repeat case: 2 * -----2 or even 2 * -2, consume all the minus signals
            else if operator_stack.len() > 1 && operand_stack.len() == 2 {
                while operator_stack.len() > 1 {

                    let last_operand = operand_stack.pop().unwrap();
                    let op = operator_stack.pop().unwrap();

                    operand_stack.push(
                        Expr::UnaryExpression(op, Box::new(last_operand)));
                }
            }
            //if it executes the previous if, we will have an operand, operator, and an unary exp operand 

            let has_sufficient_operands = operand_stack.len() >= 2;
            let has_pending_operators = !operator_stack.is_empty();

            if has_sufficient_operands && has_pending_operators {
                let rhs_root = operand_stack.pop().unwrap();
                let lhs_root = operand_stack.pop().unwrap();
                let op = operator_stack.pop().unwrap();


                let mut bin_op = Expr::BinaryOperation(
                    Box::new(lhs_root.clone()),
                    op,
                    Box::new(rhs_root.clone()),
                );
                if let Expr::BinaryOperation(lhs_down, op_down, rhs_down) = &lhs_root {
                    let precedence_down = precedence(*op_down);
                    let precedence_root = precedence(op);
                    if precedence_root > precedence_down {
                        bin_op = Expr::BinaryOperation(
                            lhs_down.clone(),
                            *op_down,
                            Box::new(Expr::BinaryOperation(
                                rhs_down.clone(),
                                op,
                                Box::new(rhs_root.clone()),
                            )),
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
                            Box::new(Expr::BinaryOperation(
                                rhs_down.clone(),
                                op,
                                Box::new(lhs_root.clone()),
                            )),
                        );
                    }
                }
                operand_stack.push(bin_op);
            }
        
        }
    }

    //consume the remaining operators
    if operand_stack.len() == 1 {
        while operator_stack.len() > 0 {
            let expr = operand_stack.pop().unwrap();
            operand_stack.push(Expr::UnaryExpression(operator_stack.pop().unwrap(), Box::new(expr)));
        }
    }


    if !operator_stack.is_empty() {
        panic!(
            "Unparsed operators: {:?}, operands = {:?}",
            operator_stack, operand_stack
        );
    }

    if operand_stack.len() > 1 {
        panic!("Unparsed operands: {:?}", operand_stack);
    }

    if operand_stack.is_empty() {
        panic!("Empty operand stack, didn't parse anything");
    }
    let remaining_tokens = Vec::from(token_queue);
    let resulting_expr = clean_parens(operand_stack.pop().unwrap());
    ParseExpressionResult {
        remaining_tokens,
        resulting_expr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_1_plus_1() {
        //1 + 1
        let result = parse(vec![
            Token::LiteralInteger(1),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(1),
        ]);

        let expected = Expr::BinaryOperation(1.into(), Operator::Plus, 1.into());
        assert_eq!(result, expected)
    }

    #[test]
    fn parse_float_10_plus_integer_1() {
        //10.0 + 1
        let result = parse(vec![
            Token::LiteralFloat(10.0.into()),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(1),
        ]);

        let expected = Expr::BinaryOperation(10.0.into(), Operator::Plus, 1.into());
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_float_3_values_2_ops() {
        //1 + 2 + 3
        let result = parse(vec![
            Token::LiteralInteger(1),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(2),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(3),
        ]);
        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(1.into(), Operator::Plus, 2.into())),
            Operator::Plus,
            3.into(),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn parse_multiplication() {
        //1 * 2
        let result = parse(vec![
            Token::LiteralInteger(1),
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(2),
        ]);

        let expected = Expr::BinaryOperation(1.into(), Operator::Multiply, 2.into());
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_multiplication_2() {
        //1 * 2 * 3
        let result = parse(vec![
            Token::LiteralInteger(1),
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(2),
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(3),
        ]);

        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(
                1.into(),
                Operator::Multiply,
                2.into(),
            )),
            Operator::Multiply,
            3.into(),
        );

        assert_eq!(result, expected);
    }
    #[test]
    fn parse_mul_rhs_precedence() {
        //1 + 2 * 3
        let result = parse(vec![
            Token::LiteralInteger(1),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(2),
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(3),
        ]);

        /*
          +
         / \
        1   *
           / \
          2   3
        */
        let expected = Expr::BinaryOperation(
            1.into(),
            Operator::Plus,
            Box::new(Expr::BinaryOperation(
                2.into(),
                Operator::Multiply,
                3.into(),
            )),
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_div_rhs_precedence() {
        //1 + 2 / 3
        let result = parse(vec![
            Token::LiteralInteger(1),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(2),
            Token::Operator(Operator::Divide),
            Token::LiteralInteger(3),
        ]);

        /*
          +
         / \
        1  (div)
           / \
          2   3
        */
        let expected = Expr::BinaryOperation(
            1.into(),
            Operator::Plus,
            Box::new(Expr::BinaryOperation(2.into(), Operator::Divide, 3.into())),
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_mul_lhs_precedence() {
        //1 * 2 + 3
        let result = parse(vec![
            Token::LiteralInteger(1),
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(2),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(3),
        ]);

        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(
                1.into(),
                Operator::Multiply,
                2.into(),
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
            Token::LiteralInteger(5),
        ]);

        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(
                1.into(),
                Operator::Plus,
                Box::new(Expr::BinaryOperation(
                    Box::new(Expr::BinaryOperation(
                        2.into(),
                        Operator::Multiply,
                        3.into(),
                    )),
                    Operator::Multiply,
                    4.into(),
                )),
            )),
            Operator::Plus,
            5.into(),
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
            Token::LiteralInteger(8),
        ]);

        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(
                Box::new(Expr::BinaryOperation(
                    1.into(),
                    Operator::Plus,
                    Box::new(Expr::BinaryOperation(
                        2.into(),
                        Operator::Multiply,
                        3.into(),
                    )),
                )),
                Operator::Plus,
                Box::new(Expr::BinaryOperation(
                    Box::new(Expr::BinaryOperation(
                        4.into(),
                        Operator::Multiply,
                        5.into(),
                    )),
                    Operator::Divide,
                    6.into(),
                )),
            )),
            Operator::Plus,
            Box::new(Expr::BinaryOperation(
                7.into(),
                Operator::Multiply,
                8.into(),
            )),
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_literal_parens() {
        //(1)
        let result = parse(vec![
            Token::Operator(Operator::OpenParen),
            Token::LiteralInteger(1),
            Token::Operator(Operator::CloseParen),
        ]);

        let expected = Expr::IntegerValue(1);

        assert_eq!(expected, result);
    }
    #[test]
    fn parse_parens_expr() {
        //(1 + 2) * 3
        let result = parse(vec![
            Token::Operator(Operator::OpenParen),
            Token::LiteralInteger(1),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(2),
            Token::Operator(Operator::CloseParen),
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(3),
        ]);

        /*
          +
         / \
        1   *
           / \
          2   3
        */
        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(1.into(), Operator::Plus, 2.into())),
            Operator::Multiply,
            3.into(),
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn integration_with_lexer() {
        let tokens = tokenize("(1 + 2) * 3").unwrap();
        let result = parse(tokens);
        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(1.into(), Operator::Plus, 2.into())),
            Operator::Multiply,
            3.into(),
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn complex_parenthesized_expr() {
        let tokens = tokenize("(1 + 2) * (3 + 1 + (10 / 5))").unwrap();
        let result = parse(tokens);
        let expected = Expr::BinaryOperation(
            Box::new(Expr::BinaryOperation(1.into(), Operator::Plus, 2.into())),
            Operator::Multiply,
            Box::new(Expr::BinaryOperation(
                Box::new(Expr::BinaryOperation(3.into(), Operator::Plus, 1.into())),
                Operator::Plus,
                Box::new(Expr::BinaryOperation(10.into(), Operator::Divide, 5.into())),
            )),
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn tons_of_useless_parenthesis() {
        let tokens = tokenize("(((((((((1)))))))))").unwrap();
        let result = parse(tokens);

        let expected = Expr::IntegerValue(1);

        assert_eq!(expected, result);
    }

    #[test]
    fn just_an_identifier() {
        let tokens = tokenize("some_identifier").unwrap();
        let result = parse(tokens);
        let expected = Expr::Variable(String::from("some_identifier"));

        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_without_parameters() {
        let tokens = tokenize("some_identifier()").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(String::from("some_identifier"), vec![]);

        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_one_param() {
        let tokens = tokenize("some_identifier(1)").unwrap();
        let result = parse(tokens);
        let expected =
            Expr::FunctionCall(String::from("some_identifier"), vec![Expr::IntegerValue(1)]);

        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_many_params() {
        let tokens = tokenize("some_identifier(1, 2, 3)").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![
                Expr::IntegerValue(1),
                Expr::IntegerValue(2),
                Expr::IntegerValue(3),
            ],
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_expression() {
        let tokens = tokenize("some_identifier(1 * 2)").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![Expr::BinaryOperation(
                1.into(),
                Operator::Multiply,
                2.into(),
            )],
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_list_of_expressions() {
        let tokens = tokenize("some_identifier(1 * 2, 3 + 5, 88)").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![
                Expr::BinaryOperation(1.into(), Operator::Multiply, 2.into()),
                Expr::BinaryOperation(3.into(), Operator::Plus, 5.into()),
                Expr::IntegerValue(88),
            ],
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_nested_call_with_empty_params() {
        let tokens = tokenize("some_identifier(nested())").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![Expr::FunctionCall(String::from("nested"), vec![])],
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_nested_call_with_single_params() {
        let tokens = tokenize("some_identifier(nested(1))").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![Expr::FunctionCall(
                String::from("nested"),
                vec![Expr::IntegerValue(1)],
            )],
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_nested_call_with_multiple_params() {
        let tokens = tokenize("some_identifier(nested(1, 2))").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![Expr::FunctionCall(
                String::from("nested"),
                vec![Expr::IntegerValue(1), Expr::IntegerValue(2)],
            )],
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_nested_call_with_multiple_expr() {
        let tokens = tokenize("some_identifier(nested(1 * 2, 2 / 3.4))").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![Expr::FunctionCall(
                String::from("nested"),
                vec![
                    Expr::BinaryOperation(1.into(), Operator::Multiply, 2.into()),
                    Expr::BinaryOperation(2.into(), Operator::Divide, (3.4).into()),
                ],
            )],
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_nested_call_with_multiple_expr_also_unnested() {
        let tokens = tokenize("some_identifier(nested(1 * 2, 2 / 3.4), 3, nested2())").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![
                Expr::FunctionCall(
                    String::from("nested"),
                    vec![
                        Expr::BinaryOperation(1.into(), Operator::Multiply, 2.into()),
                        Expr::BinaryOperation(2.into(), Operator::Divide, (3.4).into()),
                    ],
                ),
                Expr::IntegerValue(3),
                Expr::FunctionCall(String::from("nested2"), vec![]),
            ],
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_nested_call_with_multiple_expr_also_unnested_used_in_expression_right() {
        let tokens = tokenize("some_identifier(nested(1 * 2, 2 / 3.4), 3, nested2()) * 5").unwrap();
        let result = parse(tokens);
        let call = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![
                Expr::FunctionCall(
                    String::from("nested"),
                    vec![
                        Expr::BinaryOperation(1.into(), Operator::Multiply, 2.into()),
                        Expr::BinaryOperation(2.into(), Operator::Divide, (3.4).into()),
                    ],
                ),
                Expr::IntegerValue(3),
                Expr::FunctionCall(String::from("nested2"), vec![]),
            ],
        );
        let expected = Expr::BinaryOperation(Box::new(call), Operator::Multiply, 5.into());
        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_nested_call_with_multiple_expr_also_unnested_used_in_expression_left() {
        let tokens = tokenize("5 * some_identifier(nested(1 * 2, 2 / 3.4), 3, nested2())").unwrap();
        let result = parse(tokens);
        let call = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![
                Expr::FunctionCall(
                    String::from("nested"),
                    vec![
                        Expr::BinaryOperation(1.into(), Operator::Multiply, 2.into()),
                        Expr::BinaryOperation(2.into(), Operator::Divide, (3.4).into()),
                    ],
                ),
                Expr::IntegerValue(3),
                Expr::FunctionCall(String::from("nested2"), vec![]),
            ],
        );
        let expected = Expr::BinaryOperation(5.into(), Operator::Multiply, Box::new(call));
        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_with_a_bunch_of_useless_params() {
        let tokens = tokenize("func((((((1))))))").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(String::from("func"), vec![Expr::IntegerValue(1)]);
        assert_eq!(expected, result);
    }

    #[test]
    fn minus_one() {
        let tokens = tokenize("-1").unwrap();
        let result = parse(tokens);
        let expected = Expr::UnaryExpression(Operator::Minus, Box::new(Expr::IntegerValue(1)));
        assert_eq!(expected, result);
    }

    #[test]
    fn minus_expr() {
        let tokens = tokenize("-(5.0 / 9.0)").unwrap();
        let result = parse(tokens);
        let expected = Expr::UnaryExpression(Operator::Minus, 
            Box::new(Expr::BinaryOperation(
                (5.0).into(),
                Operator::Divide,
                (9.0).into()
            )));
        assert_eq!(expected, result);
    }

    #[test]
    fn two_times_minus_one() {
        let tokens = tokenize("2 * -1").unwrap();
        let result = parse(tokens);
        let expected = Expr::BinaryOperation(
                (2).into(),
                Operator::Multiply,
                Box::new(Expr::UnaryExpression(
                    Operator::Minus,
                    1.into()
                ))
            );
        assert_eq!(expected, result);
    }


    #[test]
    fn two_times_minus_repeated_one() {
        let tokens = tokenize("2 * --1").unwrap();
        let result = parse(tokens);
        let expected = Expr::BinaryOperation(
                (2).into(),
                Operator::Multiply,
                Box::new(Expr::UnaryExpression(
                    Operator::Minus,
                    Box::new(Expr::UnaryExpression(
                        Operator::Minus,
                        1.into()
                    ))))
            );
        assert_eq!(expected, result);
    }

    #[test]
    fn two_times_minus_plus_minus_one() {
        let tokens = tokenize("2 * -+-1").unwrap();
        let result = parse(tokens);
        let expected = Expr::BinaryOperation(
                (2).into(),
                Operator::Multiply,
                Box::new(Expr::UnaryExpression(
                    Operator::Minus,
                    Box::new(Expr::UnaryExpression(
                        Operator::Plus,
                        Box::new(Expr::UnaryExpression(
                            Operator::Minus,
                            1.into()
                        ))))))
            );
        assert_eq!(expected, result);
    }

    #[test]
    fn two_times_minus_plus_minus_one_parenthesized() {
        let tokens = tokenize("2 * (-+-1)").unwrap();
        let result = parse(tokens);
        let expected = Expr::BinaryOperation(
                (2).into(),
                Operator::Multiply,
                Box::new(Expr::UnaryExpression(
                    Operator::Minus,
                    Box::new(Expr::UnaryExpression(
                        Operator::Plus,
                        Box::new(Expr::UnaryExpression(
                            Operator::Minus,
                            1.into()
                        ))))))
            );
        assert_eq!(expected, result);
    }

    #[test]
    fn two_times_minus_plus_minus_one_in_function_call() {
        let tokens = tokenize("2 * func(-+-1)").unwrap();
        let result = parse(tokens);
        let expected = Expr::BinaryOperation(
                (2).into(),
                Operator::Multiply,
                Box::new(Expr::FunctionCall(
                    String::from("func"),
                    vec![Expr::UnaryExpression(
                        Operator::Minus,
                        Box::new(Expr::UnaryExpression(
                            Operator::Plus,
                            Box::new(Expr::UnaryExpression(
                                Operator::Minus,
                                1.into()
                            )))))]
                ))
            );
        assert_eq!(expected, result);
    }


    #[test]
    fn fahrenheit_1_expr() {
        //-(5.0 / 9.0) * 32
        let tokens = tokenize("-(5.0 / 9.0) * 32").unwrap();
        let result = parse(tokens);

        let dividend = Expr::BinaryOperation(
            Box::new(Expr::UnaryExpression(
                Operator::Minus, 
                Box::new(Expr::BinaryOperation(
                    (5.0).into(),
                    Operator::Divide,
                    (9.0).into()
                )))),
            Operator::Multiply,
            (32).into()
        );

        assert_eq!(dividend, result);
    }

    #[test]
    fn fahrenheit_expr() {
        //(-(5.0 / 9.0) * 32)    /    (1 - (5.0 / 9.0))
        let tokens = tokenize("(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))").unwrap();
        let result = parse(tokens);

        let dividend = Expr::BinaryOperation(
            Box::new(Expr::UnaryExpression(
                Operator::Minus, 
                Box::new(Expr::BinaryOperation(
                    (5.0).into(),
                    Operator::Divide,
                    (9.0).into()
                )))),
            Operator::Multiply,
            (32).into()
        );

        let divisor = Expr::BinaryOperation(
            1.into(),
            Operator::Minus,
            Box::new(Expr::BinaryOperation(
                (5.0).into(),
                Operator::Divide,
                (9.0).into()
            ))
        );

        let fahrenheit = Expr::BinaryOperation(
            Box::new(dividend),
            Operator::Divide,
            Box::new(divisor)
        );
        
        assert_eq!(fahrenheit, result);
    }
}


//2 * - 1
//2 = operand [2], operator = []
//* = operand [2], operator = [*]
//- = operand [2], operator = [*-]
//1 = operand [2, 1], operator = [*-]
//*(- 2 1)