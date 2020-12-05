use crate::lexer::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    IntegerValue(i128),
    FloatValue(Float),
    StringValue(String),
    BooleanValue(bool),
    FunctionCall(String, Vec<Expr>),
    Variable(String),
    BinaryOperation(Box<Expr>, Operator, Box<Expr>),
    Parenthesized(Box<Expr>),
    UnaryExpression(Operator, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ASTIfStatement {
    pub expression: Expr,
    pub statements: Vec<AST>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AST {
    StandaloneExpr(Expr),
    Assign {
        variable_name: String,
        expression: Expr,
    },
    IfStatement {
        true_branch: ASTIfStatement,
        elifs: Vec<ASTIfStatement>,
        final_else: Option<Vec<AST>>,
    },
    WhileStatement {
        expression: Expr,
        body: Vec<AST>,
    },
    Break,
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

pub struct Parser {
    parsing_state: Vec<ParsingState>,
    tokens: Vec<Token>,
}

struct ParsingState {
    operator_stack: Vec<Operator>,
    operand_stack: Vec<Expr>,
    index: usize,
    current_indent: usize,
}

#[derive(Debug, Clone)]
pub enum ParsingError {
    ExprError(String),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            parsing_state: vec![ParsingState {
                index: 0,
                operator_stack: vec![],
                operand_stack: vec![],
                current_indent: 0,
            }],
            tokens: tokens,
        }
    }

    fn new_stack(&mut self) {
        self.parsing_state.push(ParsingState {
            index: self.parsing_state.last().unwrap().index,
            operator_stack: vec![],
            operand_stack: vec![],
            current_indent: self.parsing_state.last().unwrap().current_indent,
        })
    }

    fn pop_stack(&mut self) -> ParsingState {
        self.parsing_state.pop().unwrap()
    }

    fn next(&mut self) {
        self.advance(1)
    }

    fn advance(&mut self, offset: isize) {
        let index = self.parsing_state.last().unwrap().index as isize;
        self.parsing_state.last_mut().unwrap().index = (index + offset) as usize;
    }

    fn increment_expected_indent(&mut self) {
        let indent = self.parsing_state.last().unwrap().current_indent as isize;
        self.parsing_state.last_mut().unwrap().current_indent = (indent + 1) as usize;
    }

    fn decrement_expected_indent(&mut self) {
        let indent = self.parsing_state.last().unwrap().current_indent as isize;
        self.parsing_state.last_mut().unwrap().current_indent = (indent - 1) as usize;
    }

    fn get_expected_indent(&mut self) -> usize {
        return self.parsing_state.last_mut().unwrap().current_indent;
    }

    fn set_cur(&mut self, index: usize) {
        self.parsing_state.last_mut().unwrap().index = index;
    }

    fn cur(&self) -> &Token {
        self.cur_offset(0)
    }

    fn cur_opt(&self) -> Option<&Token> {
        self.cur_offset_opt(0)
    }

    fn cur_offset(&self, offset: isize) -> &Token {
        return self.cur_offset_opt(offset).unwrap();
    }

    fn cur_offset_opt(&self, offset: isize) -> Option<&Token> {
        let index = self.parsing_state.last().unwrap().index as isize + offset;
        self.tokens.get(index as usize)
    }

    fn is_last(&self) -> bool {
        self.parsing_state.last().unwrap().index == self.tokens.len() - 1
    }

    fn is_not_end(&self) -> bool {
        self.parsing_state.last().unwrap().index < self.tokens.len()
    }

    fn can_go(&self) -> bool {
        self.is_not_end() && !self.cur_is_newline()
    }

    fn cur_is_newline(&self) -> bool {
        if let Token::NewLine = self.cur() {
            return true;
        } else {
            return false;
        }
    }

    fn push_operand(&mut self, token: Expr) {
        self.parsing_state
            .last_mut()
            .unwrap()
            .operand_stack
            .push(token);
    }

    fn push_operator(&mut self, operator: Operator) {
        self.parsing_state
            .last_mut()
            .unwrap()
            .operator_stack
            .push(operator);
    }

    fn operand_stack(&self) -> &Vec<Expr> {
        return &self.parsing_state.last().unwrap().operand_stack;
    }

    fn operator_stack(&self) -> &Vec<Operator> {
        return &self.parsing_state.last().unwrap().operator_stack;
    }

    fn operand_stack_mut(&mut self) -> &mut Vec<Expr> {
        return &mut self.parsing_state.last_mut().unwrap().operand_stack;
    }

    fn operator_stack_mut(&mut self) -> &mut Vec<Operator> {
        return &mut self.parsing_state.last_mut().unwrap().operator_stack;
    }

    pub fn parse_assign(&mut self) -> Option<AST> {
        if let Token::Identifier(id) = self.cur().clone() {
            self.next();
            if !self.can_go() {
                None
            } else {
                if let Token::Assign = self.cur() {
                    self.next();
                    let expr = self.parse_expr().expect("Expected expression after assign");
                    Some(AST::Assign {
                        variable_name: id.to_string(),
                        expression: expr.resulting_expr,
                    })
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn parse_if_statement(&mut self) -> Option<AST> {
        if let Token::IfKeyword = self.cur().clone() {
            self.next();
            if !self.can_go() {
                None
            } else {
                let expr = self.parse_expr().expect("Expected expr").resulting_expr;
                if let Token::Colon = self.cur() {
                    self.next();
                } else {
                    panic!("Expected colon after if expr");
                }

                if let Token::NewLine = self.cur() {
                    self.next();
                } else {
                    panic!("Expected newline after colon");
                }

                self.increment_expected_indent();
                let ast = self.parse_ast().unwrap();
                let mut if_statement = AST::IfStatement {
                    true_branch: ASTIfStatement {
                        expression: expr,
                        statements: ast,
                    },
                    elifs: vec![],
                    final_else: None,
                };
                self.decrement_expected_indent();

                let cur_identation = self.get_expected_indent();
                //lets try getting the else statement:
                self.new_stack();
                let identation_else = self.skip_whitespace_newline();

                if self.can_go() && identation_else == cur_identation {
                    if let Token::ElseKeyword = self.cur() {
                        self.next();
                        if let Token::Colon = self.cur() {
                            self.next();
                        } else {
                            panic!("Expected colon after if expr");
                        }

                        if let Token::NewLine = self.cur() {
                            self.next();
                        } else {
                            panic!("Expected newline after colon");
                        }

                        self.increment_expected_indent();
                        let ast = self.parse_ast().unwrap();
                        if_statement = match if_statement {
                            AST::IfStatement {
                                true_branch,
                                elifs,
                                final_else: _,
                            } => AST::IfStatement {
                                true_branch,
                                elifs,
                                final_else: Some(ast),
                            },
                            _ => panic!("Unrecognized ast on if else parsing"),
                        };
                        self.decrement_expected_indent();
                    } else {
                        self.pop_stack();
                    }
                } else {
                    self.pop_stack();
                }
                return Some(if_statement);
            }
        } else {
            None
        }
    }

    pub fn parse_while_statement(&mut self) -> Option<AST> {
        if let Token::WhileKeyword = self.cur().clone() {
            self.next();
            if !self.can_go() {
                None
            } else {
                let expr = self.parse_expr().expect("Expected expr").resulting_expr;
                if let Token::Colon = self.cur() {
                    self.next();
                } else {
                    panic!("Expected colon after while expr");
                }

                if let Token::NewLine = self.cur() {
                    self.next();
                } else {
                    panic!("Expected newline after colon");
                }

                self.increment_expected_indent();
                let ast = self.parse_ast().unwrap();
                let while_statement = AST::WhileStatement {
                    expression: expr,
                    body: ast,
                };
                self.decrement_expected_indent();

                return Some(while_statement);
            }
        } else {
            None
        }
    }

    //returns the identation level until the first non-whitespace token
    //final state of this function is right at newline, before the identations
    fn skip_whitespace_newline(&mut self) -> usize {
        let mut identation_level = 0;
        while self.is_not_end() {
            match self.cur() {
                Token::NewLine => {
                    identation_level = 0;
                }
                Token::Indentation => identation_level = identation_level + 1,
                _ => {
                    break;
                }
            }
            self.next();
        }
        return identation_level;
    }

    pub fn parse_ast(&mut self) -> Result<Vec<AST>, ParsingError> {
        let mut results = vec![];

        loop {
            self.new_stack();

            let last_identation = self.skip_whitespace_newline();

            if last_identation == self.get_expected_indent() {
                let popped = self.pop_stack();
                //correct indentation found: commit
                self.set_cur(popped.index);
            } else {
                self.pop_stack();
                return Ok(results);
            }

            if !self.is_not_end() {
                return Ok(results);
            }

            let mut parsed_successfully = false;

            if !parsed_successfully {
                self.new_stack();
                if let Some(assign_ast) = self.parse_assign() {
                    results.push(assign_ast);
                    parsed_successfully = true;
                    let popped = self.pop_stack();
                    //correct indentation found: commit
                    self.set_cur(popped.index);
                    assert!(
                        !self.is_not_end() || self.cur_is_newline(),
                        "Newline or EOF expected after assign"
                    );
                } else {
                    self.pop_stack();
                }
            }

            if !parsed_successfully {
                self.new_stack();
                let expr = self.parse_if_statement();
                match expr {
                    Some(ast_if) => {
                        results.push(ast_if);
                        parsed_successfully = true;
                        let popped = self.pop_stack();
                        //correct indentation found: commit
                        self.set_cur(popped.index);
                        assert!(
                            !self.is_not_end() || self.cur_is_newline(),
                            "Newline or EOF expected after if block"
                        );
                    }
                    None => {
                        parsed_successfully = false;
                        self.pop_stack();
                    }
                }
            }

            if !parsed_successfully {
                self.new_stack();
                let expr = self.parse_while_statement();
                match expr {
                    Some(ast_if) => {
                        results.push(ast_if);
                        parsed_successfully = true;
                        let popped = self.pop_stack();
                        //correct indentation found: commit
                        self.set_cur(popped.index);
                        assert!(
                            !self.is_not_end() || self.cur_is_newline(),
                            "Newline or EOF expected after if block"
                        );
                    }
                    None => {
                        parsed_successfully = false;
                        self.pop_stack();
                    }
                }
            }

            if !parsed_successfully {
                self.new_stack();
                let tok = self.cur();
                match tok {
                    Token::BreakKeyword => {
                        results.push(AST::Break);
                        self.next();
                        parsed_successfully = true;
                        let popped = self.pop_stack();
                        //correct indentation found: commit
                        self.set_cur(popped.index);
                        assert!(
                            !self.is_not_end() || self.cur_is_newline(),
                            "Newline or EOF expected after if block, got {:?}",
                            self.cur_opt()
                        );
                    }
                    _ => {
                        parsed_successfully = false;
                        self.pop_stack();
                    }
                }
            }

            if !parsed_successfully {
                self.new_stack();
                let expr = self.parse_expr()?;
                results.push(AST::StandaloneExpr(expr.resulting_expr));
                let popped = self.pop_stack();
                //correct indentation found: commit
                self.set_cur(popped.index);
                parsed_successfully = true;
                assert!(
                    !self.is_not_end() || self.cur_is_newline(),
                    "Newline or EOF expected after standalone expr"
                );
            }

            if !parsed_successfully {
                panic!("Could not parse code")
            }

            if self.is_not_end() {
                if !self.cur_is_newline() {
                    panic!(
                        "is not end but is also not newline, cur = {:?}, parsed = {:?}",
                        self.cur(),
                        results
                    )
                } else {
                    continue;
                }
            } else {
                break;
            }
        }

        return Ok(results);
    }

    pub fn parse_expr(&mut self) -> Result<ParseExpressionResult, ParsingError> {
        loop {
            if !self.can_go() {
                break;
            }
            let mut was_operand = false;
            let mut not_part_of_expr = false;
            //if there is an open paren, we collect all the tokens for this open paren
            //and parse the sub-expression recursively
            {
                let tok: Token = self.cur().clone();
                match tok {
                    Token::Operator(Operator::OpenParen) => {
                        self.new_stack(); //new parsing stack/state
                        self.next(); //move to the first character, out of the OpenParen

                        match self.parse_expr() {
                            //try parse stuff
                            Ok(expr_result) => {
                                //worked
                                //commit the result
                                let resulting_expr = expr_result.resulting_expr;
                                let parenthesized = Expr::Parenthesized(Box::new(resulting_expr));

                                let popped = self.pop_stack();
                                self.push_operand(parenthesized);
                                self.set_cur(popped.index);

                                was_operand = true;
                            }
                            Err(e) => {
                                eprintln!("Failed parsing exprssion: {:?}", e);
                                return Err(e);
                            }
                        }
                    }
                    Token::Identifier(identifier_str) => {
                        if self.is_last() {
                            self.push_operand(Expr::Variable(identifier_str.to_string()));
                        } else {
                            //if we have an identifier now,
                            //then peek the next token to see if is a open paren.
                            //if it is a open paren, then we are parsing a function.
                            //Otherwise, consider that this is simply a variable
                            self.next();
                            let cur_token = self.cur();

                            if let Token::Operator(Operator::OpenParen) = cur_token {
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
                                self.next();

                                if let Token::Operator(Operator::CloseParen) = self.cur() {
                                    self.push_operand(Expr::FunctionCall(
                                        identifier_str.clone(),
                                        vec![],
                                    ));
                                } else {
                                    self.new_stack();
                                    let list_of_exprs = self.parse_comma_sep_list_expr();

                                    //
                                    match list_of_exprs {
                                        //try parse stuff
                                        Ok(expressions) => {
                                            //worked
                                            //commit the result
                                            let popped = self.pop_stack();
                                            let resulting_exprs = expressions.resulting_expr_list;

                                            self.push_operand(Expr::FunctionCall(
                                                identifier_str.to_string(),
                                                resulting_exprs,
                                            ));

                                            self.set_cur(popped.index);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed parsing exprssion: {:?}", e);
                                            return Err(e);
                                        }
                                    }
                                }
                            } else {
                                self.advance(-1); //is just a variable name, go back
                                self.push_operand(Expr::Variable(identifier_str.to_string()));
                            }
                        }

                        was_operand = true;
                    }
                    Token::LiteralInteger(i) => {
                        self.push_operand(Expr::IntegerValue(i));
                        was_operand = true;
                    }
                    Token::LiteralFloat(f) => {
                        self.push_operand(Expr::FloatValue(f));
                        was_operand = true;
                    }
                    Token::LiteralString(f) => {
                        self.push_operand(Expr::StringValue(f));
                        was_operand = true;
                    }
                    Token::True => {
                        self.push_operand(Expr::BooleanValue(true));
                        was_operand = true;
                    }
                    Token::False => {
                        self.push_operand(Expr::BooleanValue(false));
                        was_operand = true;
                    }
                    Token::Operator(Operator::CloseParen) => {
                        not_part_of_expr = true;
                    }
                    Token::Operator(o) => self.push_operator(o),
                    _ => {
                        not_part_of_expr = true;
                    }
                }
            }
            if not_part_of_expr {
                break;
            } else {
                self.next();
            }

            if was_operand {
                //base case: there is only an operator and an operand, like "-1"
                if self.operand_stack().len() == 1 && self.operator_stack().len() == 1 {
                    let last_operand = self.operand_stack_mut().pop().unwrap();
                    let op = self.operator_stack_mut().pop().unwrap();
                    self.push_operand(Expr::UnaryExpression(op, Box::new(last_operand)));
                }
                //repeat case: 2 * -----2 or even 2 * -2, consume all the minus signals
                else if self.operator_stack().len() > 1 && self.operand_stack().len() == 2 {
                    while self.operator_stack().len() > 1 {
                        let last_operand = self.operand_stack_mut().pop().unwrap();
                        let op = self.operator_stack_mut().pop().unwrap();

                        self.push_operand(Expr::UnaryExpression(op, Box::new(last_operand)));
                    }
                }
                //if it executes the previous if, we will have an operand, operator, and an unary exp operand

                let has_sufficient_operands = self.operand_stack().len() >= 2;
                let has_pending_operators = !self.operator_stack().is_empty();

                if has_sufficient_operands && has_pending_operators {
                    let rhs_root = self.operand_stack_mut().pop().unwrap();
                    let lhs_root = self.operand_stack_mut().pop().unwrap();
                    let op = self.operator_stack_mut().pop().unwrap();

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
                    self.push_operand(bin_op);
                }
            }
        }

        //consume the remaining operators
        if self.operand_stack().len() == 1 {
            while self.operator_stack().len() > 0 {
                let expr = self.operand_stack_mut().pop().unwrap();
                let operator = self.operator_stack_mut().pop().unwrap();
                self.push_operand(Expr::UnaryExpression(operator, Box::new(expr)));
            }
        }

        if !self.operator_stack().is_empty() {
            return Err(ParsingError::ExprError(format!(
                "Unparsed operators: {:?}, operands = {:?}",
                self.operator_stack(),
                self.operand_stack()
            )));
        }

        if self.operand_stack().len() > 1 {
            return Err(ParsingError::ExprError(format!(
                "Unparsed operands: {:?}",
                self.operand_stack()
            )));
        }

        if self.operand_stack().is_empty() {
            return Err(ParsingError::ExprError(String::from(
                "Empty operand stack, didn't parse anything",
            )));
        }
        //let remaining_tokens = Vec::from(token_queue);
        let resulting_expr = clean_parens(self.operand_stack_mut().pop().unwrap());

        Ok(ParseExpressionResult {
            resulting_expr: resulting_expr,
        })
    }

    //expr, expr, ..., expr
    fn parse_comma_sep_list_expr(&mut self) -> Result<ParseListExpressionResult, ParsingError> {
        let mut expressions = vec![];
        loop {
            let parse_result = self.parse_expr();

            match parse_result {
                Ok(r) => {
                    expressions.push(r.resulting_expr);
                }
                Err(e) => {
                    eprintln!("Error on parse: {:?}", e);
                    break;
                }
            }

            if self.can_go() {
                if let Token::Comma = self.cur() {
                    self.next();
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if expressions.is_empty() {
            return Err(ParsingError::ExprError(String::from("While parsing list of expressions: no expression was found. Deal with edge cases before calling this expr.")));
        }

        Ok(ParseListExpressionResult {
            resulting_expr_list: expressions,
        })
    }
}

struct ParseListExpressionResult {
    //remaining_tokens: Vec<&'a Token>,
    resulting_expr_list: Vec<Expr>,
}

pub struct ParseExpressionResult {
    //remaining_tokens: Vec<&'a Token>,
    resulting_expr: Expr,
}

pub fn parse_ast(tokens: Vec<Token>) -> Vec<AST> {
    let mut parser = Parser::new(tokens);
    return parser.parse_ast().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    //Parses a single expression
    fn parse(tokens: Vec<Token>) -> Expr {
        let mut parser = Parser::new(tokens);
        return parser.parse_expr().unwrap().resulting_expr;
    }

    #[test]
    fn multiline_code() {
        let tokens = tokenize(
            "x = 'abc' + 'cde'
y = x + str(True)",
        )
        .unwrap();
        let result = parse_ast(tokens);
        let expected = vec![
            AST::Assign {
                variable_name: String::from("x"),
                expression: Expr::BinaryOperation(
                    Box::new(Expr::StringValue(String::from("abc"))),
                    Operator::Plus,
                    Box::new(Expr::StringValue(String::from("cde"))),
                ),
            },
            AST::Assign {
                variable_name: String::from("y"),
                expression: Expr::BinaryOperation(
                    Box::new(Expr::Variable(String::from("x"))),
                    Operator::Plus,
                    Box::new(Expr::FunctionCall(
                        String::from("str"),
                        vec![Expr::BooleanValue(true)],
                    )),
                ),
            },
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn while_statement() {
        let tokens = tokenize(
            "
while True:
    x = 1
    break
",
        )
        .unwrap();

        let result = parse_ast(tokens);
        let expected = vec![AST::WhileStatement {
            expression: Expr::BooleanValue(true),
            body: vec![
                AST::Assign {
                    variable_name: String::from("x"),
                    expression: Expr::IntegerValue(1),
                },
                AST::Break,
            ],
        }];
        assert_eq!(expected, result);
    }

    #[test]
    fn while_statement_with_if_and_expr() {
        let tokens = tokenize(
            "while x < 1000000:
    if x / 5 == 0:
        break
        
",
        )
        .unwrap();

        let result = parse_ast(tokens);
        let expected = vec![AST::WhileStatement {
            expression: Expr::BinaryOperation(
                Box::new(Expr::Variable("x".to_string())),
                Operator::Less,
                Box::new(Expr::IntegerValue(1000000))
            ),
            body: vec![
                AST::IfStatement {
                    true_branch: ASTIfStatement {
                        expression: Expr::BinaryOperation(
                            Box::new(Expr::BinaryOperation(
                                Box::new(Expr::Variable("x".to_string())),
                                Operator::Divide,
                                Box::new(Expr::IntegerValue(5))
                            )),
                            Operator::Equals,
                            Box::new(Expr::IntegerValue(0))
                        ),
                        statements: vec![AST::Break]
                    },
                    elifs: vec![],
                    final_else: None
                }
            ],
        }];
        assert_eq!(expected, result);
    }


    #[test]
    fn if_statement_with_print_after_and_newlines_before_and_after() {
        let tokens = tokenize(
            "
if x == 0:
    x = x + 1
else:
    x = 999
    if x == 1:
        print(2)
print(x)
",
        )
        .unwrap();

        
        let result = parse_ast(tokens);
        let expected = vec![
            AST::IfStatement {
                true_branch: ASTIfStatement {
                    expression: Expr::BinaryOperation(
                        Box::new(Expr::Variable(String::from("x"))),
                        Operator::Equals,
                        Box::new(Expr::IntegerValue(0)),
                    ),
                    statements: vec![AST::Assign {
                        variable_name: String::from("x"),
                        expression: Expr::BinaryOperation(
                            Box::new(Expr::Variable(String::from("x"))),
                            Operator::Plus,
                            Box::new(Expr::IntegerValue(1)),
                        ),
                    }],
                },
                elifs: vec![],
                final_else: Some(vec![
                    AST::Assign {
                        variable_name: String::from("x"),
                        expression: Expr::IntegerValue(999),
                    },
                    AST::IfStatement {
                        true_branch: ASTIfStatement {
                            expression: Expr::BinaryOperation(
                                Box::new(Expr::Variable(String::from("x"))),
                                Operator::Equals,
                                Box::new(Expr::IntegerValue(1)),
                            ),
                            statements: vec![AST::StandaloneExpr(Expr::FunctionCall(
                                String::from("print"),
                                vec![Expr::IntegerValue(2)],
                            ))],
                        },
                        elifs: vec![],
                        final_else: None,
                    },
                ]),
            },
            AST::StandaloneExpr(Expr::FunctionCall(
                String::from("print"),
                vec![Expr::Variable(String::from("x"))],
            )),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn bunch_of_newlines() {
        let source_wacky = "

<tab><tab><tab><tab><tab><tab><tab>        

if x == 0:
<tab><tab><tab><tab><tab><tab><tab>
<tab>x = x + 1<tab><tab><tab><tab><tab><tab>
<tab>

<tab>if x == 1:
<tab><tab><tab><tab><tab>
<tab><tab>print(2)

<tab>
print(x)


";

        let source_replaced = source_wacky.replace("<tab>", "    ");

        let tokens = tokenize(source_replaced.as_str()).unwrap();

        
        let result = parse_ast(tokens);
        let expected = vec![
            AST::IfStatement {
                true_branch: ASTIfStatement {
                    expression: Expr::BinaryOperation(
                        Box::new(Expr::Variable(String::from("x"))),
                        Operator::Equals,
                        Box::new(Expr::IntegerValue(0)),
                    ),
                    statements: vec![
                        AST::Assign {
                            variable_name: String::from("x"),
                            expression: Expr::BinaryOperation(
                                Box::new(Expr::Variable(String::from("x"))),
                                Operator::Plus,
                                Box::new(Expr::IntegerValue(1)),
                            ),
                        },
                        AST::IfStatement {
                            true_branch: ASTIfStatement {
                                expression: Expr::BinaryOperation(
                                    Box::new(Expr::Variable(String::from("x"))),
                                    Operator::Equals,
                                    Box::new(Expr::IntegerValue(1)),
                                ),
                                statements: vec![AST::StandaloneExpr(Expr::FunctionCall(
                                    String::from("print"),
                                    vec![Expr::IntegerValue(2)],
                                ))],
                            },
                            elifs: vec![],
                            final_else: None,
                        },
                    ],
                },
                elifs: vec![],
                final_else: None,
            },
            AST::StandaloneExpr(Expr::FunctionCall(
                String::from("print"),
                vec![Expr::Variable(String::from("x"))],
            )),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn if_statement() {
        let tokens = tokenize(
            "if x == 0:
    x = x + 1",
        )
        .unwrap();
        let result = parse_ast(tokens);
        let expected = vec![AST::IfStatement {
            true_branch: ASTIfStatement {
                expression: Expr::BinaryOperation(
                    Box::new(Expr::Variable(String::from("x"))),
                    Operator::Equals,
                    Box::new(Expr::IntegerValue(0)),
                ),
                statements: vec![AST::Assign {
                    variable_name: String::from("x"),
                    expression: Expr::BinaryOperation(
                        Box::new(Expr::Variable(String::from("x"))),
                        Operator::Plus,
                        Box::new(Expr::IntegerValue(1)),
                    ),
                }],
            },
            elifs: vec![],
            final_else: None,
        }];
        assert_eq!(expected, result);
    }

    #[test]
    fn if_statement_with_print_after() {
        let tokens = tokenize(
            "if x == 0:
    x = x + 1
print(x)",
        )
        .unwrap();
        let result = parse_ast(tokens);
        let expected = vec![
            AST::IfStatement {
                true_branch: ASTIfStatement {
                    expression: Expr::BinaryOperation(
                        Box::new(Expr::Variable(String::from("x"))),
                        Operator::Equals,
                        Box::new(Expr::IntegerValue(0)),
                    ),
                    statements: vec![AST::Assign {
                        variable_name: String::from("x"),
                        expression: Expr::BinaryOperation(
                            Box::new(Expr::Variable(String::from("x"))),
                            Operator::Plus,
                            Box::new(Expr::IntegerValue(1)),
                        ),
                    }],
                },
                elifs: vec![],
                final_else: None,
            },
            AST::StandaloneExpr(Expr::FunctionCall(
                String::from("print"),
                vec![Expr::Variable(String::from("x"))],
            )),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn multiline_code2() {
        let tokens = tokenize(
            "x = 'abc' + 'cde'
y = x + str(True)
print(y)",
        )
        .unwrap();
        let result = parse_ast(tokens);
        let expected = vec![
            AST::Assign {
                variable_name: String::from("x"),
                expression: Expr::BinaryOperation(
                    Box::new(Expr::StringValue(String::from("abc"))),
                    Operator::Plus,
                    Box::new(Expr::StringValue(String::from("cde"))),
                ),
            },
            AST::Assign {
                variable_name: String::from("y"),
                expression: Expr::BinaryOperation(
                    Box::new(Expr::Variable(String::from("x"))),
                    Operator::Plus,
                    Box::new(Expr::FunctionCall(
                        String::from("str"),
                        vec![Expr::BooleanValue(true)],
                    )),
                ),
            },
            AST::StandaloneExpr(Expr::FunctionCall(
                String::from("print"),
                vec![Expr::Variable(String::from("y"))],
            )),
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_literal_alone() {
        //1 + 1
        let result = parse(vec![Token::LiteralInteger(1)]);

        let expected = Expr::IntegerValue(1);
        assert_eq!(result, expected)
    }

    #[test]
    fn parse_variable() {
        //1 + 1
        let result = parse(vec![Token::Identifier(String::from("x"))]);

        let expected = Expr::Variable(String::from("x"));
        assert_eq!(result, expected)
    }

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
    fn identifier_multiplied() {
        let tokens = tokenize("some_identifier * 5").unwrap();
        let result = parse(tokens);
        let expected = Expr::BinaryOperation(
            Box::new(Expr::Variable(String::from("some_identifier"))),
            Operator::Multiply,
            5.into(),
        );

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
    fn function_call_with_nested_call_with_multiple_expr2() {
        let tokens = tokenize("some_identifier(nested(1), 1)").unwrap();
        let result = parse(tokens);
        let expected = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![
                Expr::FunctionCall(String::from("nested"), vec![Expr::IntegerValue(1)]),
                Expr::IntegerValue(1),
            ],
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
    fn multiply_fcall() {
        let tokens = tokenize("some_identifier(1) * 5").unwrap();
        let result = parse(tokens);
        let call = Expr::FunctionCall(String::from("some_identifier"), vec![Expr::IntegerValue(1)]);
        let expected = Expr::BinaryOperation(Box::new(call), Operator::Multiply, 5.into());
        assert_eq!(expected, result);
    }

    #[test]
    fn multiply_fcall_multiparams() {
        let tokens = tokenize("some_identifier(1, 2) * 5").unwrap();
        let result = parse(tokens);
        let call = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![Expr::IntegerValue(1), Expr::IntegerValue(2)],
        );
        let expected = Expr::BinaryOperation(Box::new(call), Operator::Multiply, 5.into());
        assert_eq!(expected, result);
    }

    #[test]
    fn multiply_fcall_nested_last() {
        let tokens = tokenize("some_identifier(nested()) * 5").unwrap();
        let result = parse(tokens);
        let call = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![Expr::FunctionCall(String::from("nested"), vec![])],
        );
        let expected = Expr::BinaryOperation(Box::new(call), Operator::Multiply, 5.into());
        assert_eq!(expected, result);
    }

    #[test]
    fn multiply_fcall_multiparam_nested_last() {
        let tokens = tokenize("some_identifier(1, nested()) * 5").unwrap();
        let result = parse(tokens);
        let call = Expr::FunctionCall(
            String::from("some_identifier"),
            vec![
                Expr::IntegerValue(1),
                Expr::FunctionCall(String::from("nested"), vec![]),
            ],
        );
        let expected = Expr::BinaryOperation(Box::new(call), Operator::Multiply, 5.into());
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
        let expected = Expr::UnaryExpression(
            Operator::Minus,
            Box::new(Expr::BinaryOperation(
                (5.0).into(),
                Operator::Divide,
                (9.0).into(),
            )),
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn two_times_minus_one() {
        let tokens = tokenize("2 * -1").unwrap();
        let result = parse(tokens);
        let expected = Expr::BinaryOperation(
            (2).into(),
            Operator::Multiply,
            Box::new(Expr::UnaryExpression(Operator::Minus, 1.into())),
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
                Box::new(Expr::UnaryExpression(Operator::Minus, 1.into())),
            )),
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
                    Box::new(Expr::UnaryExpression(Operator::Minus, 1.into())),
                )),
            )),
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
                    Box::new(Expr::UnaryExpression(Operator::Minus, 1.into())),
                )),
            )),
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
                        Box::new(Expr::UnaryExpression(Operator::Minus, 1.into())),
                    )),
                )],
            )),
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fahrenheit_1_expr() {
        let tokens = tokenize("-(5.0 / 9.0) * 32").unwrap();
        let result = parse(tokens);

        let dividend = Expr::BinaryOperation(
            Box::new(Expr::UnaryExpression(
                Operator::Minus,
                Box::new(Expr::BinaryOperation(
                    (5.0).into(),
                    Operator::Divide,
                    (9.0).into(),
                )),
            )),
            Operator::Multiply,
            (32).into(),
        );

        assert_eq!(dividend, result);
    }

    #[test]
    fn fahrenheit_expr() {
        let tokens = tokenize("(-(5.0 / 9.0) * 32) / (1 - (5.0 / 9.0))").unwrap();
        let result = parse(tokens);

        let dividend = Expr::BinaryOperation(
            Box::new(Expr::UnaryExpression(
                Operator::Minus,
                Box::new(Expr::BinaryOperation(
                    (5.0).into(),
                    Operator::Divide,
                    (9.0).into(),
                )),
            )),
            Operator::Multiply,
            (32).into(),
        );

        let divisor = Expr::BinaryOperation(
            1.into(),
            Operator::Minus,
            Box::new(Expr::BinaryOperation(
                (5.0).into(),
                Operator::Divide,
                (9.0).into(),
            )),
        );

        let fahrenheit =
            Expr::BinaryOperation(Box::new(dividend), Operator::Divide, Box::new(divisor));

        assert_eq!(fahrenheit, result);
    }

    #[test]
    fn test_assign() {
        let tokens = tokenize("x = 1").unwrap();
        let result = parse_ast(tokens);
        let expected = vec![AST::Assign {
            variable_name: String::from("x"),
            expression: Expr::IntegerValue(1),
        }];
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_ast_first_token_is_identifier() {
        let tokens = tokenize("x * 1").unwrap();
        let result = parse_ast(tokens);
        let expected = vec![AST::StandaloneExpr(Expr::BinaryOperation(
            Box::new(Expr::Variable(String::from("x"))),
            Operator::Multiply,
            1.into(),
        ))];
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_assign_expr() {
        let tokens = tokenize("x = x * 1").unwrap();
        let result = parse_ast(tokens);
        let expected = vec![AST::Assign {
            variable_name: String::from("x"),
            expression: Expr::BinaryOperation(
                Box::new(Expr::Variable(String::from("x"))),
                Operator::Multiply,
                1.into(),
            ),
        }];
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_just_id_ast() {
        let tokens = tokenize("x").unwrap();
        let result = parse_ast(tokens);
        let expected = vec![AST::StandaloneExpr(Expr::Variable(String::from("x")))];
        assert_eq!(expected, result);
    }

    #[test]
    fn not_operator() {
        let tokens = tokenize("not True").unwrap();
        let result = parse(tokens);
        let expected = Expr::UnaryExpression(Operator::Not, Box::new(Expr::BooleanValue(true)));

        assert_eq!(expected, result);
    }

    #[test]
    fn not_true_and_false() {
        let tokens = tokenize("not (True and False)").unwrap();
        let result = parse(tokens);
        let expected = Expr::UnaryExpression(
            Operator::Not,
            Box::new(Expr::BinaryOperation(
                Box::new(Expr::BooleanValue(true)),
                Operator::And,
                Box::new(Expr::BooleanValue(false)),
            )),
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn assign_boolean_expr() {
        let tokens = tokenize("x = not (True and False) or (False)").unwrap();
        let result = parse_ast(tokens);
        let expected = vec![AST::Assign {
            variable_name: String::from("x"),
            expression: Expr::BinaryOperation(
                Box::new(Expr::UnaryExpression(
                    Operator::Not,
                    Box::new(Expr::BinaryOperation(
                        Box::new(Expr::BooleanValue(true)),
                        Operator::And,
                        Box::new(Expr::BooleanValue(false)),
                    )),
                )),
                Operator::Or,
                Box::new(Expr::BooleanValue(false)),
            ),
        }];

        assert_eq!(expected, result);
    }

    #[test]
    fn assign_string_expr() {
        let tokens = tokenize("x = 'abc'").unwrap();
        let result = parse_ast(tokens);
        let expected = vec![AST::Assign {
            variable_name: String::from("x"),
            expression: Expr::StringValue(String::from("abc")),
        }];

        assert_eq!(expected, result);
    }

    #[test]
    fn assign_string_concat_expr() {
        let tokens = tokenize("x = 'abc' + 'cde'").unwrap();
        let result = parse_ast(tokens);
        let expected = vec![AST::Assign {
            variable_name: String::from("x"),
            expression: Expr::BinaryOperation(
                Box::new(Expr::StringValue(String::from("abc"))),
                Operator::Plus,
                Box::new(Expr::StringValue(String::from("cde"))),
            ),
        }];

        assert_eq!(expected, result);
    }
}
