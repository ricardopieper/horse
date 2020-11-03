use std::cmp::Ordering;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    BitShiftLeft,
    BitShiftRight,
    Not,
    Equals,
    NotEquals,
    OpenParen,
    CloseParen
}
#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
pub struct Float(pub f64);

impl From<f64> for Float {
    fn from(w: f64) -> Float {
        Float(w)
    }
}

impl Eq for Float {}

impl Ord for Float {
    fn cmp(&self, other: &Float) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Token {
    LiteralFloat(Float),
    LiteralInteger(i128),
    Operator(Operator),
    Identifier(String),
    Assign,
    None,
    Comma
}

#[derive(Debug)]
enum PartialToken {
    UndefinedOrWhitespace,
    LiteralFloat(String),
    Operator(String),
    Identifier(String), 
    Comma
}

impl PartialToken {
    fn to_token(self) -> Token {
        match self {
            Self::UndefinedOrWhitespace => panic!("Unexpected undefined token. This is a tokenizer bug."),
            Self::Identifier(s) => {
                if s == "None" {
                    Token::None
                } else {
                    Token::Identifier(s)
                }
            },
            Self::Comma => Token::Comma,
            Self::LiteralFloat(s) => {
                
                if s.contains('.') || s.contains('e') {
                    match s.parse::<f64>() {
                        Ok(f) => Token::LiteralFloat(Float(f)),
                        _ => panic!("Error parsing float value {}. Should have generated a tokenizer error. This is a bug.", s)
                    }
                } else {
                    match s.parse::<i128>() {
                        Ok(f) => Token::LiteralInteger(f),
                        _ => panic!("Error parsing integer value {}. Should have generated a tokenizer error. This is a bug.", s)
                    }
                }
            },
            Self::Operator(s) => {
                match s.as_str() {
                    "+" => Token::Operator(Operator::Plus),
                    "-" => Token::Operator(Operator::Minus),
                    "*" => Token::Operator(Operator::Multiply),
                    "/" => Token::Operator(Operator::Divide),
                    "<<" => Token::Operator(Operator::BitShiftLeft),
                    ">>" => Token::Operator(Operator::BitShiftRight),
                    "!" => Token::Operator(Operator::Not),
                    "==" => Token::Operator(Operator::Equals),
                    "=" => Token::Assign,
                    "!=" => Token::Operator(Operator::NotEquals),
                    "(" => Token::Operator(Operator::OpenParen),
                    ")" => Token::Operator(Operator::CloseParen),
                    _ => panic!("Unimplemented operator {}", s)
                }
            }
        }
    }
}

pub struct Tokenizer {
    index: usize,
    chars: Vec<char>,
    cur_partial_token: PartialToken,
    final_result: Vec<Token>,
    eater_buf: String,
}

impl Tokenizer {
    pub fn new(source: &str) -> Tokenizer {
        Tokenizer {
            index: 0,
            chars: source.chars().collect(),
            cur_partial_token: PartialToken::UndefinedOrWhitespace,
            final_result: vec![],
            eater_buf: String::new(),
        }
    }

    fn reset_eater_buffer(&mut self) {
        self.eater_buf = String::new();
    }

    fn next(&mut self) {
        self.advance(1)
    }

    fn advance(&mut self, offset: usize) {
        self.index = self.index + offset;
    }

    fn cur(&self) -> char {
        self.cur_offset(0)
    }

    fn cur_offset(&self, offset: usize) -> char {
        self.chars[self.index + offset]
    }

    fn can_go(&self) -> bool {
        self.index < self.chars.len()
    }

    fn eat_numbers(&mut self) -> bool {
        let mut ate = false;
        while self.can_go() && self.cur().is_numeric() {
            self.eater_buf.push(self.cur());
            self.next();
            ate = true;
        }
        ate
    }

    fn eat_identifier(&mut self) -> bool {
        let first_char_is_valid_identifier = self.can_go() && self.cur().is_ascii_alphabetic()  || self.cur() == '_';

        if first_char_is_valid_identifier {
            self.eater_buf.push(self.cur());
            self.next();
        } else {
            return false;
        }

        while self.can_go() && (self.cur().is_ascii_alphanumeric()  || self.cur() == '_') {
            self.eater_buf.push(self.cur());
            self.next();
        }

        true
    }

    fn eat_char(&mut self, char_to_eat: char) -> bool {
        if self.can_go() && self.cur() == char_to_eat {
            self.eater_buf.push(self.cur());
            self.next();
            true
        } else {
            false
        }
    }

    fn commit_current_token(&mut self) {
        match self.cur_partial_token {
            PartialToken::UndefinedOrWhitespace => {}
            _ => {
                let cur_token = std::mem::replace(
                    &mut self.cur_partial_token,
                    PartialToken::UndefinedOrWhitespace,
                );
                self.final_result.push(cur_token.to_token());
            }
        };
    }

    fn clone_buf(&self) -> String {
        self.eater_buf.clone()
    }

    fn match_partial(&mut self, query: &str) -> (bool, usize) {
        let mut matched_chars = 0;
        let chars: Vec<char> = query.chars().collect();
        for i in 0..query.len() {
            if self.cur_offset(i) != chars[i] {
                return (false, 0);
            }
            matched_chars = matched_chars + 1
        }
        return (true, matched_chars);
    }

    fn match_first_and_advance<'a>(&mut self, query: &'a [&'a str]) -> Option<&'a str> {
        for q in query {
            let (success, len) = self.match_partial(q);
            if success {
                self.advance(len);
                return Some(q);
            }
        }
        return None;
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, String> {
        let operators = &["+", "-", "*", "/", "<<", ">>", "!=", "==", "=", "!", "(", ")"];
        while self.can_go() {
            self.commit_current_token();
            if self.cur().is_numeric() {
                self.reset_eater_buffer();
                self.eat_numbers();
                self.eat_char('.');
                self.eat_numbers();
                self.eat_char('e');
                self.eat_char('-');
                self.eat_numbers();
                self.cur_partial_token = PartialToken::LiteralFloat(self.clone_buf());
                self.reset_eater_buffer();
            } 
            else if self.cur() == ',' {
                self.cur_partial_token = PartialToken::Comma;
                self.commit_current_token();
                self.next();
            }
            else if self.cur().is_whitespace() {
                //if it's whitespace and there's a pending token, add it
                self.next();
            } else if let Some(s) = self.match_first_and_advance(operators) {
                self.cur_partial_token = PartialToken::Operator(String::from(s));
                self.commit_current_token();
            } else if self.cur().is_ascii_alphabetic() || self.cur() == '_' {
                self.eat_identifier();
                self.cur_partial_token = PartialToken::Identifier(self.clone_buf());
                self.reset_eater_buffer();
            }
            else {
                return Err(format!("Unrecognized token {}", self.cur()));
            }
        }
        self.commit_current_token();
        Ok(self.final_result)
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    Tokenizer::new(source).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tokenizer_simple_number() -> Result<(), String> {
        let result = tokenize("2")?;
        assert_eq!(result, [Token::LiteralInteger(2)]);
        Ok(())
    }
    #[test]
    fn tokenizer_bigger_number() -> Result<(), String> {
        let result = tokenize("22")?;
        assert_eq!(result, [Token::LiteralInteger(22)]);
        Ok(())
    }
    #[test]
    fn tokenizer_decimal_number() -> Result<(), String> {
        let result = tokenize("22.321")?;
        assert_eq!(result, [Token::LiteralFloat(Float(22.321))]);
        Ok(())
    }

    #[test]
    fn tokenizer_decimal_exponent_number() -> Result<(), String> {
        let result = tokenize("22.22e2")?;
        assert_eq!(result, [Token::LiteralFloat(Float(22.22e2))]);
        Ok(())
    }
    #[test]
    fn tokenizer_operator() -> Result<(), String> {
        let result = tokenize("+")?;
        assert_eq!(result, [Token::Operator(Operator::Plus)]);
        Ok(())
    }

    #[test]
    fn tokenizer_number_space_operator() -> Result<(), String> {
        let result = tokenize("6 +")?;
        assert_eq!(
            result,
            [
                Token::LiteralInteger(6),
                Token::Operator(Operator::Plus)
            ]
        );
        Ok(())
    }

    #[test]
    fn tokenizer_number_space_operator_space_operator() -> Result<(), String> {
        let result = tokenize("6 + +")?;
        assert_eq!(
            result,
            [
                Token::LiteralInteger(6),
                Token::Operator(Operator::Plus),
                Token::Operator(Operator::Plus)
            ]
        );
        Ok(())
    }

    #[test]
    fn tokenizer_not_equals() -> Result<(), String> {
        let result = tokenize("10 != 12")?;
        assert_eq!(
            result,
            [
                Token::LiteralInteger(10),
                Token::Operator(Operator::NotEquals),
                Token::LiteralInteger(12),
            ]
        );
        Ok(())
    }

    #[test]
    fn tokenizer_unrecognized_token() -> Result<(), &'static str> {
        let result = tokenize("10 ^ 12");
        return match result {
            Ok(_) => Err("Operator ^ doesnt exist and shouldn't be tokenized"),
            Err(_) => Ok(())
        }
    }

    #[test]
    fn tokenizer_many_operators() -> Result<(), String> {
        let result = tokenize("10 + - / * << >> ! != == -12")?;
        assert_eq!(
            result,
            [
                Token::LiteralInteger(10),
                Token::Operator(Operator::Plus),
                Token::Operator(Operator::Minus),
                Token::Operator(Operator::Divide),
                Token::Operator(Operator::Multiply),
                Token::Operator(Operator::BitShiftLeft),
                Token::Operator(Operator::BitShiftRight),
                Token::Operator(Operator::Not),
                Token::Operator(Operator::NotEquals),
                Token::Operator(Operator::Equals),
                Token::Operator(Operator::Minus),
                Token::LiteralInteger(12),
            ]
        );
        Ok(())
    }

    #[test]
    fn tokenizer_number_space_operator_space_number() -> Result<(), String> {
        let result = tokenize("6 + 6")?;
        assert_eq!(
            result,
            [
                Token::LiteralInteger(6),
                Token::Operator(Operator::Plus),
                Token::LiteralInteger(6),
            ]
        );
        Ok(())
    }

    #[test]
    fn tokenizer_number_space_operator_lots_of_space_number() -> Result<(), String> {
        let result = tokenize("6         +                                6.2312e99")?;
        assert_eq!(
            result,
            [
                Token::LiteralInteger(6),
                Token::Operator(Operator::Plus),
                Token::LiteralFloat(Float(6.2312e99)),
            ]
        );
        Ok(())
    }

    #[test]
    fn tokenizer_number_operator_number() -> Result<(), String> {
        let result = tokenize("6+6")?;
        assert_eq!(
            result,
            [
                Token::LiteralInteger(6),
                Token::Operator(Operator::Plus),
                Token::LiteralInteger(6),
            ]
        );
        Ok(())
    }

    #[test]
    fn tokenizer_space_corner_cases() -> Result<(), String> {
        let result = tokenize("   6         +             6.2312e99   ")?;
        assert_eq!(
            result,
            [
                Token::LiteralInteger(6),
                Token::Operator(Operator::Plus),
                Token::LiteralFloat(Float(6.2312e99)),
            ]
        );
        Ok(())
    }

    #[test]
    fn tokenier_openparen() -> Result<(), String> {
        let result = tokenize("(")?;
        assert_eq!(result, [Token::Operator(Operator::OpenParen)]);
        Ok(())
    }

    #[test]
    fn tokenier_closeparen() -> Result<(), String> {
        let result = tokenize(")")?;
        assert_eq!(result, [Token::Operator(Operator::CloseParen)]);
        Ok(())
    }

    #[test]
    fn tokenier_opencloseparen() -> Result<(), String> {
        let result = tokenize("()")?;
        assert_eq!(result, [Token::Operator(Operator::OpenParen), Token::Operator(Operator::CloseParen)]);
        Ok(())
    }

    #[test]
    fn tokenier_opencloseparen_with_expr() -> Result<(), String> {
        let result = tokenize("(1 + 2) * 3")?;
        assert_eq!(result, [
            Token::Operator(Operator::OpenParen), 
            Token::LiteralInteger(1),
            Token::Operator(Operator::Plus),
            Token::LiteralInteger(2),
            Token::Operator(Operator::CloseParen), 
            Token::Operator(Operator::Multiply),
            Token::LiteralInteger(3)]);
        Ok(())
    }

    #[test]
    fn tokenizer_identifier() -> Result<(), String> {
        let result = tokenize("some_identifier")?;
        assert_eq!(result, [
            Token::Identifier(String::from("some_identifier"))]);
        Ok(())
    }

    #[test]
    fn tokenizer_function_call() -> Result<(), String> {
        let result = tokenize("some_identifier(1)")?;
        assert_eq!(result, [
            Token::Identifier(String::from("some_identifier")),
            Token::Operator(Operator::OpenParen),
            Token::LiteralInteger(1),
            Token::Operator(Operator::CloseParen)
            ]);
        Ok(())
    }


    #[test]
    fn assign_operator() -> Result<(), String> {
        let result = tokenize("x = 1")?;
        assert_eq!(result, [
            Token::Identifier(String::from("x")),
            Token::Assign,
            Token::LiteralInteger(1)
            ]);
        Ok(())
    }

    #[test]
    fn none() -> Result<(), String> {
        let result = tokenize("None")?;
        assert_eq!(result, [
            Token::None
        ]);
        Ok(())
    }
}